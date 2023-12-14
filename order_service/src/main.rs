mod api;
mod db;
mod model;
mod networking;
mod services;

use crate::db::order_db::{OrderDb, OrderDbClient};
use crate::networking::catalog_network_service::CatalogApiClient;
use crate::services::order_service::OrderService;
use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::{web, App, HttpServer};
use common::constants::global_constants;
use common::utilities::logger;
use event_bus::EventBus;
use std::sync::Arc;

pub const MICROSERVICE_NAME: &str = "Order";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logger::initialize("order_output.log", MICROSERVICE_NAME);
    initialize_server().await
}

async fn initialize_server() -> std::io::Result<()> {
    let mock_db = OrderDbClient::new();
    let event_bus = EventBus::new(&format!(
        "{}:{}",
        global_constants::HOST,
        global_constants::EVENT_BUS_PORT
    ));
    let catalog_network_service = CatalogApiClient {
        host: format!(
            "http://{}:{}",
            global_constants::HOST,
            global_constants::CATALOG_SERVICE_PORT
        ),
    };
    let order_service = Arc::new(OrderService::new(mock_db, event_bus, catalog_network_service));
    HttpServer::new(move || {
        App::new()
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .app_data(web::Data::new(order_service.clone()))
            .service(api::place_order)
    })
    .bind((global_constants::HOST, global_constants::ORDER_SERVICE_PORT))?
    .run()
    .await
}
