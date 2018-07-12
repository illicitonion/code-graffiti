use super::models::LineInRepo;
use postgres::{Connection, TlsMode};
use std::borrow::Borrow;

pub type DatabaseConnection = Connection;

pub fn connect(database: &str) -> Result<DatabaseConnection, String> {
    Connection::connect(database, TlsMode::None)
        .map_err(|e| format!("Connecting to database: {}", e))
}

pub fn comments_for_line<C: Borrow<DatabaseConnection>>(
    connection: C,
    context: &LineInRepo,
) -> Result<Vec<String>, String> {
    Ok(connection
        .borrow()
        .query(
            "SELECT comment FROM comments WHERE
repo_user = $1 AND
repo_name = $2 AND
path = $3 AND
sha = $4 AND
line = $5
ORDER BY id ASC",
            &[
                &context.repo_user,
                &context.repo_name,
                &context.path,
                &context.sha,
                &(context.line as i32),
            ],
        )
        .map_err(|err| format!("Error querying: {}", err))?
        .iter()
        .map(|row| row.get(0))
        .collect())
}

pub fn leave_comment(
    connection: &DatabaseConnection,
    context: &LineInRepo,
    body: &str,
) -> Result<(), String> {
    connection.execute(
        "INSERT INTO comments (repo_user, repo_name, path, sha, line, comment) VALUES ($1, $2, $3, $4, $5, $6)",
        &[
            &context.repo_user,
            &context.repo_name,
            &context.path,
            &context.sha,
            &(context.line as i32),
            &body,
        ],
    ).map_err(|err| format!("Error inserting {}", err))?;
    Ok(())
}
