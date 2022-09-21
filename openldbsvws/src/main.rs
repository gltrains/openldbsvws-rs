use anyhow::Result;
use clap::Command;
use openldbsvws_lib::ServiceDetails;
use reqwest::Client;
use std::time::Duration;
use tokio::runtime::Builder;

macro_rules! service_details {
    () => {"<soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:typ=\"http://thalesgroup.com/RTTI/2013-11-28/Token/types\" xmlns:ldb=\"http://thalesgroup.com/RTTI/2021-11-01/ldbsv/\"><soapenv:Header><typ:AccessToken><typ:TokenValue>{token}</typ:TokenValue></typ:AccessToken></soapenv:Header><soapenv:Body><ldb:GetServiceDetailsByRIDRequest><ldb:rid>{rid}</ldb:rid></ldb:GetServiceDetailsByRIDRequest></soapenv:Body></soapenv:Envelope>"}
}

macro_rules! arrival_details {
    () => {"<soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:typ=\"http://thalesgroup.com/RTTI/2013-11-28/Token/types\" xmlns:ldb=\"http://thalesgroup.com/RTTI/2021-11-01/ldb/\"><soapenv:Header><typ:AccessToken><typ:TokenValue>{token}</typ:TokenValue></typ:AccessToken></soapenv:Header><soapenv:Body><ldb:GetArrivalBoardRequest><ldb:numRows>150</ldb:numRows><ldb:crs>{crs}</ldb:crs><ldb:filterCrs>{filter_crs}</ldb:filterCrs><ldb:filterType>{filter_type}</ldb:filterType><ldb:timeOffset>{time_offset}</ldb:timeOffset><ldb:timeWindow>{time_window}</ldb:timeWindow></ldb:GetArrivalBoardRequest></soapenv:Body></soapenv:Envelope>"}
}

fn main() -> Result<()> {
    let matches = Command::new("openldbsvws")
        .subcommand_required(true)
        .about("query data from openldbsvws")
        .version("0.1.0")
        .subcommand(
            Command::new("service")
                .about("Gets information about a service")
                .arg(clap::arg!(<SERVICE>).required(true))
                .arg(clap::arg!(-t <TOKEN>).id("TOKEN").required(true)),
        )
        .get_matches();

    let client = Client::new();
    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    match matches.subcommand() {
        Some(("service", sub_matches)) => {
            let service = sub_matches.get_one::<String>("SERVICE").expect("required");
            let token = sub_matches.get_one::<String>("TOKEN").expect("required");
            println!("Getting information for service {}", service);

            let result = rt.block_on(async {
                let service_details_payload =
                    format!(service_details!(), token = token, rid = service);
                let res = client
                    .post("https://lite.realtime.nationalrail.co.uk/OpenLDBSVWS/ldbsv13.asmx")
                    .body(service_details_payload)
                    .timeout(Duration::new(5, 0))
                    .header("Content-Type", "text/xml")
                    .header("Accept", "text/xml")
                    .send()
                    .await
                    .unwrap();

                let status = res.status();

                let string = res.text().await.unwrap();

                if !status.is_success() {
                    panic!();
                }

                ServiceDetails::try_from(&*string);

                todo!()
            });

            println!("{:#?}", result);

            Ok(())
        }
        _ => unreachable!(),
    }
}
