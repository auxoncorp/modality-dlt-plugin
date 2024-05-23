#![allow(unused)]
use tokio::net::TcpStream;

struct Config {

}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut stream = TcpStream::connect("localhost:3049").await?;

//    dlt_core::parse::dlt_message(input, filter_config_opt, with_storage_header)
    Ok(())
}
