use sea_orm_migration::prelude::*;

mod m20240508_000001_create_tables;
mod m20240603_000002_node_v2;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240508_000001_create_tables::Migration),
            Box::new(m20240603_000002_node_v2::Migration),
        ]
    }
}
