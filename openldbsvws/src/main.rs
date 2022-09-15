use reqwest::Client;
use clap::Command;
use anyhow::Result;
use tokio::runtime::Builder;
use openldbsvws_lib::{get_service_details};

fn main() -> Result<()> {
    let matches = Command::new("openldbsvws")
        .subcommand_required(true)
        .about("query data from openldbsvws")
        .version("0.1.0")
        .subcommand(
            Command::new("service")
                .about("Gets information about a service")
                .arg(
                    clap::arg!(<SERVICE>)
                        .required(true)
                )
                .arg(
                    clap::arg!(-t <TOKEN>)
                        .id("TOKEN")
                        .required(true)
                )
        )
        .get_matches();

    let client = Client::new();
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    match matches.subcommand() {
        Some(("service", sub_matches)) => {
            let service = sub_matches.get_one::<String>("SERVICE").expect("required");
            let token = sub_matches.get_one::<String>("TOKEN").expect("required");
            println!("Getting information for service {}", service);
            
            println!("{:#?}", rt.block_on(get_service_details(client, token, service))?);

            Ok(())
        },
        _ => unreachable!()
    }
}
