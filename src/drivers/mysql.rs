use mysql::{prelude::Queryable, Pool};
use std::{collections::BTreeMap, error::Error};

use super::{ColumnInfo, Driver};

const MYSQL_DEFAULT_TABLES: [&'static str; 4] =
    ["mysql", "information_schema", "performance_schema", "sys"];

pub struct MySQLDriver {
    pool: Pool,
}

impl From<Pool> for MySQLDriver {
    fn from(value: Pool) -> Self {
        Self { pool: value }
    }
}

impl Driver for MySQLDriver {
    async fn list_databases(
        &self,
        ignore_default_tables: bool,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut conn = self.pool.get_conn()?;

        Ok(conn
            .query("SHOW DATABASES")?
            .into_iter()
            .filter(|table: &String| {
                !ignore_default_tables || !MYSQL_DEFAULT_TABLES.contains(&table.as_str())
            })
            .map(|table| table.to_string())
            .collect())
    }

    async fn list_tables(&self, database: String) -> Result<Vec<String>, Box<dyn Error>> {
        let mut conn = self.pool.get_conn()?;

        Ok(conn.exec(format!("SHOW TABLES IN {database}"), ())?)
    }

    async fn list_columns(
        &self,
        database: String,
        table: String,
    ) -> Result<Vec<ColumnInfo>, Box<dyn Error>> {
        todo!()
    }

    async fn list_records(
        &self,
        database: String,
        table: String,
        filter: Option<String>,
        sort: Option<String>,
    ) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::MySQLDriver;
    use crate::drivers::Driver;
    use mysql::{OptsBuilder, Pool};

    #[tokio::test]
    pub async fn test_database_list() {
        assert_eq!(
            vec!["slq".to_string()],
            connect("admin").list_databases(true).await.unwrap()
        );

        assert_eq!(
            vec!["other_database".to_string(), "slq".to_string()],
            connect("super_admin").list_databases(true).await.unwrap()
        );
    }

    #[tokio::test]
    pub async fn test_table_list() {
        let tables = connect("admin")
            .list_tables("slq".to_string())
            .await
            .unwrap();

        assert_eq!(
            vec!["slq_data".to_string(), "slq_other_data".to_string()],
            tables
        );
    }

    fn connect(user: &str) -> MySQLDriver {
        let pool = Pool::new(
            OptsBuilder::new()
                .user(Some(user))
                .pass(Some("passwd"))
                .ip_or_hostname(Some("localhost"))
                .tcp_port(3300)
                .db_name(Some("slq")),
        )
        .unwrap();
        MySQLDriver::from(pool)
    }
}
