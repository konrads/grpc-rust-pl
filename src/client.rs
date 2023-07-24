use clap::{Parser, Subcommand};
use payments::bitcoin_client::BitcoinClient;
use payments::{HintsRequest, InitAccountRequest, PaymentRequest};
use tonic::Request;

pub mod payments {
    tonic::include_proto!("payments");
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init {
        #[arg(short, long)]
        address: String,
        #[arg(short, long)]
        init_amount: Option<u32>,
    },
    Transfer {
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
        #[arg(short, long)]
        amount: u32,
    },
    Hints {
        #[arg(long, value_delimiter = ',')]
        hints: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut client = BitcoinClient::connect("http://[::1]:50051").await?;

    match args.command {
        Command::Init {
            address,
            init_amount,
        } => {
            let reply = client
                .init_account(Request::new(InitAccountRequest {
                    addr: address,
                    init_amount,
                }))
                .await?;
            if reply.get_ref().successful {
                println!("SUCCESS: {}", reply.get_ref().message);
            } else {
                println!("FAILURE: {}", reply.get_ref().message);
                std::process::exit(1);
            }
        }
        Command::Transfer { from, to, amount } => {
            let reply = client
                .send_payment(Request::new(PaymentRequest {
                    from_addr: from,
                    to_addr: to,
                    amount,
                }))
                .await?;
            if reply.get_ref().successful {
                println!("SUCCESS: {}", reply.get_ref().message);
            } else {
                println!("FAILURE: {}", reply.get_ref().message);
                std::process::exit(1);
            }
        }
        Command::Hints { hints } => {
            client
                .send_hints(Request::new(HintsRequest { hints }))
                .await?;
            println!("Hints sent");
        }
    };

    Ok(())
}
