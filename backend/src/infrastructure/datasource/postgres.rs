use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::time::Duration;

use crate::domain::data::{ColumnInfo, RowsResponse, TableInfo, TableSchema};
use crate::infrastructure::datasource::DataSource;
use crate::presentation::request::RowsQuery;

pub struct PostgresDataSource {
    pool: PgPool,
}

impl PostgresDataSource {
    pub async fn new(connection_string: &str) -> anyhow::Result<Self> {
        // Mask password in logs
        let safe_conn = connection_string.split('@').next_back().unwrap_or("***");
        tracing::info!(target = %safe_conn, "Creating PostgreSQL connection pool...");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .connect(connection_string)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    target = %safe_conn,
                    "Failed to create connection pool. Check that the database is running and credentials are correct."
                );
                e
            })?;

        // Verify the connection actually works
        tracing::debug!("Verifying connection with a test query...");
        sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Connection verification query failed");
                e
            })?;

        tracing::info!(
            target = %safe_conn,
            max_connections = 5,
            "PostgreSQL connection pool created and verified"
        );
        Ok(Self { pool })
    }

    /// Resolve the primary key column(s) for a given table
    async fn get_primary_key_columns(&self, table_name: &str) -> anyhow::Result<Vec<String>> {
        tracing::debug!(table = %table_name, "Resolving primary key columns");
        let rows = sqlx::query(
            r#"
            SELECT kcu.column_name
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
              ON tc.constraint_name = kcu.constraint_name
              AND tc.table_schema = kcu.table_schema
            WHERE tc.constraint_type = 'PRIMARY KEY'
              AND tc.table_name = $1
              AND tc.table_schema = 'public'
            ORDER BY kcu.ordinal_position
            "#,
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await?;

        let columns: Vec<String> = rows
            .iter()
            .map(|r| r.get::<String, _>("column_name"))
            .collect();
        tracing::debug!(table = %table_name, pk_columns = ?columns, "Primary key columns resolved");
        Ok(columns)
    }

    /// Build a safe identifier (prevents SQL injection for table/column names)
    fn quote_ident(name: &str) -> String {
        // Double-quote and escape any existing double quotes
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

#[async_trait]
impl DataSource for PostgresDataSource {
    async fn list_tables(&self) -> anyhow::Result<Vec<TableInfo>> {
        tracing::info!("Listing tables from information_schema");
        let rows = sqlx::query(
            r#"
            SELECT table_name, table_schema
            FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_type = 'BASE TABLE'
            ORDER BY table_name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to list tables");
            e
        })?;

        let tables: Vec<TableInfo> = rows
            .iter()
            .map(|r| TableInfo {
                table_name: r.get("table_name"),
                table_schema: r.get("table_schema"),
            })
            .collect();

        tracing::info!(count = tables.len(), "Tables found");
        for t in &tables {
            tracing::debug!(table = %t.table_name, schema = %t.table_schema);
        }
        Ok(tables)
    }

    async fn get_table_schema(&self, table_name: &str) -> anyhow::Result<TableSchema> {
        tracing::info!(table = %table_name, "Getting table schema");
        let pk_columns = self.get_primary_key_columns(table_name).await?;

        let rows = sqlx::query(
            r#"
            SELECT
                c.column_name,
                c.data_type,
                c.is_nullable,
                c.column_default,
                c.character_maximum_length
            FROM information_schema.columns c
            WHERE c.table_name = $1
              AND c.table_schema = 'public'
            ORDER BY c.ordinal_position
            "#,
        )
        .bind(table_name)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(table = %table_name, error = %e, "Failed to get table schema");
            e
        })?;

        let columns: Vec<ColumnInfo> = rows
            .iter()
            .map(|r| {
                let col_name: String = r.get("column_name");
                ColumnInfo {
                    is_primary_key: pk_columns.contains(&col_name),
                    column_name: col_name,
                    data_type: r.get("data_type"),
                    is_nullable: r.get::<String, _>("is_nullable") == "YES",
                    column_default: r.get("column_default"),
                    max_length: r.get::<Option<i32>, _>("character_maximum_length"),
                }
            })
            .collect();

        tracing::info!(table = %table_name, column_count = columns.len(), pk = ?pk_columns, "Schema retrieved");
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
            "Listing rows"
        );

        let table = Self::quote_ident(table_name);

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
                    "like" => "ILIKE",
                    _ => "=",
                };
                if parts[1] == "like" {
                    filter_values.push(format!("%{}%", parts[2]));
                } else {
                    filter_values.push(parts[2].to_string());
                }
                where_clause = format!(" WHERE {}::text {} $1", col, op);
                tracing::debug!(
                    column = parts[0],
                    operator = op,
                    value = parts[2],
                    "Filter applied"
                );
            }
        }

        // Count query
        let count_sql = format!("SELECT COUNT(*) as cnt FROM {}{}", table, where_clause);
        tracing::debug!(sql = %count_sql, "Executing count query");
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

        // Build data query
        let data_sql = format!(
            "SELECT row_to_json(t.*) as row_data FROM {} AS t{}{} LIMIT {} OFFSET {}",
            table, where_clause, order_clause, per_page, offset
        );
        tracing::debug!(sql = %data_sql, "Executing data query");

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
            .map(|r| r.get::<serde_json::Value, _>("row_data"))
            .collect();

        tracing::info!(
            table = %table_name,
            total_count = total_count,
            returned = json_rows.len(),
            page = page,
            "Rows retrieved"
        );

        Ok(RowsResponse {
            rows: json_rows,
            total_count,
            page,
            per_page,
        })
    }

    async fn get_row(&self, table_name: &str, pk_value: &str) -> anyhow::Result<serde_json::Value> {
        tracing::info!(table = %table_name, pk = %pk_value, "Getting single row");
        let pk_columns = self.get_primary_key_columns(table_name).await?;
        let pk_col = pk_columns
            .first()
            .ok_or_else(|| anyhow::anyhow!("No primary key found for table {}", table_name))?;

        let table = Self::quote_ident(table_name);
        let sql = format!(
            "SELECT row_to_json(t.*) as row_data FROM {} AS t WHERE {}::text = $1",
            table,
            Self::quote_ident(pk_col)
        );

        let row = sqlx::query(&sql)
            .bind(pk_value)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(table = %table_name, pk = %pk_value, error = %e, "Failed to get row");
                e
            })?;

        tracing::debug!(table = %table_name, pk = %pk_value, "Row retrieved");
        Ok(row.get::<serde_json::Value, _>("row_data"))
    }

    async fn insert_row(
        &self,
        table_name: &str,
        data: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        tracing::info!(table = %table_name, "Inserting new row");
        tracing::debug!(table = %table_name, data = %data, "Insert data");

        let obj = data
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Data must be a JSON object"))?;

        let table = Self::quote_ident(table_name);
        let mut columns = Vec::new();
        let mut placeholders = Vec::new();
        let mut values: Vec<String> = Vec::new();
        let mut idx = 1;

        for (key, val) in obj.iter() {
            if val.is_null() {
                continue;
            }
            columns.push(Self::quote_ident(key));
            placeholders.push(format!("${}", idx));
            values.push(match val {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            });
            idx += 1;
        }

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING row_to_json({}.*)  as row_data",
            table,
            columns.join(", "),
            placeholders.join(", "),
            table
        );
        tracing::debug!(sql = %sql, "Executing insert");

        let mut query = sqlx::query(&sql);
        for v in &values {
            query = query.bind(v);
        }

        let row = query.fetch_one(&self.pool).await.map_err(|e| {
            tracing::error!(table = %table_name, error = %e, "Failed to insert row");
            e
        })?;

        tracing::info!(table = %table_name, "Row inserted successfully");
        Ok(row.get::<serde_json::Value, _>("row_data"))
    }

    async fn update_row(
        &self,
        table_name: &str,
        pk_value: &str,
        data: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        tracing::info!(table = %table_name, pk = %pk_value, "Updating row");
        tracing::debug!(table = %table_name, pk = %pk_value, data = %data, "Update data");

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
        let mut idx = 1;

        for (key, val) in obj.iter() {
            if key == pk_col {
                continue;
            }
            set_clauses.push(format!("{} = ${}", Self::quote_ident(key), idx));
            values.push(match val {
                serde_json::Value::Null => {
                    set_clauses.pop();
                    set_clauses.push(format!("{} = NULL", Self::quote_ident(key)));
                    continue;
                }
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            });
            idx += 1;
        }

        let pk_placeholder = format!("${}", idx);
        values.push(pk_value.to_string());

        let sql = format!(
            "UPDATE {} SET {} WHERE {}::text = {} RETURNING row_to_json({}.*)  as row_data",
            table,
            set_clauses.join(", "),
            Self::quote_ident(pk_col),
            pk_placeholder,
            table
        );
        tracing::debug!(sql = %sql, "Executing update");

        let mut query = sqlx::query(&sql);
        for v in &values {
            query = query.bind(v);
        }

        let row = query.fetch_one(&self.pool).await.map_err(|e| {
            tracing::error!(table = %table_name, pk = %pk_value, error = %e, "Failed to update row");
            e
        })?;

        tracing::info!(table = %table_name, pk = %pk_value, "Row updated successfully");
        Ok(row.get::<serde_json::Value, _>("row_data"))
    }

    async fn delete_row(&self, table_name: &str, pk_value: &str) -> anyhow::Result<()> {
        tracing::info!(table = %table_name, pk = %pk_value, "Deleting row");
        let pk_columns = self.get_primary_key_columns(table_name).await?;
        let pk_col = pk_columns
            .first()
            .ok_or_else(|| anyhow::anyhow!("No primary key found for table {}", table_name))?;

        let table = Self::quote_ident(table_name);
        let sql = format!(
            "DELETE FROM {} WHERE {}::text = $1",
            table,
            Self::quote_ident(pk_col)
        );
        tracing::debug!(sql = %sql, pk = %pk_value, "Executing delete");

        sqlx::query(&sql)
            .bind(pk_value)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(table = %table_name, pk = %pk_value, error = %e, "Failed to delete row");
                e
            })?;

        tracing::info!(table = %table_name, pk = %pk_value, "Row deleted successfully");
        Ok(())
    }
}
