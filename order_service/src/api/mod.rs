use crate::db::order_db::OrderDbClient;
use crate::model::OrderRequest;
use crate::networking::catalog_network_service::CatalogApiClient;
use crate::services::order_service::{OrderService, PlaceOrderError};
use actix_web::{post, web, Responder};
use event_bus::EventBus;
use std::sync::Arc;

#[post("/order")]
pub async fn place_order(
    order_request: web::Json<OrderRequest>,
    order_service: web::Data<Arc<OrderService<EventBus, OrderDbClient, CatalogApiClient>>>,
) -> impl Responder {
    let result = order_service.get_ref().place_order(&order_request).await;
    if let Err(err) = result {
        return match err {
            PlaceOrderError::ItemOutOfStock => format!("This item is out of stock"),
            PlaceOrderError::CatalogNetworkError => {
                format!("An error occurred and some of our systems are down, please try again later.")
            }
        };
    }

    return format!(
        "Order has been placed successfully! It's on its way to: {} at {}",
        order_request.name, order_request.address
    );
}
