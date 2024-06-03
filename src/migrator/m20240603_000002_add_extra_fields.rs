use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240603_000002_add_extra_fields"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(ColumnDef::new(Node::ChainId).string().null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Node::Table)
                    .add_column(ColumnDef::new(Node::ProtocolVersion).integer().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(Table::alter().drop_column(Node::ChainId).to_owned())
            .await?;
        manager
            .alter_table(Table::alter().drop_column(Node::ProtocolVersion).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
pub enum Node {
    Table,
    ChainId,
    ProtocolVersion,
}
