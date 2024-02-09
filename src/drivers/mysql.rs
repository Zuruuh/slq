use mysql::{
    prelude::{FromRow, Queryable},
    FromValueError, Pool, Row, Value,
};
use std::{any::type_name, collections::BTreeMap, error::Error};

use super::{ColumnInfo, Driver, RowValues};

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

fn format_date_part(part: u8) -> String {
    (part > 9)
        .then(|| part.to_string())
        .unwrap_or_else(|| format!("0{part}"))
}

impl Driver for MySQLDriver {
    async fn list_databases(
        &self,
        ignore_default_tables: bool,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut conn = self.pool.get_conn()?;

        Ok(conn
            .query("show databases")?
            .into_iter()
            .filter(|table: &String| {
                !ignore_default_tables || !MYSQL_DEFAULT_TABLES.contains(&table.as_str())
            })
            .map(|table| table.to_string())
            .collect())
    }

    async fn list_tables(&self, database: String) -> Result<Vec<String>, Box<dyn Error>> {
        let mut conn = self.pool.get_conn()?;

        Ok(conn.exec(format!("show tables in {database}"), ())?)
    }

    async fn list_columns(
        &self,
        database: String,
        table: String,
    ) -> Result<Vec<ColumnInfo>, Box<dyn Error>> {
        let mut conn = self.pool.get_conn()?;

        Ok(conn.exec(format!("show columns in {database}.{table}"), ())?)
    }

