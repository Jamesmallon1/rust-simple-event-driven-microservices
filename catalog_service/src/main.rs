use crate::db::catalog_db::CatalogDb;
mod api;
mod db;
mod services;

use crate::db::catalog_db::CatalogDbClient;
use crate::services::catalog_service::CatalogService;
use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::{web, App, HttpServer};
use common::constants::global_constants;
use common::traits::listener_service::ListenerService;
use common::utilities::logger;
use event_bus::EventBus;
use std::sync::{Arc, RwLock};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logger::initialize("catalog_output.log", "Catalog");
    initialize_server().await
}

async fn initialize_server() -> std::io::Result<()> {
    let mock_db: CatalogDbClient = CatalogDbClient::new();
    let event_bus = EventBus::new(&format!(
        "{}:{}",
        global_constants::HOST,
        global_constants::EVENT_BUS_PORT
    ));
    let mut raw_catalog_service = CatalogService::new(mock_db, event_bus);
    raw_catalog_service.start_event_listeners();
    let catalog_service = Arc::new(raw_catalog_service);
    HttpServer::new(move || {
        App::new()
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .app_data(web::Data::new(catalog_service.clone()))
            .service(api::get_catalog)
            .service(api::get_stock)
    })
    .bind((global_constants::HOST, global_constants::CATALOG_SERVICE_PORT))?
    .run()
    .await
}
