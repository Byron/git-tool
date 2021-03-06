use super::*;
use crate::errors;
use http::Uri;
use serde::Deserialize;

pub struct GitHubRegistry;

#[async_trait::async_trait]
impl<C: Core> Registry<C> for GitHubRegistry {
    async fn get_entries(&self, core: &C) -> Result<Vec<String>, Error> {
        let uri: Uri = format!(
            "https://api.github.com/repos/SierraSoftworks/git-tool/git/trees/main?recursive=true"
        )
        .parse()?;

        let req = hyper::Request::get(uri)
            .header("User-Agent", version!("Git-Tool/"))
            .body(hyper::Body::empty())
            .map_err(|e| {
                errors::system_with_internal(
                    "Unable to construct web request for Git-Tool registry entries.",
                    "Please report this error to us by opening a ticket in GitHub.",
                    e,
                )
            })?;

        let resp = core.http_client().request(req).await?;

        match resp.status() {
            http::StatusCode::OK => {
                let body = hyper::body::to_bytes(resp.into_body()).await?;
                let tree: GitHubTree = serde_json::from_slice(&body)?;

                let mut entries: Vec<String> = Vec::new();

                let prefix = "registry/";
                let suffix = ".yaml";

                for node in tree.tree {
                    if node.node_type == "blob"
                        && node.path.starts_with(prefix)
                        && node.path.ends_with(suffix)
                    {
                        let len = node.path.len();
                        let name: String = node.path[prefix.len()..(len - suffix.len())].into();
                        entries.push(name);
                    }
                }

                Ok(entries)
            }
            http::StatusCode::TOO_MANY_REQUESTS => Err(errors::user(
                "GitHub has rate limited requests from your IP address.",
                "Please wait until GitHub removes this rate limit before trying again.",
            )),
            status => {
                let inner_error = errors::hyper::HyperResponseError::with_body(resp).await;
                Err(errors::system_with_internal(
                    &format!("Received an HTTP {} response from GitHub when attempting to list items in the Git-Tool registry.", status),
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                    inner_error))
            }
        }
    }

    async fn get_entry(&self, core: &C, id: &str) -> Result<Entry, Error> {
        let uri = format!(
            "https://raw.githubusercontent.com/SierraSoftworks/git-tool/main/registry/{}.yaml",
            id
        )
        .parse()?;
        let resp = core.http_client().get(uri).await?;

        match resp.status() {
            http::StatusCode::OK => {
                let body = hyper::body::to_bytes(resp.into_body()).await?;
                let entity = serde_yaml::from_slice(&body)?;
                Ok(entity)
            },
            http::StatusCode::NOT_FOUND => {
                Err(errors::user(
                    &format!("Could not find {} in the Git-Tool registry.", id),
                    "Please make sure that you've selected a configuration entry which exists in the registry. You can check this with `git-tool config list`."))
            },
            http::StatusCode::TOO_MANY_REQUESTS => {
                Err(errors::user(
                    "GitHub has rate limited requests from your IP address.",
                    "Please wait until GitHub removes this rate limit before trying again."))
            },
            status => {
                let inner_error = errors::hyper::HyperResponseError::with_body(resp).await;
                Err(errors::system_with_internal(
                    &format!("Received an HTTP {} response from GitHub when attempting to fetch /registry/{}.yaml.", status, id),
                    "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub.",
                    inner_error))
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct GitHubTree {
    pub tree: Vec<GitHubTreeNode>,
    pub truncated: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct GitHubTreeNode {
    #[serde(rename = "type")]
    pub node_type: String,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_entries() {
        let core = CoreBuilder::default().build();
        let registry = GitHubRegistry;

        let entries = registry.get_entries(&core).await.unwrap();
        assert_ne!(entries.len(), 0);
        assert!(entries.iter().any(|i| i == "apps/bash"));
    }

    #[tokio::test]
    async fn get_entry() {
        let core = CoreBuilder::default().build();
        let registry = GitHubRegistry;

        let entry = registry.get_entry(&core, "apps/bash").await.unwrap();
        assert_eq!(entry.name, "Bash");
    }
}
