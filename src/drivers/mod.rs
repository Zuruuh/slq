use std::{collections::BTreeMap, error::Error, fmt::Debug};

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
        offset: usize,
        limit: usize,
    ) -> Result<Vec<RowValues>, Box<dyn Error>>;

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

#[derive(Default, PartialEq, Eq)]
pub struct RowValues(BTreeMap<String, String>);

impl From<BTreeMap<String, String>> for RowValues {
    fn from(value: BTreeMap<String, String>) -> Self {
        Self(value)
    }
}

impl std::fmt::Debug for RowValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct ColumnInfo {
    pub name: String,
    pub r#type: String,
    pub nullable: bool,
    pub key: Option<String>,
    pub default: Option<String>,
    pub extra: Option<String>,
}

pub struct ConstraintInfo;
pub struct IndexInfo;
pub struct ForeignKeyInfo;
