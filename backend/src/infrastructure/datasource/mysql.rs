use async_trait::async_trait;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySqlPool, Row};
use std::time::Duration;

use crate::domain::data::{ColumnInfo, RowsResponse, TableInfo, TableSchema};
use crate::infrastructure::datasource::DataSource;
use crate::presentation::request::RowsQuery;

pub struct MySqlDataSource {
    pool: MySqlPool,
}

/// Helper: decode a column that MySQL may return as VARBINARY, VARCHAR, or bytes.
/// Tries String first, falls back to Vec<u8> → String conversion.
fn get_string(row: &sqlx::mysql::MySqlRow, col: &str) -> String {
    row.try_get::<String, _>(col).unwrap_or_else(|_| {
        let bytes: Vec<u8> = row.try_get(col).unwrap_or_default();
        String::from_utf8(bytes).unwrap_or_default()
    })
}

/// Helper: decode an optional string column.
fn get_opt_string(row: &sqlx::mysql::MySqlRow, col: &str) -> Option<String> {
    match row.try_get::<Option<String>, _>(col) {
        Ok(v) => v,
        Err(_) => {
            let bytes: Option<Vec<u8>> = row.try_get(col).unwrap_or(None);
            bytes.map(|b| String::from_utf8(b).unwrap_or_default())
        }
    }
}

impl MySqlDataSource {
    pub async fn new(connection_string: &str) -> anyhow::Result<Self> {
        let safe_conn = connection_string.split('@').next_back().unwrap_or("***");
        tracing::info!(target = %safe_conn, "Creating MySQL connection pool...");

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .connect(connection_string)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    target = %safe_conn,
                    "Failed to create MySQL connection pool."
                );
                e
            })?;

        tracing::debug!("Verifying MySQL connection with a test query...");
        sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "MySQL connection verification query failed");
                e
            })?;

        tracing::info!(
            target = %safe_conn,
            max_connections = 5,
            "MySQL connection pool created and verified"
        );
        Ok(Self { pool })
    }

    /// Resolve the primary key column(s) for a given table
    async fn get_primary_key_columns(&self, table_name: &str) -> anyhow::Result<Vec<String>> {
        tracing::debug!(table = %table_name, "Resolving primary key columns (MySQL)");
        let rows = sqlx::query(
            r#"
            SELECT COLUMN_NAME
            FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE
            WHERE TABLE_NAME = ?
              AND TABLE_SCHEMA = DATABASE()
              AND CONSTRAINT_NAME = 'PRIMARY'
            ORDER BY ORDINAL_POSITION
            "#,
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let columns: Vec<String> = rows.iter().map(|r| get_string(r, "COLUMN_NAME")).collect();
        tracing::debug!(table = %table_name, pk_columns = ?columns, "Primary key columns resolved");
        Ok(columns)
    }

    /// Build a safe identifier (prevents SQL injection for table/column names)
    fn quote_ident(name: &str) -> String {
        format!("`{}`", name.replace('`', "``"))
    }
}

#[async_trait]
impl DataSource for MySqlDataSource {
    async fn list_tables(&self) -> anyhow::Result<Vec<TableInfo>> {
        tracing::info!("Listing tables from INFORMATION_SCHEMA (MySQL)");
        let rows = sqlx::query(
            r#"
            SELECT TABLE_NAME, TABLE_SCHEMA
            FROM INFORMATION_SCHEMA.TABLES
            WHERE TABLE_SCHEMA = DATABASE()
              AND TABLE_TYPE = 'BASE TABLE'
            ORDER BY TABLE_NAME
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to list tables (MySQL)");
            e
        })?;

        let tables: Vec<TableInfo> = rows
            .iter()
            .map(|r| TableInfo {
                table_name: get_string(r, "TABLE_NAME"),
                table_schema: get_string(r, "TABLE_SCHEMA"),
            })
            .collect();

