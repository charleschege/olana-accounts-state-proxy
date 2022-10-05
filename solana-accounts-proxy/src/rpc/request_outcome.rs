/// The outcome of a DB operation
#[derive(Debug)]
pub enum Outcome {
    /// The DB operation returned the required value of the SQL query
    Success(String),
    /// The SQL query results were NULL or an error occurred
    Failure(String),
}
