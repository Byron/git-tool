use super::{*, core::Target};

pub struct NewFolder {}

#[async_trait::async_trait]
impl<F: FileSource, L: Launcher, R: Resolver> Task<F, L, R> for NewFolder {
    async fn apply_repo(&self, _core: &core::Core<F, L, R>, repo: &core::Repo) -> Result<(), core::Error> {
        let path = repo.get_path();

        std::fs::create_dir_all(path)?;

        Ok(())
    }

    async fn apply_scratchpad(&self, _core: &core::Core<F, L, R>, scratch: &core::Scratchpad) -> Result<(), core::Error> {
        let path = scratch.get_path();

        std::fs::create_dir_all(path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[tokio::test]
    async fn test_repo_exists() {
        let temp = TempDir::new("gt-tasks-new-folder").unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test1", 
            temp.path().into());

        let core = get_core();
        let task = NewFolder{};
        
        assert_eq!(repo.get_path().exists(), true);

        task.apply_repo(&core, &repo).await.unwrap();
        assert_eq!(repo.get_path().exists(), true);
    }

    #[tokio::test]
    async fn test_repo_new() {
        let temp = TempDir::new("gt-tasks-new-folder").unwrap();
        let repo = core::Repo::new(
            "github.com/sierrasoftworks/test3", 
            temp.path().join("repo").into());

        let core = get_core();
        let task = NewFolder{};
        
        assert_eq!(repo.get_path().exists(), false);

        task.apply_repo(&core, &repo).await.unwrap();
        assert_eq!(repo.get_path().exists(), true);

        std::fs::remove_dir(repo.get_path()).unwrap();
    }

    #[tokio::test]
    async fn test_scratch_exists() {
        let temp = TempDir::new("gt-tasks-new-folder").unwrap();
        let scratch = core::Scratchpad::new(
            "2019w15", 
            temp.path().into());

        let core = get_core();
        let task = NewFolder{};
        
        assert_eq!(scratch.get_path().exists(), true);

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().exists(), true);
    }

    #[tokio::test]
    async fn test_scratch_new() {
        let temp = TempDir::new("gt-tasks-new-folder").unwrap();
        let scratch = core::Scratchpad::new(
            "2019w19", 
            temp.path().join("scratch").into());

        let core = get_core();
        let task = NewFolder{};
        
        assert_eq!(scratch.get_path().exists(), false);

        task.apply_scratchpad(&core, &scratch).await.unwrap();
        assert_eq!(scratch.get_path().exists(), true);

        std::fs::remove_dir(scratch.get_path()).unwrap();
    }

    fn get_core() -> core::Core {
        core::Core::builder()
            .with_config(&core::Config::for_dev_directory(get_dev_dir().as_path()))
            .build()
    }

    fn get_dev_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(file!())
            .parent()
            .and_then(|f| f.parent())
            .and_then(|f| f.parent())
            .and_then(|f| Some(f.join("test")))
            .and_then(|f| Some(f.join("devdir")))
            .unwrap()
    }
}