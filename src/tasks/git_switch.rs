use super::*;
use crate::{core::Target, git};

pub struct GitSwitch {
    pub branch: String,
    pub create_if_missing: bool,
}

#[async_trait::async_trait]
impl<C: Core> Task<C> for GitSwitch {
    async fn apply_repo(&self, _core: &C, repo: &core::Repo) -> Result<(), core::Error> {
        let mut create = self.create_if_missing;

        if create && git::git_branches(&repo.get_path()).await?.contains(&self.branch) {
            create = false;
        }

        git::git_switch(&repo.get_path(), &self.branch, create).await
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
    use crate::core::{Config, Repo};

    use super::*;
    use crate::tasks::GitInit;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_repo() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-switch",
            temp.path().join("repo").into(),
        );

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .with_mock_output()
            .with_mock_resolver(|r| {
                r.set_repo(Repo::new(
                    "example.com/test/cmd-branch",
                    temp.path().to_path_buf(),
                ))
            })
            .build();

        sequence![
            GitInit {},
            GitCheckout {
                branch: "main".into(),
            },
            GitSwitch {
                branch: "test".into(),
                create_if_missing: true,
            }
        ]
        .apply_repo(&core, &repo)
        .await
        .unwrap();
        assert!(repo.valid());

        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "test"
        );
    }

    #[tokio::test]
    async fn test_repo_no_create() {
        let temp = tempdir().unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test-git-switch",
            temp.path().join("repo").into(),
        );

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .with_mock_output()
            .with_mock_resolver(|r| {
                r.set_repo(Repo::new(
                    "example.com/test/cmd-branch",
                    temp.path().to_path_buf(),
                ))
            })
            .build();

        sequence![
            GitInit {},
            GitCheckout {
                branch: "main".into(),
            },
            GitSwitch {
                branch: "test".into(),
                create_if_missing: false,
            }
        ]
        .apply_repo(&core, &repo)
        .await
        .expect_err("this command should fail");
        assert!(repo.valid());

        assert_eq!(
            git::git_current_branch(&repo.get_path()).await.unwrap(),
            "main"
        );
    }

    #[tokio::test]
    async fn test_scratch() {
        let temp = tempdir().unwrap();
        let scratch = core::Scratchpad::new("2019w15", temp.path().join("scratch").into());

        let core = core::CoreBuilder::default()
            .with_config(&Config::for_dev_directory(temp.path()))
            .with_mock_output()
            .with_mock_resolver(|r| {
                r.set_repo(Repo::new(
                    "example.com/test/cmd-branch",
                    temp.path().to_path_buf(),
                ))
            })
            .build();

        let task = GitSwitch {
            branch: "test".into(),
            create_if_missing: true,
        };

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().join(".git").exists(), false);
        assert_eq!(scratch.exists(), false);
    }
}
