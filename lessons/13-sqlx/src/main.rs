//! A tiny CLI over the repository. Data lives in `tasks.db` (or `$DATABASE_URL`), so it
//! survives between runs.

use std::env;
use std::error::Error;

use sqlx_tasks::{TaskRepo, TaskUpdate, connect};

const USAGE: &str = "usage: cargo run -- [list | add <title> | done <id> | rm <id>]";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:tasks.db".to_string());
    let repo = TaskRepo::new(connect(&url).await?);

    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        [] | ["list"] => {
            for task in repo.list().await? {
                let mark = if task.completed { 'x' } else { ' ' };
                println!("{:>4} [{mark}] {}", task.id, task.title);
            }
        }
        ["add", title] => {
            let task = repo.create(title).await?;
            println!("created #{}", task.id);
        }
        ["done", id] => {
            let changes = TaskUpdate {
                completed: Some(true),
                ..TaskUpdate::default()
            };
            match repo.update(id.parse()?, changes).await? {
                Some(task) => println!("completed #{}", task.id),
                None => println!("no task #{id}"),
            }
        }
        ["rm", id] => {
            if repo.delete(id.parse()?).await? {
                println!("deleted #{id}");
            } else {
                println!("no task #{id}");
            }
        }
        _ => return Err(USAGE.into()),
    }

    repo.pool().close().await;
    Ok(())
}
