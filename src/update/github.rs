use super::*;
use crate::{core::Core, errors};
use http::Uri;
use hyper::body::HttpBody;
use serde::Deserialize;
use std::env::consts::{ARCH, OS};

pub struct GitHubSource {
    pub repo: String,
    pub artifact_prefix: String,
    pub release_tag_prefix: String,
}

impl Default for GitHubSource {
    fn default() -> Self {
        Self {
            repo: "SierraSoftworks/git-tool".to_string(),
            artifact_prefix: "git-tool-".to_string(),
            release_tag_prefix: "v".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl<C: Core> Source<C> for GitHubSource {
    async fn get_releases(&self, core: &C) -> Result<Vec<Release>, crate::core::Error> {
        let uri: Uri = format!("https://api.github.com/repos/{}/releases", self.repo).parse()?;
        info!("Making GET request to {} to check for new releases.", uri);

        let req = hyper::Request::get(uri)
            .header("User-Agent", version!("Git-Tool/v"))
            .body(hyper::Body::empty())
            .map_err(|e| {
                errors::system_with_internal(
                    "Unable to construct web request for Git-Tool releases.",
                    "Please report this error to us by opening a ticket in GitHub.",
                    e,
                )
            })?;

        let resp = core.http_client().request(req).await?;
        debug!(
            "Received HTTP {} {} from GitHub when requesting releases.",
            resp.status().as_u16(),
            resp.status().canonical_reason().unwrap_or("UNKNOWN")
        );

        match resp.status() {
            http::StatusCode::OK => {
                let body = hyper::body::to_bytes(resp.into_body()).await?;
                let releases: Vec<GitHubRelease> = serde_json::from_slice(&body)?;

                self.get_releases_from_response(releases)
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

    async fn get_binary<W: std::io::Write + Send>(
        &self,
        core: &C,
        release: &Release,
        variant: &ReleaseVariant,
        into: &mut W,
    ) -> Result<(), crate::core::Error> {
        let uri: Uri = format!(
            "https://github.com/{}/releases/download/{}/{}",
            self.repo, release.id, variant.id
        )
        .parse()?;

        self.download_to_file(core, uri, into).await
    }
}

impl GitHubSource {
    fn get_releases_from_response(
        &self,
        releases: Vec<GitHubRelease>,
    ) -> Result<Vec<Release>, errors::Error> {
        let mut output: Vec<Release> = Vec::new();
        output.reserve(releases.len());

        for r in releases {
            if !r.tag_name.starts_with(&self.release_tag_prefix) {
                continue;
            }

            match r.tag_name[self.release_tag_prefix.len()..].parse() {
                Ok(version) => output.push(Release {
                    id: r.tag_name.clone(),
                    changelog: r.body.clone(),
                    version,
                    variants: self.get_variants_from_response(&r),
                }),
                Err(_) => {}
            }
        }

        Ok(output)
    }

    fn get_variants_from_response(&self, release: &GitHubRelease) -> Vec<ReleaseVariant> {
        let mut variants = Vec::new();

        for a in release.assets.iter() {
            if !a.name.starts_with(&self.artifact_prefix) {
                continue;
            }

            let spec_name = a.name[self.artifact_prefix.len()..]
                .trim_end_matches(".exe")
                .to_string();
            let mut parts = spec_name.split('-');

            let arch = match parts.next_back() {
                Some(spec_arch) => spec_arch.to_string(),
                None => ARCH.to_string(),
            };

            let platform = match parts.next_back() {
                Some(os) => os.to_string(),
                None => OS.to_string(),
            };

            variants.push(ReleaseVariant {
                id: a.name.clone(),
                arch,
                platform,
            })
        }

        variants
    }

    async fn download_to_file<C: Core, W: std::io::Write + Send>(
        &self,
        core: &C,
        uri: Uri,
        into: &mut W,
    ) -> Result<(), errors::Error> {
        let mut recursion_limit = 5;
        let mut current_uri = uri.clone();

        while recursion_limit > 0 {
            recursion_limit -= 1;

            let req = hyper::Request::get(current_uri)
                .header("User-Agent", version!("Git-Tool/v"))
                .body(hyper::Body::empty())
                .map_err(|e| {
                    errors::system_with_internal(
                        "Unable to construct web request for Git-Tool releases.",
                        "Please report this error to us by opening a ticket in GitHub.",
                        e,
                    )
                })?;

            let resp = core.http_client().request(req).await?;

            match resp.status() {
                http::StatusCode::OK => {
                    let mut body = resp.into_body();

                    while let Some(buf) = body.data().await {
                        let buf = buf?;
                        into.write_all(&buf)?;
                    }

                    return Ok(())
                },
                http::StatusCode::FOUND | http::StatusCode::MOVED_PERMANENTLY => {
                    let new_location = resp.headers().get("Location")
                        .ok_or(errors::system(
                    "GitHub returned a redirect response without an appropriate Location header.",
                    "Please report this issue to us on GitHub so that we can investigate and implement a fix/workaround for the problem."))
                    .and_then(|h| h.to_str()
                        .map_err(|e| errors::system_with_internal(
                            "GitHub returned a redirect response without an appropriate Location header.",
                            "Please report this issue to us on GitHub so that we can investigate and implement a fix/workaround for the problem.",
                            e)))?;

                    current_uri = new_location.parse()?;
                    continue;
                },
                http::StatusCode::TOO_MANY_REQUESTS => {
                    return Err(errors::user(
                        "GitHub has rate limited requests from your IP address.",
                        "Please wait until GitHub removes this rate limit before trying again."))
                },
                status => {
                    return Err(errors::system(
                        &format!("Received an HTTP {} response from GitHub when attempting to download the update for your platform ({}).", status, uri),
                        "Please read the error message below and decide if there is something you can do to fix the problem, or report it to us on GitHub."))
                }
            }
        }

        Err(errors::system(
            &format!("Reached redirect limit when attempting to download the update for your platform ({})", uri),
            "Please report this issue to us on GitHub so that we can investigate and implement a fix/workaround for the problem."))
    }
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    pub name: String,
    pub tag_name: String,
    pub body: String,
    pub prerelease: bool,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::CoreBuilder;
    use std::{
        io::Write,
        sync::{Arc, Mutex},
    };

    mock_connector_in_order!(MockGetReleasesFlow {
r#"HTTP/1.1 200 OK
Content-Type: application/vnd.github.v3+json
Content-Length: 321

[
    {
        "name": "Version 2.0.0",
        "tag_name":"v2.0.0",
        "body": "Example Release",
        "prerelease": false,
        "assets": [
            { "name": "git-tool-windows-amd64.exe" },
            { "name": "git-tool-linux-amd64" },
            { "name": "git-tool-darwin-amd64" }
        ]
    }
]
"#

r#"HTTP/1.1 200 OK
Content-Type: application/octet-stream
Content-Length: 8

testdata
"#});

    #[tokio::test]
    async fn test_get_releases() {
        let source = GitHubSource::default();
        let core = CoreBuilder::default()
            .with_http_connector(MockGetReleasesFlow::default())
            .build();

        let releases = source.get_releases(&core).await.unwrap();

        assert_eq!(releases.len(), 1);
        for release in releases {
            assert!(
                release.id.contains(&release.version.to_string()),
                "the release version should be derived from the tag"
            );
            assert_ne!(
                &release.changelog, "",
                "the release changelog should not be empty"
            );
        }
    }

    #[tokio::test]
    async fn test_download() {
        let source = GitHubSource::default();
        let core = CoreBuilder::default()
            .with_http_connector(MockGetReleasesFlow::default())
            .build();

        let releases = source.get_releases(&core).await.unwrap();
        let latest =
            Release::get_latest(releases.iter()).expect("There should be an available release");
        let variant = latest
            .variants
            .first()
            .expect("There should be a variant available");

        let mut target = sink();

        source
            .get_binary(&core, &latest, &variant, &mut target)
            .await
            .unwrap();

        assert!(target.get_length() > 0);
    }

    fn sink() -> Sink {
        Sink {
            length: Arc::new(Mutex::new(0)),
        }
    }

    struct Sink {
        length: Arc<Mutex<usize>>,
    }

    impl Sink {
        pub fn get_length(&self) -> usize {
            self.length.lock().map(|m| *m).unwrap_or_default()
        }
    }

    impl Write for Sink {
        #[inline]
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.length
                .lock()
                .map(|mut m| {
                    *m += buf.len();
                    buf.len()
                })
                .map_err(|_| std::io::ErrorKind::Other.into())
        }

        #[inline]
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
