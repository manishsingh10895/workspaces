#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Database Erorr")]
    DbError(String),
}
