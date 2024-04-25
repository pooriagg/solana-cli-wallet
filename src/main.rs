use solana_sdk::{
    signature::{Keypair, Signer},
    pubkey::{Pubkey},
    commitment_config::{CommitmentConfig},
    transaction::{Transaction},
    system_instruction,
    native_token::{
        lamports_to_sol,
        sol_to_lamports
    }
};
use solana_client::rpc_client::RpcClient;

use std::{
    time::Duration,
    thread::sleep,
    io,
    io::prelude::*,
    fs::{
        File,
        OpenOptions
    }
};

use clearscreen;
use spinoff::{Spinner, spinners, Color};
use colored;

mod executor {
    use std::{str::FromStr, io::BufReader};

    use colored::Colorize;
    use crate::*;

    pub fn show_menu(
        connection: &RpcClient,
        keypair: &Keypair
    ) {
        clearscreen::clear().expect("Failed to clear the screen.");
        connection.get_slot().expect("Connection lost.");

        let user_balance = connection.get_balance(&keypair.pubkey()).unwrap_or(0);

        println!("\n================== Menu ===================\n");
        println!("💎 Balance - {} SOL\n", lamports_to_sol(user_balance));
        println!("1. Show History");
        println!("2. Transfer Sol");
        println!("3. {}\n", "Exit".bold().red());
        println!("Enter a option :");

        let mut option = String::new();
        loop {
            io::stdin().read_line(&mut option).unwrap_or(0);
            let number: u8 = option.trim().parse().unwrap_or_default();
    
            match number {
              1 => show_logs(
                    connection,
                    keypair
                ),
              2 => transfer_sol(
                    connection,
                    keypair
                ),
              3 => terminate_program(),
              0 => println!("Please enter a number."),
              _ => println!("Please enter valid option.")
            };
    
            option.clear();
        };
    }

    pub fn transfer_sol(
        connection: &RpcClient,
        keypair: &Keypair
    ) {
        clearscreen::clear().expect("Failed to clear the screen.");

        println!("\n=================== Transfer ===================\n");
        // Get recepient
        println!("Enter recepient address :");
        let mut input_1 = String::new();
        io::stdin().read_line(&mut input_1).unwrap();
        let recepient = Pubkey::from_str(input_1.trim()).unwrap();
        // Get lamports amount to send
        println!("\nEnter sol amount to send :");
        let mut input_2 = String::new();
        io::stdin().read_line(&mut input_2).unwrap();
        let sol = input_2.trim().parse::<f64>().expect("Please enter a valid sol amount.");

        println!();

        let mut sp = Spinner::new(
            spinners::Dots,
            "Creating transaction, please wait... 📦",
            Color::White
        );

        // Create and send transaction with transfer instruction
        let from = keypair.pubkey();
        let to = recepient;
        let lamports = sol_to_lamports(sol);

        let transfer_instruction = system_instruction::transfer(
            &from,
            &to,
            lamports
        );
        let latest_blockhash = connection.get_latest_blockhash().expect("Failed to fetch the latest block hash.");
        let tx = Transaction::new_signed_with_payer(
            &[transfer_instruction],
            Some(&from),
            &[keypair],
            latest_blockhash
        );

        sp.update(
            spinners::Dots,
            "Sending transaction... ⏩",
            Color::Green
        );

        let tx_sig = connection.send_and_confirm_transaction(&tx).expect("Failed to send transaction.");
        loop {
            let result = connection.confirm_transaction_with_commitment(
                &tx_sig,
                CommitmentConfig::finalized()
            ).unwrap();
            if result.value {
                break;
            };
        };

        // Write logs
        let log = format!(
            "============================\nFrom : {:?},\nTo : {:?},\nSol : {:?},\nTransaction Signature : {:?}\n============================\n\n",
            from,
            to,
            lamports_to_sol(lamports),
            tx_sig
        );
        let mut option = OpenOptions::new();
        let mut file = option.create(true).write(true).append(true).open("./logs.txt").unwrap();
        file.write(log.as_bytes());
        drop(file);

        sp.stop_and_persist("🚀", "Transaction sent successfully.");

        println!();

        // Returning to the menu section
        println!("Press any key to return to menu...");
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer);

        show_menu(
            connection,
            keypair
        );
    }

    pub fn show_logs(
        connection: &RpcClient,
        keypair: &Keypair
    ) {
        clearscreen::clear().expect("Failed to clear the screen.");

        let file = File::open("./logs.txt");
        match file {
            Ok(f) => {
                let mut buffer = String::new();
                let mut reader = BufReader::new(f);
                let mut line: usize;
                loop {
                    line = reader.read_line(&mut buffer).unwrap();
                    if line == 0 {
                        break;
                    };
                };
        
                io::stdout().write_all(buffer.trim().as_bytes()).unwrap();
        
                println!("\n");
        
                // Returning to the menu section
                println!("Press any key to return to menu...");
                let mut any_key = String::new();
                io::stdin().read_line(&mut any_key);
        
                show_menu(
                    connection,
                    keypair
                );
            },
            Err(_) => {
                println!("⚠️  Cannot find 'logs.txt' file to read the history! Press any key to return back to the menu...");
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer);
                show_menu(connection, keypair);
            }
        };
    }

    pub fn terminate_program() {
        clearscreen::clear().expect("Failed to clear the screen.");

        let mut sp = Spinner::new(spinners::Clock, "Closing wallet in 3 secs...", Color::Red);
        sleep(Duration::from_secs(3));
        sp.update(spinners::Clock, "📢 Wallet Closed.", Color::Red);
        sp.stop();

        println!();

        std::process::exit(0x0);
    }

    pub fn trim_keypair(buffer: &str) -> Vec<u8> {
        let mut output = buffer.trim().to_string();
    
        output = output.replace("[", "");
        output = output.replace("]", "");
    
        let temp: Vec<_> = output.split(",").collect();
        let mut secret_key: Vec<_> = temp.iter().map(|byte| byte.parse::<u8>().unwrap()).collect();
    
        return secret_key;
    }
}

fn main() {
    // Establishing connection to solana blockchain
    let connection: RpcClient = RpcClient::new_with_commitment(
        "<API_KEY>".to_owned(),
        CommitmentConfig::confirmed()
    );

    let file = File::open("./keypair.json");
    match file {
        Ok(f) => {
            let mut reader = io::BufReader::new(f);
            let mut buffer = String::new();
        
            reader.read_line(&mut buffer).unwrap();
            
            let secret_key = executor::trim_keypair(&buffer);
            let keypair = Keypair::from_bytes(&secret_key.as_ref()).expect("Keypair is invalid.");
            
            // Run program
            executor::show_menu(
                &connection,
                &keypair
            );
        },
        Err(_) => {
            println!("Please place your valid keypair(.json) file in the root of the wallet program and press a key... 📁");
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer);
            main();
        }
    };
}
