// main.rs

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_files::Files;
use dashmap::DashMap;
use dotenv::dotenv;
use models::data_structures::Bone;

mod routes;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Initialize shared state
    type SharedState = Bone;
    let shared_state: SharedState = Arc::new(DashMap::new());

    // Get the static files path
    let static_files_path = env::var("STATIC_FILES_PATH").unwrap_or_else(|_| "./static".to_string());
    let static_files_path = PathBuf::from(static_files_path);
    println!("http://localhost:6060");

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_state.clone()))
            .wrap(Logger::default())
            .route("/ws/{boof_path}", web::get().to(routes::websocket::lore_exchange))
            .configure(routes::api::init)
            .service(Files::new("/identity/{path}", static_files_path.clone()).index_file("index.html"))
            .service(Files::new("/", static_files_path.clone()).index_file("index.html"))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
