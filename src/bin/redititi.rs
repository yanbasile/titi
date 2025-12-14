//! Redititi - Redis-like server for terminal automation
//!
//! A standalone server that enables programmatic control of terminal sessions.

use std::process;
<<<<<<< HEAD
use titi::redititi_server::{RedititiTcpServer, TokenAuth};
=======
use titi::server::{TcpServer, TokenAuth};
>>>>>>> 962c700 (Add working Titi + Redititi integration tests)

#[tokio::main]
async fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut port = 6379;
    let mut token_file = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Invalid port number");
                        process::exit(1);
                    });
                    i += 2;
                } else {
                    eprintln!("--port requires a value");
                    process::exit(1);
                }
            }
            "--token-file" => {
                if i + 1 < args.len() {
                    token_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("--token-file requires a value");
                    process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            "--version" | "-v" => {
                println!("redititi {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                process::exit(1);
            }
        }
    }

    // Initialize authentication
    let auth = TokenAuth::new().unwrap_or_else(|e| {
        eprintln!("Failed to initialize authentication: {:?}", e);
        process::exit(1);
    });

    log::info!("═══════════════════════════════════════════════");
    log::info!("  Redititi - Terminal Automation Server");
    log::info!("═══════════════════════════════════════════════");
    log::info!("Port:       {}", port);
    log::info!("Token file: {:?}", auth.token_path());
    log::info!("Token:      {}", auth.token());
    log::info!("");
    log::info!("Server starting...");

    // Create and run server
    let addr = format!("127.0.0.1:{}", port);
<<<<<<< HEAD
    let server = RedititiTcpServer::new(addr, auth);
=======
    let server = TcpServer::new(addr, auth);
>>>>>>> 962c700 (Add working Titi + Redititi integration tests)

    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
        process::exit(1);
    }
}

fn print_help() {
    println!("Redititi - Redis-like server for terminal automation");
    println!();
    println!("USAGE:");
    println!("    redititi [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -p, --port <PORT>             Port to listen on (default: 6379)");
    println!("    --token-file <FILE>           Custom token file location");
    println!("    -h, --help                    Print help information");
    println!("    -v, --version                 Print version information");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    TITI_TOKEN                    Authentication token (overrides file)");
    println!("    RUST_LOG                      Logging level (debug, info, warn, error)");
    println!();
    println!("EXAMPLES:");
    println!("    redititi                      Start server on default port 6379");
    println!("    redititi --port 16379         Start server on port 16379");
    println!("    RUST_LOG=debug redititi       Start with debug logging");
}
