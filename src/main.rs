mod block_persistence;
mod block_provider;
mod config;
mod ergo_client;
mod model;
mod storage;

use crate::block_persistence::ErgoBlockPersistence;
use crate::block_provider::ErgoBlockProvider;
use crate::config::ErgoConfig;
use crate::model::Block;
use anyhow::Result;
use chain_syncer::api::{BlockPersistence, BlockProvider};
use chain_syncer::scheduler::Scheduler;
use chain_syncer::settings::{AppConfig, HttpSettings, IndexerSettings};
use chain_syncer::{combine, info};
use futures::future::ready;
use redbit::redb::Database;
use redbit::*;
use std::env;
use std::sync::Arc;
use ergo_lib::chain::block::FullBlock;

async fn maybe_run_server(http_conf: &HttpSettings, db: Arc<Database>) -> () {
    if http_conf.enable {
        info!("Starting http server at {}", http_conf.bind_address);
        serve(RequestState { db: Arc::clone(&db) }, http_conf.bind_address).await
    } else {
        ready(()).await
    }
}

async fn maybe_run_indexing(index_config: &IndexerSettings, scheduler: Scheduler<FullBlock, Block>) -> () {
    if index_config.enable {
        info!("Starting indexing process");
        scheduler.schedule(&index_config).await
    } else {
        ready(()).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app_config = AppConfig::new("config/settings")?;
    let ergo_config = ErgoConfig::new("config/ergo")?;
    let db_path: String = format!("{}/{}/{}", app_config.indexer.db_path, "main", "ergo");
    let db = Arc::new(storage::get_db(env::home_dir().unwrap().join(&db_path))?);
    let fetching_par: usize = app_config.indexer.fetching_parallelism.clone().into();
    
    let block_provider: Arc<dyn BlockProvider<FullBlock, Block>> = Arc::new(ErgoBlockProvider::new(&ergo_config, fetching_par));
    let block_persistence: Arc<dyn BlockPersistence<Block>> = Arc::new(ErgoBlockPersistence { db: Arc::clone(&db) });
    let scheduler: Scheduler<FullBlock, Block> = Scheduler::new(block_provider, block_persistence);

    let indexing_f = maybe_run_indexing(&app_config.indexer, scheduler);
    let server_f = maybe_run_server(&app_config.http, Arc::clone(&db));
    combine::futures(indexing_f, server_f).await;
    Ok(())
}
