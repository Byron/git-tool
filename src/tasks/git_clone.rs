use super::*;
use crate::{core::Target, errors, git};

pub struct GitClone {}

#[async_trait::async_trait]
impl<C: Core> Task<C> for GitClone {
    async fn apply_repo(&self, core: &C, repo: &core::Repo) -> Result<(), core::Error> {
        if repo.exists() {
            return Ok(());
        }

        let service = core.config().get_service(&repo.get_domain()).ok_or(
            errors::user(
                &format!("Could not find a service entry in your config file for {}", repo.get_domain()), 
                &format!("Ensure that your git-tool configuration has a service entry for this service, or add it with `git-tool config add service/{}`", repo.get_domain()))
        )?;

        let url = if core.config().get_features().use_http_transport() {
            service.get_http_url(repo)?
        } else {
            service.get_git_url(repo)?
        };

        git::git_clone(&repo.get_path(), &url).await
    }

    async fn apply_scratchpad(
        &self,
        _core: &C,
        _scratch: &core::Scratchpad,
    ) -> Result<(), core::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo_basic() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new(
            "github.com/git-fixtures/basic",
            temp.path().join("repo").into(),
        );

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        GitClone {}.apply_repo(&core, &repo).await.unwrap();
        assert!(repo.valid());
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch").into());

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .build();

        let task = GitClone {};

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
        assert_eq!(scratch.exists(), false);
    }
}
