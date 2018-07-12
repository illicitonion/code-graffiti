use super::schema::comments;

#[derive(Queryable)]
pub struct Comment {
    pub id: i32,
    pub repo_user: String,
    pub repo_name: String,
    pub path: String,
    pub sha: String,
    pub line: i32,
    pub comment: String,
}

#[derive(Insertable)]
#[table_name = "comments"]
pub struct NewComment<'a> {
    pub repo_user: &'a str,
    pub repo_name: &'a str,
    pub path: &'a str,
    pub sha: &'a str,
    pub line: i32,
    pub comment: &'a str,
}
