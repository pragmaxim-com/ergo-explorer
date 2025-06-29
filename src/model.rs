use chain_syncer::api::{BlockHeaderLike, BlockLike, ChainSyncError};
use std::error::Error;
use chrono::DateTime;
pub use redbit::*;
use std::fmt;

#[column]
pub struct AssetName(pub Vec<u8>);
#[column]
pub struct AssetAction(pub u8);
#[column]
pub struct Tree(pub Vec<u8>);
#[column]
pub struct TreeT8(pub Vec<u8>);
#[column]
pub struct BoxId(pub Vec<u8>);

#[root_key]
pub struct BlockHeight(pub u32);

#[pointer_key(u16)]
pub struct TxPointer(BlockHeight);
#[pointer_key(u16)]
pub struct UtxoPointer(TxPointer);
#[pointer_key(u16)]
pub struct InputPointer(TxPointer);
#[pointer_key(u8)]
pub struct AssetPointer(UtxoPointer);

#[column]
pub struct Hash(pub String);
#[column]
pub struct BlockHash(pub [u8; 32]);
#[column]
pub struct TxHash(pub [u8; 32]);
#[column]
pub struct Address(pub Vec<u8>);

#[column]
#[derive(Copy, Hash)]
pub struct BlockTimestamp(pub u32);
impl fmt::Display for BlockTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let datetime = DateTime::from_timestamp(self.0 as i64, 0).unwrap();
        let readable_date = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        write!(f, "{}", readable_date)
    }
}

#[entity]
pub struct Block {
    #[pk]
    pub id: BlockHeight,
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    #[column(transient)]
    pub weight: u32,
}

#[entity]
pub struct BlockHeader {
    #[fk(one2one)]
    pub id: BlockHeight,
    #[column(index)]
    pub hash: BlockHash,
    #[column(index)]
    pub prev_hash: BlockHash,
    #[column(range)]
    pub timestamp: BlockTimestamp,
}

#[entity]
pub struct Transaction {
    #[fk(one2many)]
    pub id: TxPointer,
    #[column(index)]
    pub hash: TxHash,
    pub utxos: Vec<Utxo>,
    pub inputs: Vec<InputRef>,
    #[column(transient)]
    pub transient_inputs: Vec<BoxId>,
}

#[entity]
pub struct Utxo {
    #[fk(one2many)]
    pub id: UtxoPointer,
    #[column]
    pub amount: u64,
    #[column(index)]
    pub box_id: BoxId,
    #[column(dictionary)]
    pub address: Address,
    #[column(dictionary)]
    pub tree: Tree,
    #[column(dictionary)]
    pub tree_t8: TreeT8,
    pub assets: Vec<Asset>,
}

#[entity]
pub struct Asset {
    #[fk(one2many, range)]
    pub id: AssetPointer,
    #[column]
    pub amount: u64,
    #[column(index, dictionary)]
    pub name: AssetName,
    #[column(index)]
    pub asset_action: AssetAction,
}

#[entity]
pub struct InputRef {
    #[fk(one2many)]
    pub id: InputPointer,
}

impl BlockHeaderLike for BlockHeader {
    fn height(&self) -> u32 {
        self.id.0
    }
    fn hash(&self) -> [u8; 32] {
        self.hash.0
    }
    fn prev_hash(&self) -> [u8; 32] {
        self.prev_hash.0
    }
    fn timestamp(&self) -> u32 {
        self.timestamp.0
    }
}

impl BlockLike for Block {
    type Header = BlockHeader;

    fn header(&self) -> &Self::Header {
        &self.header
    }

    fn weight(&self) -> u32 {
        self.weight
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExplorerError {
    #[error("Reqwest error: {source}{}", source.source().map(|e| format!(": {}", e)).unwrap_or_default())]
    Reqwest {
        #[from]
        source: reqwest::Error,
    },

    #[error("Url parsing error: {0}")]
    Url(#[from] url::ParseError),

    #[error("Custom error: {0}")]
    Custom(String),
}

impl From<ExplorerError> for ChainSyncError {
    fn from(err: ExplorerError) -> Self {
        ChainSyncError::new(&err.to_string())
    }
}
