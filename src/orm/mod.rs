use super::models::LineInRepo;
use diesel::{self, pg::PgConnection, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use std::borrow::Borrow;

mod models;
use orm::models::{Comment, NewComment};

pub mod schema;

pub type DatabaseConnection = PgConnection;

pub fn connect(database: &str) -> Result<DatabaseConnection, String> {
    PgConnection::establish(database).map_err(|e| format!("Connecting to database: {}", e))
}

pub fn comments_for_line<C: Borrow<DatabaseConnection>>(
    connection: C,
    context: &LineInRepo,
) -> Result<Vec<String>, String> {
    use self::schema::comments::dsl::*;
    let ret = comments
        .filter(repo_user.eq(&context.repo_user))
        .filter(repo_name.eq(&context.repo_name))
        .filter(path.eq(&context.path))
        .filter(sha.eq(&context.sha))
        .filter(line.eq(context.line as i32))
        .order(id.asc())
        .load::<Comment>(connection.borrow())
        .map_err(|e| format!("{}", e))?
        .into_iter()
        .map(|c| c.comment)
        .collect();
    Ok(ret)
}

pub fn leave_comment(
    connection: &DatabaseConnection,
    context: &LineInRepo,
    body: &str,
) -> Result<(), String> {
    let comment = NewComment {
        repo_user: &context.repo_user,
        repo_name: &context.repo_name,
        path: &context.path,
        sha: &context.sha,
        line: context.line as i32,
        comment: body,
    };

    diesel::insert_into(schema::comments::table)
        .values(&comment)
        .execute(connection)
        .map_err(|err| format!("Error saving comment: {}", err))?;
    Ok(())
}
