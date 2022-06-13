use anyhow::Result;
use futures_util::StreamExt;
use solana_account_decoder::UiAccount;
use solana_client::rpc_response::Response;
use solana_client::rpc_response::SlotInfo;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient, rpc_client::RpcClient,
    rpc_config::RpcAccountInfoConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    rpc_port,
    signature::{Keypair, Signer},
    system_transaction,
    sysvar::rent::Rent,
    transaction::Transaction,
};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::unbounded_channel;
use tokio::task;

pub fn demo_pubsub_client_async(
    config_keypair: Keypair,
    rpc_client: RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(async move {
        let (account_sender, mut account_receiver) = unbounded_channel::<Response<UiAccount>>();
        let (slot_sender, mut slot_receiver) = unbounded_channel::<SlotInfo>();

        let ws_url = &format!("ws://127.0.0.1:{}/", rpc_port::DEFAULT_RPC_PUBSUB_PORT);
        // let ws_url = "wss://api.devnet.solana.com/";

        let pubsub_client = Arc::new(PubsubClient::new(ws_url).await.unwrap());

        let config_pubkey = config_keypair.pubkey();

        tokio::spawn({
            let _pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut account_notifications, account_unsubscribe) = _pubsub_client
                    .account_subscribe(
                        &config_pubkey,
                        Some(RpcAccountInfoConfig {
                            commitment: Some(CommitmentConfig::confirmed()),
                            ..RpcAccountInfoConfig::default()
                        }),
                    )
                    .await
                    .unwrap();

                while let Some(account) = account_notifications.next().await {
                    account_sender.send(account).unwrap();
                }
                account_unsubscribe().await;
            }
        });

        tokio::spawn({
            let _pubsub_client = Arc::clone(&pubsub_client);
            async move {
                let (mut slot_notifications, slot_unsubscribe) =
                    _pubsub_client.slot_subscribe().await.unwrap();
                while let Some(slot_info) = slot_notifications.next().await {
                    slot_sender.send(slot_info).unwrap();
                }
                slot_unsubscribe().await;
            }
        });

        task::spawn(async move {
            loop {
                if let Some(result) = account_receiver.recv().await {
                    println!("account pubsub result: {:?}", result);
                }
            }
        });

        task::spawn(async move {
            loop {
                if let Some(result) = slot_receiver.recv().await {
                    println!("slot pubsub result: {:?}", result);
                }
            }
        });

        task::spawn_blocking(move || {
            // Create transaction for pubsub test
            let transfer_amount = Rent::default().minimum_balance(0);
            let recent_blockhash = rpc_client.get_latest_blockhash().unwrap();
            let transactions: Vec<Transaction> = (0..10)
                .map(|_| {
                    system_transaction::transfer(
                        &config_keypair,
                        &solana_sdk::pubkey::new_rand(),
                        transfer_amount,
                        recent_blockhash,
                    )
                })
                .collect();

            for tx in transactions {
                let sig = rpc_client.send_and_confirm_transaction(&tx).unwrap();
                println!("transfer sig: {}", sig);
            }
        });

        loop {
            tokio::task::yield_now();
        }
    });

    Ok(())
}
