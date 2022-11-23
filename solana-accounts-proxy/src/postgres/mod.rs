mod pg_connection;
pub use pg_connection::*;

mod ga_queries;
pub use ga_queries::*;

mod pg_row_types;
pub use pg_row_types::*;

mod gpa_queries;
pub use gpa_queries::*;

/// Print the length of the `Row`s Vec and the total size in MiB of the Vec
pub fn row_data_size_info(rows_len: usize) {
    let row_len = rows_len as f32;
    tracing::debug!("NUM OF ACCOUNTS - {:?}", &row_len);
    let row_size = row_len / (1024f32 * 1024f32);
    tracing::debug!("{:?}MiB", &row_size);
}
