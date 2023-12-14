use crate::db::order_db::OrderDb;
use crate::model::OrderRequest;
use crate::networking::catalog_network_service::CatalogNetworkService;
use crate::MICROSERVICE_NAME;
use event_bus::event::Event;
use event_bus::events::order_placed_event::OrderPlacedEvent;
use event_bus::{topic, EventProducer};
use log::{error, info};
use std::sync::Mutex;

pub struct OrderService<E: EventProducer, D: for<'a> OrderDb<'a>, C: CatalogNetworkService> {
    event_bus: E,
    db: Mutex<D>,
    catalog_network_service: C,
}

impl<E: EventProducer, D: for<'a> OrderDb<'a>, C: CatalogNetworkService> OrderService<E, D, C> {
    /// Creates a new instance of `OrderService`.
    ///
    /// This method initializes the service with a given mock order database, a network service to
    /// contact the catalog microservice and the event producer to notify other services.
    ///
    /// Arguments:
    /// - `db`: An instance of `MockOrderDb` to be used by this service.
    /// - `event_bus`: An instance of `EventBus` to be used by this service.
    /// - `catalog_network_service`: An instance of `CatalogNetworkService` to be used by this service.
    ///
    /// Returns:
    /// - `OrderService`: A new instance of `OrderService`.
    pub fn new(db: D, event_bus: E, catalog_network_service: C) -> OrderService<E, D, C> {
        let db = Mutex::new(db);
        OrderService {
            event_bus,
            db,
            catalog_network_service,
        }
    }

    /// Places an order for a clothing item.
    ///
    /// This method handles the process of placing an order, including checking stock availability,
    /// updating the database with the new order, and broadcasting an event to indicate that an order has been placed.
    ///
    /// The function performs the following operations:
    /// 1. Checks the stock of the requested item using the `catalog_network_service`.
    /// 2. If the requested quantity exceeds the available stock, it returns an `ItemOutOfStock` error.
    /// 3. Adds the order to the database.
    /// 4. Broadcasts an `order_placed` event to notify other parts of the system.
    ///
    /// Note: In case of a failure while broadcasting the event, the error is logged but not propagated.
    ///       The order placement is considered successful even if event broadcasting fails.
    ///
    /// Arguments:
    /// * `order_request`: The `OrderRequest` object containing details of the item to be ordered, including item ID and quantity.
    ///
    /// Returns:
    /// * `Result<(), PlaceOrderError>`: Ok(()) if the order is successfully placed, or an appropriate error in case of failure.
    ///
    /// Errors:
    /// * `CatalogNetworkError`: If there is a failure in network communication with the catalog service.
    /// * `ItemOutOfStock`: If the requested quantity exceeds the available stock.
    pub async fn place_order(&self, order_request: &OrderRequest) -> Result<(), PlaceOrderError> {
        info!("Handling a request to place an order: {}", order_request);
        // check the stock of the item
        let stock = self.catalog_network_service.get_stock(order_request.item_id).await.map_err(|err| {
            error!("An error has occurred whilst contacting Catalog: {:?}", err);
            PlaceOrderError::CatalogNetworkError
        })?;

        if order_request.quantity > stock {
            return Err(PlaceOrderError::ItemOutOfStock);
        }

        // place order
        let mut db_guard = self.db.lock().unwrap();
        db_guard.add_order(order_request.clone());

        // send event for order placed
        let inner_event = OrderPlacedEvent {
            item_id: order_request.item_id,
            quantity: order_request.quantity,
        };

        let event = Event::new(
            "order_placed".to_string(),
            inner_event,
            MICROSERVICE_NAME.to_string(),
            None,
            None,
        );

        self.event_bus
            .broadcast_event(event, topic::ORDER_PLACED, &order_request.item_id.to_string())
            .await
            .map_err(|err| {
                error!(
                    "Could not send {} event, error occurred: {:?}",
                    topic::ORDER_PLACED,
                    err
                );
                // consider how to handle this error for example, log it, alert, or retry
                // currently, this error is logged but not propagated
                ()
            })
            .ok();

        Ok(())
    }
}

pub enum PlaceOrderError {
    ItemOutOfStock,
    CatalogNetworkError,
}
