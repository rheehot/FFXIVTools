mod ffxiv_data;

use std::error::Error;

use actix_web::{App, HttpServer, Result};

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    HttpServer::new(move || App::new().configure(ffxiv_data::config))
        .bind("127.0.0.1:8080")?
        .run()
        .await?;

    Ok(())
}
