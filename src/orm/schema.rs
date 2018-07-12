table! {
    comments (id) {
        id -> Int4,
        repo_user -> Varchar,
        repo_name -> Varchar,
        path -> Varchar,
        sha -> Varchar,
        line -> Int4,
        comment -> Text,
    }
}
