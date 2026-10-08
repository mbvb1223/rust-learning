//! File-backed databases, used only through the crate's public API, like a real application would.

use std::path::Path;

use sqlx_tasks::{Task, TaskRepo, TaskUpdate, connect};

fn sqlite_url(path: &Path) -> String {
    format!("sqlite://{}", path.display())
}

#[tokio::test]
async fn connect_creates_a_missing_database_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tasks.db");
    assert!(!path.exists());

    let pool = connect(&sqlite_url(&path)).await.unwrap();
    assert!(path.exists());
    assert_eq!(TaskRepo::new(pool).list().await.unwrap(), vec![]);
}

#[tokio::test]
async fn connect_fails_when_the_directory_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing").join("tasks.db");
    assert!(connect(&sqlite_url(&path)).await.is_err());
}

#[tokio::test]
async fn tasks_survive_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    let url = sqlite_url(&dir.path().join("tasks.db"));

    let repo = TaskRepo::new(connect(&url).await.unwrap());
    repo.create("Buy milk").await.unwrap();
    repo.create("Walk the dog").await.unwrap();
    repo.create("Delete me").await.unwrap();
    let done = TaskUpdate {
        completed: Some(true),
        ..TaskUpdate::default()
    };
    repo.update(2, done).await.unwrap();
    repo.delete(3).await.unwrap();
    repo.pool().close().await;
    drop(repo);

    // Same file, new pool: the migration must be skipped, not fail or wipe the data.
    let repo = TaskRepo::new(connect(&url).await.unwrap());
    assert_eq!(
        repo.list().await.unwrap(),
        vec![
            Task {
                id: 1,
                title: "Buy milk".to_string(),
                completed: false,
            },
            Task {
                id: 2,
                title: "Walk the dog".to_string(),
                completed: true,
            },
        ]
    );
    assert_eq!(repo.create("After restart").await.unwrap().id, 4);
}
