use std::fmt;

#[derive(Clone)]
pub struct LineInRepo {
    pub repo_user: String,
    pub repo_name: String,
    pub sha: String,
    pub path: String,
    pub line: usize,
}

pub struct GithubRepository<'a> {
    pub user: &'a str,
    pub name: &'a str,
}

impl<'a> fmt::Display for GithubRepository<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}/{}", self.user, self.name)
    }
}
