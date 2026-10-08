//! Lesson 13 — persist tasks in SQLite with SQLx. Replace every `todo!()` until `cargo test` passes.

use std::fmt;

use sqlx::SqlitePool;

/// Longest allowed title, counted in characters (not bytes) after trimming.
pub const MAX_TITLE_CHARS: usize = 200;

/// One row of the `tasks` table. `FromRow` maps columns to fields by name.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub completed: bool,
}

/// A partial update: a `None` field leaves that column unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug)]
pub enum RepoError {
    /// The input breaks a domain rule. The message is written for the API client.
    Validation(String),
    /// The database failed. Log it; never show its details to a client.
    Database(sqlx::Error),
}

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepoError::Validation(message) => f.write_str(message),
            RepoError::Database(err) => write!(f, "database error: {err}"),
        }
    }
}

impl std::error::Error for RepoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RepoError::Validation(_) => None,
            RepoError::Database(err) => Some(err),
        }
    }
}

impl From<sqlx::Error> for RepoError {
    /// Wraps any SQLx error in `RepoError::Database`, so `?` works on SQLx calls in the repository.
    fn from(err: sqlx::Error) -> Self {
        todo!()
    }
}

/// Trims surrounding whitespace and checks the result is 1..=[`MAX_TITLE_CHARS`] characters.
///
/// Returns the trimmed title as a slice of the input. Inner whitespace is kept as is.
/// `Validation` if the trimmed title is empty or longer than [`MAX_TITLE_CHARS`] characters.
pub fn validate_title(title: &str) -> Result<&str, RepoError> {
    todo!()
}

/// Opens the SQLite database at `url` (e.g. `"sqlite:tasks.db"`), creating the file if it is
/// missing, then applies pending migrations from `migrations/`.
///
/// The parent directory must already exist: SQLite creates the file, not directories.
/// Connecting to an existing database is safe: already-applied migrations are skipped.
pub async fn connect(url: &str) -> Result<SqlitePool, sqlx::Error> {
    todo!()
}

/// A new, empty, migrated in-memory database, for tests.
///
/// The pool holds exactly one connection and never closes it for being idle or old, so the data
/// lives exactly as long as the pool. Every call returns a separate database.
pub async fn connect_in_memory() -> Result<SqlitePool, sqlx::Error> {
    todo!()
}

/// Task storage. Cloning is cheap: clones share the same connection pool.
#[derive(Debug, Clone)]
pub struct TaskRepo {
    pool: SqlitePool,
}