    async fn list_records(
        &self,
        database: String,
        table: String,
        filter: Option<String>,
        sort: Option<String>,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<RowValues>, Box<dyn Error>> {
        let mut conn = self.pool.get_conn()?;
        let mut query = format!("select * from {database}.{table}");

        if let Some(filter) = filter {
            query += &format!(" where {filter}");
        }

        if let Some(sort) = sort {
            query += &format!(" sort by {sort}");
        }

        if offset > 0 {
            query += &format!(" offset {offset}");
        }

        query += &format!(" limit {limit}");

        let mut records: Vec<Row> = conn.exec(&query, ())?;

        match records.last() {
            None => Ok(Vec::new()),
            Some(record) => {
                let mut buffer = Vec::<RowValues>::new();
                let columns = record.columns();

                for record in records.into_iter() {
                    let mut row = BTreeMap::<String, String>::new();
                    for col in columns.into_iter() {
                        let col = col.name_str().to_string();
                        let value = record
                            .get_opt::<String, &str>(col.as_str())
                            .unwrap()
                            .unwrap_or_else(|err: FromValueError| match err.0 {
                                // TODO use option here
                                Value::NULL => "".to_string(),
                                Value::Bytes(bytes) => String::from_utf8(bytes).unwrap(),
                                Value::Int(int) => int.to_string(),
                                Value::UInt(uint) => uint.to_string(),
                                Value::Float(float) => float.to_string(),
                                Value::Double(double) => double.to_string(),
                                Value::Date(
                                    year,
                                    month,
                                    day,
                                    hour,
                                    minutes,
                                    seconds,
                                    _micro_seconds,
                                ) => format!(
                                    "{year}-{}-{} {}:{}:{}",
                                    format_date_part(month),
                                    format_date_part(day),
                                    format_date_part(hour),
                                    format_date_part(minutes),
                                    format_date_part(seconds),
                                ),

                                Value::Time(
                                    negative,
                                    days,
                                    hours,
                                    minutes,
                                    _seconds,
                                    _micro_seconds,
                                ) => {
                                    format!(
                                        "{} {days}d {hours}h {minutes}m",
                                        negative.then(|| '-').unwrap_or_default()
                                    )
                                }
                            });

                        row.insert(col, value);
                    }

                    buffer.push(RowValues::from(row))
                }

                Ok(buffer)
            }
        }
    }

    async fn list_constraints(
        &self,
        database: String,
        table: String,
    ) -> Result<Vec<super::ConstraintInfo>, Box<dyn Error>> {
        todo!()
    }

    async fn list_foreign_keys(
        &self,
        database: String,
        table: String,
    ) -> Result<Vec<super::ForeignKeyInfo>, Box<dyn Error>> {
        todo!()
    }

    async fn list_indexes(
        &self,
        database: String,
        table: String,
    ) -> Result<Vec<super::IndexInfo>, Box<dyn Error>> {
        todo!()
    }
}

impl FromRow for ColumnInfo {
    fn from_row(row: mysql::Row) -> Self
    where
        Self: Sized,
    {
        match Self::from_row_opt(row) {
            Ok(x) => x,
            Err(mysql::FromRowError(row)) => panic!(
                "Couldn't convert {:?} to type {}. (see FromRow documentation)",
                row,
                type_name::<Self>(),
            ),
        }
    }

    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            name: row.get("Field").unwrap(),
            r#type: row.get("Type").unwrap(),
            key: row
                .get("Key")
                .map(|str: String| (!str.is_empty()).then_some(str))
                .flatten(),
            nullable: row
                .get::<String, &str>("Nullable")
                .map(|str| str == "YES")
                .unwrap_or_default(),
            extra: row
                .get("Extra")
                .map(|str: String| (!str.is_empty()).then_some(str))
                .flatten(),
            default: row.get("Default").unwrap(),
        })
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::MySQLDriver;
    use crate::drivers::{ColumnInfo, Driver, RowValues};
    use mysql::{OptsBuilder, Pool};

    #[tokio::test]
    pub async fn test_database_list() {
        let admin_databases = connect("admin").list_databases(true).await;

        assert!(admin_databases.is_ok());
        assert_eq!(vec!["slq".to_string()], admin_databases.unwrap(),);

        let super_admin_databases = connect("super_admin").list_databases(true).await;

        assert!(super_admin_databases.is_ok());
        assert_eq!(
            vec!["other_database".to_string(), "slq".to_string()],
            super_admin_databases.unwrap()
        );
    }

    #[tokio::test]
    pub async fn test_table_list() {
        let tables = connect("admin").list_tables("slq".to_string()).await;

        assert!(tables.is_ok());
        assert_eq!(
            vec!["posts".to_string(), "users".to_string()],
            tables.unwrap()
        );
    }

    #[tokio::test]
    pub async fn test_column_list() {
        let fields = connect("admin")
            .list_columns("slq".to_string(), "users".to_string())
            .await;

        assert!(fields.is_ok());
        assert_eq!(
            vec![
                ColumnInfo {
                    name: "id".to_string(),
                    r#type: "int".to_string(),
                    key: Some("PRI".to_string()),
                    extra: Some("auto_increment".to_string()),
                    ..Default::default()
                },
                ColumnInfo {
                    name: "username".to_string(),
                    r#type: "varchar(32)".to_string(),
                    key: Some("UNI".to_string()),
                    ..Default::default()
                },
                ColumnInfo {
                    name: "email".to_string(),
                    r#type: "varchar(255)".to_string(),
                    key: Some("UNI".to_string()),
                    ..Default::default()
                },
                ColumnInfo {
                    name: "password".to_string(),
                    r#type: "varchar(255)".to_string(),
                    ..Default::default()
                },
                ColumnInfo {
                    name: "registered_at".to_string(),
                    r#type: "timestamp".to_string(),
                    default: Some("CURRENT_TIMESTAMP".to_string()),
                    extra: Some("DEFAULT_GENERATED".to_string()),
                    ..Default::default()
                }
            ],
            fields.unwrap()
        );
    }

    #[tokio::test]
    pub async fn test_record_list() {
        let records = connect("admin")
            .list_records("slq".to_string(), "users".to_string(), None, None, 0, 50)
            .await;

        assert!(records.is_ok());
        assert_eq!(
            vec![
                RowValues::from(BTreeMap::from([
                    ("email".to_string(), "john@example.com".to_string()),
                    ("username".to_string(), "john_doe".to_string()),
                    ("id".to_string(), "1".to_string()),
                    ("password".to_string(), "password123".to_string()),
                    (
                        "registered_at".to_string(),
                        "2024-02-06 16:38:26".to_string()
                    ),
                ])),
                RowValues::from(BTreeMap::from([
                    ("email".to_string(), "jane@example.com".to_string()),
                    ("username".to_string(), "jane_smith".to_string()),
                    ("id".to_string(), "2".to_string()),
                    ("password".to_string(), "secret456".to_string()),
                    (
                        "registered_at".to_string(),
                        "2024-02-06 16:38:26".to_string()
                    ),
                ])),
                RowValues::from(BTreeMap::from([
                    ("email".to_string(), "mike@example.com".to_string()),
                    ("username".to_string(), "mike_jackson".to_string()),
                    ("id".to_string(), "3".to_string()),
                    ("password".to_string(), "mysecurepassword".to_string()),
                    (
                        "registered_at".to_string(),
                        "2024-02-06 16:38:26".to_string()
                    ),
                ]))
            ],
            records.unwrap()
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
