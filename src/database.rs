use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement,
};
use sea_orm_migration::MigratorTrait;
use tracing::info;

use crate::{migrator::Migrator, Error};

pub const MAINNET_DB_NAME: &str = "mainnet";
pub const TESTNET_DB_NAME: &str = "testnet";

async fn connect(
    db_url: &str,
    db_name: &str,
    max_connections: u32,
    sslmode: &str,
) -> Result<DatabaseConnection, Error> {
    let mut opt = ConnectOptions::new(format!("{db_url}?sslmode={sslmode}"));
    opt.max_connections(max_connections)
        .min_connections(5.min(max_connections));
    let db = Database::connect(opt).await?;

    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            unimplemented!()
        }
        DbBackend::Postgres => {
            let result = db
                .execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("SELECT 1 FROM pg_database WHERE datname = '{}'", db_name),
                ))
                .await?;
            if result.rows_affected() == 0 {
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("CREATE DATABASE {}", db_name),
                ))
                .await?;
            }
            let url = format!("{db_url}/{db_name}?sslmode={sslmode}");
            let mut opt = ConnectOptions::new(url);
            opt.max_connections(max_connections)
                .min_connections(5.min(max_connections));
            Database::connect(opt).await?
        }
        DbBackend::Sqlite => unimplemented!(),
    };

    info!("connected to database: {db_name}");

    Ok(db)
}

pub async fn connect_and_refresh_schema(
    db_url: &str,
    db_name: &str,
    max_connections: u32,
    sslmode: &str,
) -> Result<DatabaseConnection, Error> {
    let db = connect(db_url, db_name, max_connections, sslmode).await?;
    let pending = Migrator::get_pending_migrations(&db).await?.len();
    if pending == 0 {
        info!("no database migrations to apply");
    } else {
        info!("applying {pending} database migrations");
        Migrator::up(&db, None).await?;
    }
    Ok(db)
}