impl TaskRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Inserts a task with the trimmed title and `completed = false`, and returns it.
    ///
    /// Ids start at 1 in a new database, increase with every insert, and are never reused, even
    /// after a delete. `Validation` for an invalid title (see [`validate_title`]); nothing is stored.
    pub async fn create(&self, title: &str) -> Result<Task, RepoError> {
        todo!()
    }

    /// Every task, ordered by id ascending. An empty table gives an empty `Vec`.
    pub async fn list(&self) -> Result<Vec<Task>, RepoError> {
        todo!()
    }

    /// The task with `id`, or `None` if there is none.
    pub async fn get(&self, id: i64) -> Result<Option<Task>, RepoError> {
        todo!()
    }

    /// Applies the `Some` fields of `changes` to task `id` and returns the updated task.
    ///
    /// - A new title is trimmed and validated like in [`TaskRepo::create`].
    /// - `Validation` if both fields are `None` or the title is invalid. Input is validated
    ///   before the database is queried, so this also applies when `id` doesn't exist.
    /// - `Ok(None)` if there is no task with `id`.
    pub async fn update(&self, id: i64, changes: TaskUpdate) -> Result<Option<Task>, RepoError> {
        todo!()
    }

    /// Deletes task `id`. `true` if a task was deleted, `false` if there was none.
    pub async fn delete(&self, id: i64) -> Result<bool, RepoError> {
        todo!()
    }

    /// Creates one task per title in a single transaction and returns them in input order.
    /// Titles are trimmed and validated like in [`TaskRepo::create`].
    ///
    /// All or nothing: if any title is invalid, returns `Validation` and stores none of `titles`.
    /// An empty slice returns an empty `Vec`.
    pub async fn create_many(&self, titles: &[&str]) -> Result<Vec<Task>, RepoError> {
        todo!()
    }

    /// Marks every task in `ids` completed, in a single transaction.
    ///
    /// All or nothing: if any id doesn't exist, returns `Ok(false)` and changes no task.
    /// Otherwise `Ok(true)`. Already-completed tasks and repeated ids count as existing.
    /// An empty slice returns `Ok(true)`.
    pub async fn complete_many(&self, ids: &[i64]) -> Result<bool, RepoError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn repo() -> TaskRepo {
        TaskRepo::new(connect_in_memory().await.expect("in-memory database"))
    }

    fn task(id: i64, title: &str, completed: bool) -> Task {
        Task {
            id,
            title: title.to_string(),
            completed,
        }
    }

    fn assert_validation<T: fmt::Debug>(result: Result<T, RepoError>) {
        assert!(
            matches!(result, Err(RepoError::Validation(_))),
            "expected Err(Validation(_)), got {result:?}"
        );
    }

    fn set_title(title: &str) -> TaskUpdate {
        TaskUpdate {
            title: Some(title.to_string()),
            ..TaskUpdate::default()
        }
    }

    fn set_completed(completed: bool) -> TaskUpdate {
        TaskUpdate {
            completed: Some(completed),
            ..TaskUpdate::default()
        }
    }

    #[test]
    fn sqlx_errors_become_database_errors() {
        let err = RepoError::from(sqlx::Error::RowNotFound);
        assert!(matches!(err, RepoError::Database(_)), "got {err:?}");
    }

    #[test]
    fn validate_title_trims_and_keeps_inner_whitespace() {
        assert_eq!(validate_title("Buy milk").unwrap(), "Buy milk");
        assert_eq!(validate_title("  Buy  milk \n").unwrap(), "Buy  milk");
    }

    #[test]
    fn validate_title_rejects_empty_and_blank() {
        assert_validation(validate_title(""));
        assert_validation(validate_title("   "));
        assert_validation(validate_title("\t\n"));
    }

    #[test]
    fn validate_title_enforces_the_length_limit_after_trimming() {
        assert_eq!(validate_title("a").unwrap(), "a");
        let longest = "a".repeat(200);
        assert_eq!(validate_title(&longest).unwrap(), longest);
        assert_eq!(validate_title(&format!("  {longest}  ")).unwrap(), longest);
        assert_validation(validate_title(&"a".repeat(201)));
    }

    #[test]
    fn validate_title_counts_chars_not_bytes() {
        let accents = "é".repeat(200); // 400 bytes
        assert_eq!(validate_title(&accents).unwrap(), accents);
        assert_validation(validate_title(&"é".repeat(201)));
    }

    #[tokio::test]
    async fn new_in_memory_database_is_empty() {
        assert_eq!(repo().await.list().await.unwrap(), vec![]);
    }

    #[tokio::test]
    async fn in_memory_databases_are_independent() {
        let first = repo().await;
        let second = repo().await;
        first.create("Only in the first").await.unwrap();
        assert_eq!(second.list().await.unwrap(), vec![]);
    }

    #[tokio::test]
    async fn create_returns_the_stored_task() {
        let repo = repo().await;
        let created = repo.create("  Buy milk ").await.unwrap();
        assert_eq!(created, task(1, "Buy milk", false));
        assert_eq!(repo.get(1).await.unwrap(), Some(created));
    }

    #[tokio::test]
    async fn create_assigns_increasing_ids() {
        let repo = repo().await;
        for expected in 1..=3 {
            assert_eq!(repo.create("task").await.unwrap().id, expected);
        }
    }

    #[tokio::test]
    async fn create_rejects_invalid_titles_and_stores_nothing() {
        let repo = repo().await;
        assert_validation(repo.create("").await);
        assert_validation(repo.create("   ").await);
        assert_validation(repo.create(&"a".repeat(201)).await);
        assert_eq!(repo.list().await.unwrap(), vec![]);
    }

    #[tokio::test]
    async fn titles_are_bound_as_data_not_sql() {
        let repo = repo().await;
        let title = "Robert'); DROP TABLE tasks; --";
        repo.create(title).await.unwrap();
        assert_eq!(repo.list().await.unwrap(), vec![task(1, title, false)]);
    }

    #[tokio::test]
    async fn list_orders_by_id() {
        let repo = repo().await;
        repo.create("b").await.unwrap();
        repo.create("a").await.unwrap();
        repo.create("c").await.unwrap();
        assert_eq!(
            repo.list().await.unwrap(),
            vec![
                task(1, "b", false),
                task(2, "a", false),
                task(3, "c", false)
            ]
        );
    }

    #[tokio::test]
    async fn get_missing_task_returns_none() {
        let repo = repo().await;
        repo.create("exists").await.unwrap();
        assert_eq!(repo.get(2).await.unwrap(), None);
        assert_eq!(repo.get(-1).await.unwrap(), None);
    }

    #[tokio::test]
    async fn update_title_keeps_completed() {
        let repo = repo().await;
        repo.create("old").await.unwrap();
        repo.update(1, set_completed(true)).await.unwrap();
        let updated = repo.update(1, set_title("  new ")).await.unwrap();
        assert_eq!(updated, Some(task(1, "new", true)));
        assert_eq!(repo.get(1).await.unwrap(), updated);
    }

    #[tokio::test]
    async fn update_completed_keeps_title() {
        let repo = repo().await;
        repo.create("title").await.unwrap();
        let done = repo.update(1, set_completed(true)).await.unwrap();
        assert_eq!(done, Some(task(1, "title", true)));
        let undone = repo.update(1, set_completed(false)).await.unwrap();
        assert_eq!(undone, Some(task(1, "title", false)));
        assert_eq!(repo.get(1).await.unwrap(), undone);
    }

    #[tokio::test]
    async fn update_both_fields() {
        let repo = repo().await;
        repo.create("old").await.unwrap();
        let changes = TaskUpdate {
            title: Some("new".to_string()),
            completed: Some(true),
        };
        assert_eq!(
            repo.update(1, changes).await.unwrap(),
            Some(task(1, "new", true))
        );
    }

    #[tokio::test]
    async fn update_missing_task_returns_none() {
        let repo = repo().await;
        assert_eq!(repo.update(1, set_completed(true)).await.unwrap(), None);
        assert_eq!(repo.update(1, set_title("new")).await.unwrap(), None);
        assert_eq!(repo.list().await.unwrap(), vec![]);
    }

    #[tokio::test]
    async fn update_rejects_invalid_input_and_changes_nothing() {
        let repo = repo().await;
        repo.create("keep").await.unwrap();
        assert_validation(repo.update(1, TaskUpdate::default()).await);
        assert_validation(repo.update(1, set_title("  ")).await);
        assert_validation(repo.update(1, set_title(&"a".repeat(201))).await);
        let invalid_title_and_completed = TaskUpdate {
            title: Some(String::new()),
            completed: Some(true),
        };
        assert_validation(repo.update(1, invalid_title_and_completed).await);
        assert_eq!(repo.get(1).await.unwrap(), Some(task(1, "keep", false)));
    }

    #[tokio::test]
    async fn update_validates_before_looking_up_the_task() {
        let repo = repo().await;
        assert_validation(repo.update(42, TaskUpdate::default()).await);
        assert_validation(repo.update(42, set_title("")).await);
    }

    #[tokio::test]
    async fn delete_removes_the_task_once() {
        let repo = repo().await;
        repo.create("a").await.unwrap();
        repo.create("b").await.unwrap();
        assert!(repo.delete(1).await.unwrap());
        assert_eq!(repo.get(1).await.unwrap(), None);
        assert!(!repo.delete(1).await.unwrap());
        assert_eq!(repo.list().await.unwrap(), vec![task(2, "b", false)]);
    }

    #[tokio::test]
    async fn delete_missing_task_returns_false() {
        let repo = repo().await;
        assert!(!repo.delete(1).await.unwrap());
    }

    #[tokio::test]
    async fn ids_are_not_reused_after_delete() {
        let repo = repo().await;
        repo.create("a").await.unwrap();
        repo.create("b").await.unwrap();
        repo.delete(2).await.unwrap();
        assert_eq!(repo.create("c").await.unwrap().id, 3);
    }

    #[tokio::test]
    async fn create_many_inserts_in_order() {
        let repo = repo().await;
        repo.create("existing").await.unwrap();
        let created = repo.create_many(&["one", " two "]).await.unwrap();
        assert_eq!(created, vec![task(2, "one", false), task(3, "two", false)]);
        assert_eq!(repo.list().await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn create_many_with_no_titles_creates_nothing() {
        let repo = repo().await;
        assert_eq!(repo.create_many(&[]).await.unwrap(), vec![]);
        assert_eq!(repo.list().await.unwrap(), vec![]);
    }

    #[tokio::test]
    async fn create_many_is_all_or_nothing() {
        let repo = repo().await;
        repo.create("existing").await.unwrap();
        assert_validation(repo.create_many(&["first", "second", "  ", "last"]).await);
        assert_eq!(repo.list().await.unwrap(), vec![task(1, "existing", false)]);
    }

    #[tokio::test]
    async fn complete_many_marks_only_the_given_tasks() {
        let repo = repo().await;
        repo.create_many(&["a", "b", "c"]).await.unwrap();
        assert!(repo.complete_many(&[1, 3]).await.unwrap());
        assert_eq!(
            repo.list().await.unwrap(),
            vec![task(1, "a", true), task(2, "b", false), task(3, "c", true)]
        );
    }

    #[tokio::test]
    async fn complete_many_with_a_missing_id_changes_nothing() {
        let repo = repo().await;
        repo.create_many(&["a", "b"]).await.unwrap();
        assert!(!repo.complete_many(&[1, 99, 2]).await.unwrap());
        assert_eq!(
            repo.list().await.unwrap(),
            vec![task(1, "a", false), task(2, "b", false)]
        );
    }

    #[tokio::test]
    async fn complete_many_accepts_completed_repeated_and_empty_ids() {
        let repo = repo().await;
        repo.create_many(&["a", "b"]).await.unwrap();
        repo.update(1, set_completed(true)).await.unwrap();
        assert!(repo.complete_many(&[]).await.unwrap());
        assert!(repo.complete_many(&[1, 2, 2]).await.unwrap());
        assert_eq!(
            repo.list().await.unwrap(),
            vec![task(1, "a", true), task(2, "b", true)]
        );
    }

    #[tokio::test]
    async fn clones_share_one_database_across_tokio_tasks() {
        let repo = repo().await;
        let clone = repo.clone();
        let created = tokio::spawn(async move { clone.create("from a spawned task").await })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(repo.list().await.unwrap(), vec![created]);
    }
}
