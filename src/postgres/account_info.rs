use sea_orm::entity::prelude::*;

use crate::Base58PublicKey;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "account_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub account_id: i32,
    #[sea_orm(unique)]
    pub key: Base58PublicKey,
    pub is_signer: bool,
    pub is_writable: bool,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Base58PublicKey,
    pub executable: bool,
    pub rent_epoch: u64,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!("No RelationDef")
    }
}

impl ActiveModelBehavior for ActiveModel {}
