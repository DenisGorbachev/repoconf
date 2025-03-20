use url::Url;

pub trait RepoName {
    fn repo_name(&self) -> &str;
}

impl RepoName for Url {
    fn repo_name(&self) -> &str {
        self.path_segments()
            .and_then(|mut split| split.next_back())
            .unwrap_or("template")
    }
}
