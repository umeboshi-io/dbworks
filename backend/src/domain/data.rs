use serde::Serialize;

/// Metadata about a database table
#[derive(Debug, Clone, Serialize)]
pub struct TableInfo {
    pub table_name: String,
    pub table_schema: String,
}

/// Metadata about a column within a table
#[derive(Debug, Clone, Serialize)]
pub struct ColumnInfo {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    pub is_primary_key: bool,
    pub max_length: Option<i32>,
}

/// Schema for a table (columns + primary key)
#[derive(Debug, Clone, Serialize)]
pub struct TableSchema {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_key_columns: Vec<String>,
}

/// Paginated response for rows
#[derive(Debug, Serialize)]
pub struct RowsResponse {
    pub rows: Vec<serde_json::Value>,
    pub total_count: i64,
    pub page: u32,
    pub per_page: u32,
}
