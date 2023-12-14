use crate::db::catalog_db::{CatalogDb, CatalogDbClient};
use crate::services::catalog_service::CatalogService;
use actix_web::{get, web, Responder};
use event_bus::EventBus;
use std::sync::Arc;

#[get("/catalog")]
pub async fn get_catalog(catalog_service: web::Data<Arc<CatalogService<EventBus, CatalogDbClient>>>) -> impl Responder {
    let items = catalog_service.get_items();
    if items.is_empty() {
        return format!("We are out of stock on everything, sorry!");
    }
    serde_json::to_string(&items).unwrap()
}

#[get("/catalog/stock/{item_id}")]
// this request handler would not be exposed by an api gateway
pub async fn get_stock(
    item_id: web::Path<u32>,
    catalog_service: web::Data<Arc<CatalogService<EventBus, CatalogDbClient>>>,
) -> impl Responder {
    let stock_amount_result = catalog_service.get_stock(item_id.into_inner());
    if stock_amount_result.is_err() {
        return format!("This item does not exist.");
    }
    stock_amount_result.unwrap().to_string()
}
