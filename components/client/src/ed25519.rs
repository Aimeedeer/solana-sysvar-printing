use anyhow::Result;
use common::DemoEd25519Instruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    ed25519_instruction,
    feature_set::FeatureSet,
    instruction::Instruction,
    precompiles::PrecompileError,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::sync::Arc;

use ed25519_dalek::{
    ed25519::signature::Signature as Ed25519Signature, Keypair as Ed25519Keypair,
    Signer as Ed25519Signer, Verifier,
};
use rand::rngs::OsRng;

pub fn demo_via_ed25519_program(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let instr = DemoEd25519Instruction.build_instruction(&program_keypair.pubkey());
    let ed25519_instr = todo!();

    let blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ed25519_instr, instr],
        Some(&config.keypair.pubkey()),
        &[&config.keypair],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("sig: {}", sig);

    Ok(())
}

pub fn demo_new_instruction_and_verify(
    config: &crate::util::Config,
    client: &RpcClient,
    program_keypair: &Keypair,
) -> Result<()> {
    let mut csprng = OsRng {};
    let keypair: Ed25519Keypair = Ed25519Keypair::generate(&mut csprng);

    let message: &[u8] = b"This is a demo message.";
    let signature = keypair.sign(message);

    let instruction = ed25519_instruction::new_ed25519_instruction(&keypair, message);
    println!(
        "ed25519_instruction program id: {:#?}",
        instruction.program_id
    );
    assert_eq!(instruction.program_id, solana_sdk::ed25519_program::id());

    let ok = ed25519_instruction::verify(
        &instruction.data,
        &[&[0u8; 100]],
        &Arc::new(FeatureSet::all_enabled()),
    );
    assert_eq!(ok, Ok(()));

    let err = ed25519_instruction::verify(
        &message,
        &[&[0u8; 100]],
        &Arc::new(FeatureSet::all_enabled()),
    );
    assert_eq!(err, Err(PrecompileError::InvalidInstructionDataSize));

    Ok(())
}
