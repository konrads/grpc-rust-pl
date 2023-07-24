use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Mutex;

use payments::bitcoin_server::{Bitcoin, BitcoinServer};
use payments::{HintsRequest, InitAccountRequest, PaymentRequest, Reply};
use tonic::{transport::Server, Request, Response, Status};

pub mod payments {
    tonic::include_proto!("payments");
}

#[derive(Debug, Default)]
pub struct BitcoinService {
    balances: Mutex<HashMap<String, u32>>,
}

#[tonic::async_trait]
impl Bitcoin for BitcoinService {
    async fn init_account(
        &self,
        request: Request<InitAccountRequest>,
    ) -> Result<Response<Reply>, Status> {
        println!("Got an init request {request:?}");

        let req = request.into_inner();

        let mut balances = self
            .balances
            .lock()
            .expect("Failed to obtain balances lock");

        if let Entry::Vacant(_) = balances.entry(req.addr.clone()) {
            let init_balance = req.init_amount.unwrap_or(0);
            let resp = Ok(Response::new(Reply {
                successful: true,
                message: format!(
                    "Account {} initialized and balance set to {}",
                    &req.addr, init_balance
                ),
            }));
            balances.insert(req.addr, init_balance);
            resp
        } else {
            Ok(Response::new(Reply {
                successful: false,
                message: format!("Account {} already exists", req.addr),
            }))
        }
    }

    async fn send_payment(
        &self,
        request: Request<PaymentRequest>,
    ) -> Result<Response<Reply>, Status> {
        println!("Got a request {request:?}");

        let req = request.into_inner();

        let mut balances = self
            .balances
            .lock()
            .expect("Failed to obtain balances lock");

        let contains_from = balances.contains_key(&req.from_addr);
        let contains_to = balances.contains_key(&req.from_addr);

        match (contains_from, contains_to) {
            (true, true) => {
                if balances[&req.from_addr] >= req.amount {
                    let from_balance = balances[&req.from_addr];
                    let to_balance = balances[&req.to_addr];
                    let resp = Ok(Response::new(Reply {
                        successful: true,
                        message: format!(
                            "Amount {} transferred from {} to {}",
                            &req.amount, &req.from_addr, &req.to_addr
                        ),
                    }));
                    balances.insert(req.from_addr, from_balance - req.amount);
                    balances.insert(req.to_addr, to_balance + req.amount);
                    resp
                } else {
                    Ok(Response::new(Reply {
                        successful: false,
                        message: format!("Account {} has insufficient funds", req.from_addr),
                    }))
                }
            }
            (false, false) => Ok(Response::new(Reply {
                successful: false,
                message: format!(
                    "Neither from account {} nor to account {} exist",
                    req.from_addr, req.to_addr
                ),
            })),
            (false, true) => Ok(Response::new(Reply {
                successful: false,
                message: format!("From account {} doesn't exist", req.from_addr),
            })),
            (true, false) => Ok(Response::new(Reply {
                successful: false,
                message: format!("To account {} doesn't exist", req.to_addr),
            })),
        }
    }

    async fn send_hints(&self, request: Request<HintsRequest>) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        println!(
            "Hints from client {}",
            req.hints
                .iter()
                .map(|s| format!("\n* {}", s))
                .collect::<Vec<_>>()
                .join("")
        );
        Ok(Response::new(()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let btc_service = BitcoinService::default();

    Server::builder()
        .add_service(BitcoinServer::new(btc_service))
        .serve(addr)
        .await?;
    Ok(())
}
