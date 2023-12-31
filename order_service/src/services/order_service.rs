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

#[derive(PartialEq)]
pub enum PlaceOrderError {
    ItemOutOfStock,
    CatalogNetworkError,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::order_db::{MockOrderDb, Order};
    use crate::networking::catalog_network_service::MockCatalogNetworkService;
    use event_bus::*;
    use networking::{NetworkError, NetworkErrorType};

    fn generate_random_order() -> Order {
        Order::new(
            1,
            OrderRequest {
                item_id: 1,
                name: "something".to_string(),
                address: "hello".to_string(),
                quantity: 22,
            },
        )
    }

    fn generate_random_order_request() -> OrderRequest {
        OrderRequest {
            item_id: 1,
            name: "something".to_string(),
            address: "hello".to_string(),
            quantity: 22,
        }
    }

    #[test]
    fn test_new_order_service() {
        // prepare
        let mock_event_listener = MockEventBus::new();
        let mut mock_order_db = MockOrderDb::new();
        let mock_catalog_network_service = MockCatalogNetworkService::new();
        mock_order_db.set_expected_order(Some(generate_random_order()));

        // act
        let sut = OrderService::new(mock_order_db, mock_event_listener, mock_catalog_network_service);

        // assert that db is mocked and accessible to confirm initialization
        assert_eq!(
            sut.db.lock().unwrap().get_order(1).unwrap().address,
            "hello".to_string()
        );
    }

    #[tokio::test]
    async fn test_place_order_catalog_network_error() {
        // prepare
        let mock_event_listener = MockEventBus::new();
        let mock_order_db = MockOrderDb::new();
        let mut mock_catalog_network_service = MockCatalogNetworkService::new();
        mock_catalog_network_service.expect_get_stock().return_once(move |_| {
            Err(NetworkError {
                status_code: Some(500),
                error: NetworkErrorType::Standard,
            })
        });
        let sut = OrderService::new(mock_order_db, mock_event_listener, mock_catalog_network_service);

        // act
        let result = sut.place_order(&generate_random_order_request()).await;

        // assert
        assert!(result.is_err());
        assert!(result.unwrap_err() == PlaceOrderError::CatalogNetworkError)
    }

    #[tokio::test]
    async fn test_place_order_item_out_of_stock_error() {
        // prepare
        let mock_event_listener = MockEventBus::new();
        let mock_order_db = MockOrderDb::new();
        let mut mock_catalog_network_service = MockCatalogNetworkService::new();
        mock_catalog_network_service.expect_get_stock().return_once(move |_| Ok(21));
        let sut = OrderService::new(mock_order_db, mock_event_listener, mock_catalog_network_service);

        // act
        let result = sut.place_order(&generate_random_order_request()).await;

        // assert
        assert!(result.is_err());
        assert!(result.unwrap_err() == PlaceOrderError::ItemOutOfStock)
    }

    #[tokio::test]
    async fn test_place_order_success() {
        // prepare
        let mock_event_listener = MockEventBus::new();
        let mock_order_db = MockOrderDb::new();
        let mut mock_catalog_network_service = MockCatalogNetworkService::new();
        mock_catalog_network_service.expect_get_stock().return_once(move |_| Ok(25));
        let sut = OrderService::new(mock_order_db, mock_event_listener, mock_catalog_network_service);

        // act
        let result = sut.place_order(&generate_random_order_request()).await;

        // assert
        assert!(result.is_ok());
    }
}
