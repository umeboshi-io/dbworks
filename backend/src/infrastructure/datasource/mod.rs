pub mod postgres;

use async_trait::async_trait;

use crate::domain::data::{RowsResponse, TableInfo, TableSchema};
use crate::dto::RowsQuery;

/// Trait abstracting database operations.
/// Implement this for each data source (PostgreSQL, MySQL, NoSQL, etc.)
#[async_trait]
pub trait DataSource: Send + Sync {
    /// List all user tables in the database
    async fn list_tables(&self) -> anyhow::Result<Vec<TableInfo>>;

    /// Get schema information for a specific table
    async fn get_table_schema(&self, table_name: &str) -> anyhow::Result<TableSchema>;

    /// List rows with pagination, sorting, and filtering
    async fn list_rows(&self, table_name: &str, query: &RowsQuery) -> anyhow::Result<RowsResponse>;

    /// Get a single row by its primary key value
    async fn get_row(
        &self,
        table_name: &str,
        pk_value: &str,
    ) -> anyhow::Result<serde_json::Value>;

    /// Insert a new row
    async fn insert_row(
        &self,
        table_name: &str,
        data: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value>;

    /// Update an existing row by primary key
    async fn update_row(
        &self,
        table_name: &str,
        pk_value: &str,
        data: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value>;

    /// Delete a row by primary key
    async fn delete_row(&self, table_name: &str, pk_value: &str) -> anyhow::Result<()>;
}
