use std::{collections::BTreeMap, error::Error};

pub mod mysql;

pub trait Driver {
    async fn list_databases(
        &self,
        ignore_default_tables: bool,
    ) -> Result<Vec<String>, Box<dyn Error>>;

    async fn list_tables(&self, database: String) -> Result<Vec<String>, Box<dyn Error>>;

    async fn list_columns(
        &self,
        database: String,
        table: String,
    ) -> Result<Vec<ColumnInfo>, Box<dyn Error>>;

    async fn list_records(
        &self,
        database: String,
        table: String,
        filter: Option<String>,
        sort: Option<String>,
    ) -> Result<BTreeMap<String, String>, Box<dyn Error>>;

    async fn list_constraints(
        &self,
        database: String,
        table: String,
    ) -> Result<Vec<ConstraintInfo>, Box<dyn Error>>;

    async fn list_foreign_keys(
        &self,
        database: String,
        table: String,
    ) -> Result<Vec<ForeignKeyInfo>, Box<dyn Error>>;

    async fn list_indexes(
        &self,
        database: String,
        table: String,
    ) -> Result<Vec<IndexInfo>, Box<dyn Error>>;
}

pub struct ColumnInfo {
    name: String,
    r#type: String,
}

pub struct ConstraintInfo;
pub struct IndexInfo;
pub struct ForeignKeyInfo;
