use super::super::errors;
use super::*;
use crate::tasks::*;
use clap::{App, Arg, ArgMatches, SubCommand};

pub struct NewCommand {}

impl Command for NewCommand {
    fn name(&self) -> String {
        "new".into()
    }

    fn app<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(&self.name())
            .version("1.0")
            .about("creates a new repository")
            .alias("new")
            .alias("n")
            .alias("create")
            .after_help("Creates a new repository with the provided name.")
            .arg(
                Arg::with_name("repo")
                    .help("The name of the repository to create.")
                    .index(1),
            )
            .arg(
                Arg::with_name("open")
                    .long("open")
                    .short("o")
                    .help("opens the repository in your default application after it is created."),
            )
    }
}

#[async_trait]
impl<C: Core> CommandRunnable<C> for NewCommand {
    async fn run<'a>(&self, core: &C, matches: &ArgMatches<'a>) -> Result<i32, errors::Error> {
        let repo = match matches.value_of("repo") {
            Some(name) => core.resolver().get_best_repo(name)?,
            None => Err(errors::user(
                "No repository name provided for creation.",
                "Please provide a repository name when calling this method: git-tool new my/repo",
            ))?,
        };

        if repo.valid() {
            return Ok(0);
        }

        let tasks = sequence![
            GitInit {},
            GitRemote { name: "origin" },
            GitCheckout { branch: "main" },
            CreateRemote {}
        ];

        tasks.apply_repo(core, &repo).await?;

        if matches.is_present("open") || core.config().get_features().open_new_repo_in_default_app()
        {
            let app = core.config().get_default_app().ok_or(errors::user(
                "No default application available.",
                "Make sure that you add an app to your config file using 'git-tool config add apps/bash' or similar."))?;

            let status = core.launcher().run(app, &repo).await?;
            return Ok(status);
        }

        Ok(0)
    }

    async fn complete<'a>(&self, core: &C, completer: &Completer, _matches: &ArgMatches<'a>) {
        completer.offer("--open");
        match core.resolver().get_repos() {
            Ok(repos) => {
                let mut namespaces = std::collections::HashSet::new();
                let default_svc = core
                    .config()
                    .get_default_service()
                    .map(|s| s.get_domain())
                    .unwrap_or_default();

                for repo in repos {
                    if repo.get_domain() == default_svc {
                        namespaces.insert(repo.get_domain() + "/");
                    }

                    namespaces.insert(format!("{}/{}/", repo.get_domain(), repo.get_namespace()));
                }

                completer.offer_many(namespaces.iter().map(|s| s.as_str()));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core::{Config, CoreBuilder};
    use super::*;

    #[tokio::test]
    async fn run_partial() {
        let cmd = NewCommand {};

        let args = cmd
            .app()
            .get_matches_from(vec!["new", "test/new-repo-partial"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::for_dev_directory(temp.path());

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_keychain(|s| {
                s.set_token("github.com", "test_token").unwrap();
            })
            .with_http_connector(
                crate::online::service::github::mocks::NewRepoSuccessFlow::default(),
            )
            .build();

        let repo = core
            .resolver()
            .get_best_repo("github.com/test/new-repo-partial")
            .unwrap();
        assert_eq!(repo.valid(), false);

        cmd.run(&core, &args).await.unwrap();

        assert!(repo.valid());
    }

    #[tokio::test]
    async fn run_fully_qualified() {
        let cmd = NewCommand {};

        let args = cmd
            .app()
            .get_matches_from(vec!["new", "github.com/test/new-repo-full"]);

        let temp = tempfile::tempdir().unwrap();
        let cfg = Config::for_dev_directory(temp.path());

        let core = CoreBuilder::default()
            .with_config(&cfg)
            .with_mock_keychain(|s| {
                s.set_token("github.com", "test_token").unwrap();
            })
            .with_http_connector(
                crate::online::service::github::mocks::NewRepoSuccessFlow::default(),
            )
            .build();

        let repo = core
            .resolver()
            .get_best_repo("github.com/test/new-repo-full")
            .unwrap();
        assert_eq!(repo.valid(), false);

        cmd.run(&core, &args).await.unwrap();

        assert!(repo.valid());
    }
}
