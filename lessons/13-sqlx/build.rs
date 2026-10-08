// `sqlx::migrate!()` embeds `migrations/` at compile time, but Cargo only notices new files there
// if a build script asks it to.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
