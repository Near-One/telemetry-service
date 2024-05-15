use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240508_000001_create_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Node::Table)
                    .col(ColumnDef::new(Node::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Node::AccountId).string().null())
                    .col(ColumnDef::new(Node::LastSeen).timestamp().not_null())
                    .col(ColumnDef::new(Node::LastHeight).big_integer().not_null())
                    .col(ColumnDef::new(Node::LastHash).string().not_null())
                    .col(ColumnDef::new(Node::AgentName).string().not_null())
                    .col(ColumnDef::new(Node::AgentVersion).string().not_null())
                    .col(ColumnDef::new(Node::AgentBuild).string().not_null())
                    .col(ColumnDef::new(Node::PeerCount).big_integer().not_null())
                    .col(ColumnDef::new(Node::IsValidator).boolean().not_null())
                    .col(ColumnDef::new(Node::Status).string().not_null())
                    .col(
                        ColumnDef::new(Node::BandwidthDownload)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Node::BandwidthUpload)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Node::CpuUsage).float().not_null())
                    .col(ColumnDef::new(Node::MemoryUsage).big_integer().not_null())
                    .col(
                        ColumnDef::new(Node::BootTimeSeconds)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Node::BlockProductionTrackingDelay)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Node::MinBlockProductionDelay)
                            .double()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Node::MaxBlockProductionDelay)
                            .double()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Node::MaxBlockWaitDelay).double().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        // It should be a "drop table", but we don't want to perform such operation automatically.
        Ok(())
    }
}

#[derive(Iden)]
pub enum Node {
    Table,
    Id,
    AccountId,
    LastSeen,
    LastHeight,
    LastHash,
    AgentName,
    AgentVersion,
    AgentBuild,
    PeerCount,
    IsValidator,
    Status,
    BandwidthDownload,
    BandwidthUpload,
    CpuUsage,
    MemoryUsage,
    BootTimeSeconds,
    BlockProductionTrackingDelay,
    MinBlockProductionDelay,
    MaxBlockProductionDelay,
    MaxBlockWaitDelay,
}
