//! Tendermint RPC client

use tendermint::{
    abci::{self, Transaction},
    block::Height,
    evidence::Evidence,
    net, Genesis,
};

use crate::{
    client::transport::{http_ws::HttpWsTransport, Transport},
    endpoint::*,
    Error, Request, Response,
};

pub mod event_listener;
pub mod transport;

/// Tendermint RPC client.
///
/// Presently supports JSONRPC via HTTP.
#[derive(Debug)]
pub struct Client {
    transport: Box<dyn Transport>,
}

impl Client {
    /// Create a new Tendermint RPC client, connecting to the given address
    pub fn new(address: net::Address) -> Result<Self, Error> {
        Ok(Self {
            transport: Box::new(HttpWsTransport::new(address)?),
        })
    }

    /// `/abci_info`: get information about the ABCI application.
    pub async fn abci_info(&self) -> Result<abci_info::AbciInfo, Error> {
        Ok(self.perform(abci_info::Request).await?.response)
    }

    /// `/abci_query`: query the ABCI application
    pub async fn abci_query(
        &self,
        path: Option<abci::Path>,
        data: impl Into<Vec<u8>>,
        height: Option<Height>,
        prove: bool,
    ) -> Result<abci_query::AbciQuery, Error> {
        Ok(self
            .perform(abci_query::Request::new(path, data, height, prove))
            .await?
            .response)
    }

    /// `/block`: get block at a given height.
    pub async fn block(&self, height: impl Into<Height>) -> Result<block::Response, Error> {
        self.perform(block::Request::new(height.into())).await
    }

    /// `/block`: get the latest block.
    pub async fn latest_block(&self) -> Result<block::Response, Error> {
        self.perform(block::Request::default()).await
    }

    /// `/block_results`: get ABCI results for a block at a particular height.
    pub async fn block_results<H>(&self, height: H) -> Result<block_results::Response, Error>
    where
        H: Into<Height>,
    {
        self.perform(block_results::Request::new(height.into()))
            .await
    }

    /// `/block_results`: get ABCI results for the latest block.
    pub async fn latest_block_results(&self) -> Result<block_results::Response, Error> {
        self.perform(block_results::Request::default()).await
    }

    /// `/blockchain`: get block headers for `min` <= `height` <= `max`.
    ///
    /// Block headers are returned in descending order (highest first).
    ///
    /// Returns at most 20 items.
    pub async fn blockchain(
        &self,
        min: impl Into<Height>,
        max: impl Into<Height>,
    ) -> Result<blockchain::Response, Error> {
        // TODO(tarcieri): return errors for invalid params before making request?
        self.perform(blockchain::Request::new(min.into(), max.into()))
            .await
    }

    /// `/broadcast_tx_async`: broadcast a transaction, returning immediately.
    pub async fn broadcast_tx_async(
        &self,
        tx: Transaction,
    ) -> Result<broadcast::tx_async::Response, Error> {
        self.perform(broadcast::tx_async::Request::new(tx)).await
    }

    /// `/broadcast_tx_sync`: broadcast a transaction, returning the response
    /// from `CheckTx`.
    pub async fn broadcast_tx_sync(
        &self,
        tx: Transaction,
    ) -> Result<broadcast::tx_sync::Response, Error> {
        self.perform(broadcast::tx_sync::Request::new(tx)).await
    }

    /// `/broadcast_tx_sync`: broadcast a transaction, returning the response
    /// from `CheckTx`.
    pub async fn broadcast_tx_commit(
        &self,
        tx: Transaction,
    ) -> Result<broadcast::tx_commit::Response, Error> {
        self.perform(broadcast::tx_commit::Request::new(tx)).await
    }

    /// `/commit`: get block commit at a given height.
    pub async fn commit(&self, height: impl Into<Height>) -> Result<commit::Response, Error> {
        self.perform(commit::Request::new(height.into())).await
    }

    /// `/validators`: get validators a given height.
    pub async fn validators<H>(&self, height: H) -> Result<validators::Response, Error>
    where
        H: Into<Height>,
    {
        self.perform(validators::Request::new(height.into())).await
    }

    /// `/commit`: get the latest block commit
    pub async fn latest_commit(&self) -> Result<commit::Response, Error> {
        self.perform(commit::Request::default()).await
    }

    /// `/health`: get node health.
    ///
    /// Returns empty result (200 OK) on success, no response in case of an error.
    pub async fn health(&self) -> Result<(), Error> {
        self.perform(health::Request).await?;
        Ok(())
    }

    /// `/genesis`: get genesis file.
    pub async fn genesis(&self) -> Result<Genesis, Error> {
        Ok(self.perform(genesis::Request).await?.genesis)
    }

    /// `/net_info`: obtain information about P2P and other network connections.
    pub async fn net_info(&self) -> Result<net_info::Response, Error> {
        self.perform(net_info::Request).await
    }

    /// `/status`: get Tendermint status including node info, pubkey, latest
    /// block hash, app hash, block height and time.
    pub async fn status(&self) -> Result<status::Response, Error> {
        self.perform(status::Request).await
    }

    /// `/broadcast_evidence`: broadcast an evidence.
    pub async fn broadcast_evidence(&self, e: Evidence) -> Result<evidence::Response, Error> {
        self.perform(evidence::Request::new(e)).await
    }

    /// Perform a request against the RPC endpoint
    pub async fn perform<R>(&self, request: R) -> Result<R::Response, Error>
    where
        R: Request,
    {
        let request_body = request.into_json();
        let response_body = self.transport.request(request_body).await?;
        R::Response::from_string(response_body)
    }
}