        tracing::info!(count = tables.len(), "Tables found (MySQL)");
        Ok(tables)
    }

    async fn get_table_schema(&self, table_name: &str) -> anyhow::Result<TableSchema> {
        tracing::info!(table = %table_name, "Getting table schema (MySQL)");
        let pk_columns = self.get_primary_key_columns(table_name).await?;

        let rows = sqlx::query(
            r#"
            SELECT
                COLUMN_NAME,
                DATA_TYPE,
                IS_NULLABLE,
                COLUMN_DEFAULT,
                CHARACTER_MAXIMUM_LENGTH
            FROM INFORMATION_SCHEMA.COLUMNS
            WHERE TABLE_NAME = ?
              AND TABLE_SCHEMA = DATABASE()
            ORDER BY ORDINAL_POSITION
            "#,
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(table = %table_name, error = %e, "Failed to get table schema (MySQL)");
            e
        })?;

        let columns: Vec<ColumnInfo> = rows
            .iter()
            .map(|r| {
                let col_name = get_string(r, "COLUMN_NAME");
                ColumnInfo {
                    is_primary_key: pk_columns.contains(&col_name),
                    column_name: col_name,
                    data_type: get_string(r, "DATA_TYPE"),
                    is_nullable: get_string(r, "IS_NULLABLE") == "YES",
                    column_default: get_opt_string(r, "COLUMN_DEFAULT"),
                    max_length: r
                        .try_get::<Option<i64>, _>("CHARACTER_MAXIMUM_LENGTH")
                        .unwrap_or(None)
                        .map(|v| v as i32),
                }
            })
            .collect();

        tracing::info!(table = %table_name, column_count = columns.len(), pk = ?pk_columns, "Schema retrieved (MySQL)");
        Ok(TableSchema {
            table_name: table_name.to_string(),
            columns,
            primary_key_columns: pk_columns,
        })
    }

    async fn list_rows(&self, table_name: &str, query: &RowsQuery) -> anyhow::Result<RowsResponse> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        tracing::info!(
            table = %table_name,
            page = page,
            per_page = per_page,
            sort_by = ?query.sort_by,
            sort_order = ?query.sort_order,
            filter = ?query.filter,
            "Listing rows (MySQL)"
        );

        let table = Self::quote_ident(table_name);

        // Get all columns for JSON construction
        let schema = self.get_table_schema(table_name).await?;
        let json_cols: String = schema
            .columns
            .iter()
            .map(|c| format!("'{}', {}", c.column_name, Self::quote_ident(&c.column_name)))
            .collect::<Vec<_>>()
            .join(", ");

        // Build WHERE clause from filter
        let mut where_clause = String::new();
        let mut filter_values: Vec<String> = Vec::new();
        if let Some(ref filter_str) = query.filter {
            let parts: Vec<&str> = filter_str.splitn(3, ':').collect();
            if parts.len() == 3 {
                let col = Self::quote_ident(parts[0]);
                let op = match parts[1] {
                    "eq" => "=",
                    "neq" => "!=",
                    "gt" => ">",
                    "gte" => ">=",
                    "lt" => "<",
                    "lte" => "<=",
                    "like" => "LIKE",
                    _ => "=",
                };
                if parts[1] == "like" {
                    filter_values.push(format!("%{}%", parts[2]));
                } else {
                    filter_values.push(parts[2].to_string());
                }
                where_clause = format!(" WHERE CAST({} AS CHAR) {} ?", col, op);
            }
        }

        // Count query
        let count_sql = format!("SELECT COUNT(*) as cnt FROM {}{}", table, where_clause);
        let total_count: i64 = if filter_values.is_empty() {
            sqlx::query(&count_sql)
                .fetch_one(&self.pool)
                .await?
                .get("cnt")
        } else {
            sqlx::query(&count_sql)
                .bind(&filter_values[0])
                .fetch_one(&self.pool)
                .await?
                .get("cnt")
        };

        // Build ORDER BY
        let order_clause = if let Some(ref sort_by) = query.sort_by {
            let direction = match query.sort_order.as_deref() {
                Some("desc") | Some("DESC") => "DESC",
                _ => "ASC",
            };
            format!(" ORDER BY {} {}", Self::quote_ident(sort_by), direction)
        } else {
            String::new()
        };

        // Build data query — CONCAT forces MySQL to return VARCHAR instead of JSON type
        let data_sql = format!(
            "SELECT CONCAT(JSON_OBJECT({})) as row_data FROM {}{}{} LIMIT {} OFFSET {}",
            json_cols, table, where_clause, order_clause, per_page, offset
        );
        tracing::debug!(sql = %data_sql, "Executing data query (MySQL)");

        let rows = if filter_values.is_empty() {
            sqlx::query(&data_sql).fetch_all(&self.pool).await?
        } else {
            sqlx::query(&data_sql)
                .bind(&filter_values[0])
                .fetch_all(&self.pool)
                .await?
        };

        let json_rows: Vec<serde_json::Value> = rows
            .iter()
            .map(|r| {
                let raw = get_string(r, "row_data");
                serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null)
            })
            .collect();

        tracing::info!(
            table = %table_name,
            total_count = total_count,
            returned = json_rows.len(),
            page = page,
            "Rows retrieved (MySQL)"
        );

        Ok(RowsResponse {
            rows: json_rows,
            total_count,
            page,
            per_page,
        })
    }

    async fn get_row(&self, table_name: &str, pk_value: &str) -> anyhow::Result<serde_json::Value> {
        tracing::info!(table = %table_name, pk = %pk_value, "Getting single row (MySQL)");
        let pk_columns = self.get_primary_key_columns(table_name).await?;
        let pk_col = pk_columns
            .first()
            .ok_or_else(|| anyhow::anyhow!("No primary key found for table {}", table_name))?;

        let schema = self.get_table_schema(table_name).await?;
        let json_cols: String = schema
            .columns
            .iter()
            .map(|c| format!("'{}', {}", c.column_name, Self::quote_ident(&c.column_name)))
            .collect::<Vec<_>>()
            .join(", ");

        let table = Self::quote_ident(table_name);
        let sql = format!(
            "SELECT CONCAT(JSON_OBJECT({})) as row_data FROM {} WHERE CAST({} AS CHAR) = ?",
            json_cols,
            table,
            Self::quote_ident(pk_col)
        );

        let row = sqlx::query(&sql)
            .bind(pk_value)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(table = %table_name, pk = %pk_value, error = %e, "Failed to get row (MySQL)");
                e
            })?;

        let raw = get_string(&row, "row_data");
        let json: serde_json::Value = serde_json::from_str(&raw)?;
        tracing::debug!(table = %table_name, pk = %pk_value, "Row retrieved (MySQL)");
        Ok(json)
    }

    async fn insert_row(
        &self,
        table_name: &str,
        data: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        tracing::info!(table = %table_name, "Inserting new row (MySQL)");

        let obj = data
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Data must be a JSON object"))?;

        let table = Self::quote_ident(table_name);
        let mut columns = Vec::new();
        let mut placeholders = Vec::new();
        let mut values: Vec<String> = Vec::new();

        for (key, val) in obj.iter() {
            if val.is_null() {
                continue;
            }
            columns.push(Self::quote_ident(key));
            placeholders.push("?".to_string());
            values.push(match val {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            });
        }

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table,
            columns.join(", "),
            placeholders.join(", "),
        );
        tracing::debug!(sql = %sql, "Executing insert (MySQL)");

        let mut query = sqlx::query(&sql);
        for v in &values {
            query = query.bind(v);
        }

        query.execute(&self.pool).await.map_err(|e| {
            tracing::error!(table = %table_name, error = %e, "Failed to insert row (MySQL)");
            e
        })?;

        // Fetch the inserted row using LAST_INSERT_ID if available
        let pk_columns = self.get_primary_key_columns(table_name).await?;
        if let Some(pk_col) = pk_columns.first() {
            // Check if the PK was provided in the input
            if let Some(pk_val) = obj.get(pk_col) {
                let pk_str = match pk_val {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                return self.get_row(table_name, &pk_str).await;
            }
            // Otherwise use LAST_INSERT_ID
            let last_id: u64 = sqlx::query("SELECT LAST_INSERT_ID() as id")
                .fetch_one(&self.pool)
                .await?
                .get("id");
            return self.get_row(table_name, &last_id.to_string()).await;
        }

        tracing::info!(table = %table_name, "Row inserted successfully (MySQL)");
        Ok(serde_json::json!({}))
    }

    async fn update_row(
        &self,
        table_name: &str,
        pk_value: &str,
        data: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        tracing::info!(table = %table_name, pk = %pk_value, "Updating row (MySQL)");

        let pk_columns = self.get_primary_key_columns(table_name).await?;
        let pk_col = pk_columns
            .first()
            .ok_or_else(|| anyhow::anyhow!("No primary key found for table {}", table_name))?;

        let obj = data
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Data must be a JSON object"))?;

        let table = Self::quote_ident(table_name);
        let mut set_clauses = Vec::new();
        let mut values: Vec<String> = Vec::new();

        for (key, val) in obj.iter() {
            if key == pk_col {
                continue;
            }
            match val {
                serde_json::Value::Null => {
                    set_clauses.push(format!("{} = NULL", Self::quote_ident(key)));
                }
                serde_json::Value::String(s) => {
                    set_clauses.push(format!("{} = ?", Self::quote_ident(key)));
                    values.push(s.clone());
                }
                other => {
                    set_clauses.push(format!("{} = ?", Self::quote_ident(key)));
                    values.push(other.to_string());
                }
            }
        }

        values.push(pk_value.to_string());

        let sql = format!(
            "UPDATE {} SET {} WHERE CAST({} AS CHAR) = ?",
            table,
            set_clauses.join(", "),
            Self::quote_ident(pk_col),
        );
        tracing::debug!(sql = %sql, "Executing update (MySQL)");

        let mut query = sqlx::query(&sql);
        for v in &values {
            query = query.bind(v);
        }

        query.execute(&self.pool).await.map_err(|e| {
            tracing::error!(table = %table_name, pk = %pk_value, error = %e, "Failed to update row (MySQL)");
            e
        })?;

        tracing::info!(table = %table_name, pk = %pk_value, "Row updated successfully (MySQL)");
        self.get_row(table_name, pk_value).await
    }

    async fn delete_row(&self, table_name: &str, pk_value: &str) -> anyhow::Result<()> {
        tracing::info!(table = %table_name, pk = %pk_value, "Deleting row (MySQL)");
        let pk_columns = self.get_primary_key_columns(table_name).await?;
        let pk_col = pk_columns
            .first()
            .ok_or_else(|| anyhow::anyhow!("No primary key found for table {}", table_name))?;

        let table = Self::quote_ident(table_name);
        let sql = format!(
            "DELETE FROM {} WHERE CAST({} AS CHAR) = ?",
            table,
            Self::quote_ident(pk_col)
        );
        tracing::debug!(sql = %sql, pk = %pk_value, "Executing delete (MySQL)");

        sqlx::query(&sql)
            .bind(pk_value)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(table = %table_name, pk = %pk_value, error = %e, "Failed to delete row (MySQL)");
                e
            })?;

        tracing::info!(table = %table_name, pk = %pk_value, "Row deleted successfully (MySQL)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_ident_simple() {
        assert_eq!(MySqlDataSource::quote_ident("users"), "`users`");
    }

    #[test]
    fn quote_ident_with_backtick() {
        assert_eq!(MySqlDataSource::quote_ident("my`table"), "`my``table`");
    }

    #[test]
    fn quote_ident_empty() {
        assert_eq!(MySqlDataSource::quote_ident(""), "``");
    }

    #[test]
    fn quote_ident_special_chars() {
        assert_eq!(
            MySqlDataSource::quote_ident("table-name.with spaces"),
            "`table-name.with spaces`"
        );
    }
}
