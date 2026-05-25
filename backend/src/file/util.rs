use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CanonicalPath(PathBuf);

impl CanonicalPath {
    pub fn new<P>(pb: P) -> Result<Self, std::io::Error>
    where
        P: AsRef<Path>,
    {
        pb.as_ref().canonicalize().map(Self)
    }

    pub fn strip_prefix<P>(&self, prefix: P) -> Result<&Path, std::path::StripPrefixError>
    where
        P: AsRef<Path>,
    {
        self.0.strip_prefix(prefix)
    }

    #[cfg(test)]
    pub fn as_path_buf(&self) -> &PathBuf {
        &self.0
    }
}

impl std::fmt::Display for CanonicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0.display(), f)
    }
}
