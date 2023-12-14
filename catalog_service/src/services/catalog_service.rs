use crate::db::catalog_db::{CatalogDb, ClothingItem};
use common::traits::listener_service::ListenerService;
use event_bus::event::Event;
use event_bus::events::order_placed_event::OrderPlacedEvent;
use event_bus::{topic, EventListener};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// `CatalogService` provides functionality to interact with a catalog database.
///
/// This service is responsible for accessing and manipulating catalog data,
/// abstracting the database operations from the client.
///
/// Fields:
/// - `db`: An instance of `MockCatalogDb` representing the mock catalog database.
pub struct CatalogService<E: EventListener, D: for<'a> CatalogDb<'a>> {
    event_bus: E,
    db: Arc<RwLock<D>>,
}

impl<E: EventListener, D: for<'a> CatalogDb<'a> + Send + Sync + 'static> ListenerService for CatalogService<E, D> {
    fn start_event_listeners(&mut self) {
        let listener = self
            .event_bus
            .create_event_listener::<Event<OrderPlacedEvent>>("group-1", &[topic::ORDER_PLACED])
            .expect(format!("Failed to initialize the {} listener", topic::ORDER_PLACED).as_str());

        let db_clone = self.db.clone();
        let mut receiver = listener.get_receiver();
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                let mut db = db_clone.write().unwrap();
                let item_result = db.get_mut_item(event.payload.item_id);
                match item_result {
                    None => {}
                    Some(item) => {
                        let mut stock_amount = item.stock;
                        if event.payload.quantity > stock_amount {
                            error!("Event to change stock levels has failed, Source: {}, Amount to change: {}, Current Amount: {}",
                                event.source,
                                event.payload.quantity,
                                item.stock);
                            continue;
                        }
                        stock_amount -= event.payload.quantity;
                        item.stock = stock_amount;
                        info!("Stock level for item: {} is now: {}", item.id, stock_amount);
                    }
                }
            }
        });
    }
}

impl<E: EventListener, D: for<'a> CatalogDb<'a>> CatalogService<E, D> {
    /// Creates a new instance of `CatalogService`.
    ///
    /// This method initializes the service with a given mock catalog database.
    ///
    /// Arguments:
    /// - `db`: An instance of `MockCatalogDb` to be used by this service.
    /// - `event_bus`: An instance of `EventBus` to be used by this service.
    ///
    /// Returns:
    /// - `CatalogService`: A new instance of `CatalogService`.
    pub fn new(db: D, event_bus: E) -> CatalogService<E, D> {
        let db = Arc::new(RwLock::new(db));
        CatalogService { event_bus, db }
    }

    /// Retrieves a list of available catalog items.
    ///
    /// This method returns a vector of `ClothingItemDTO` representing the items
    /// currently available in the catalog. It filters out items that have a stock of 0 or less,
    /// ensuring only items available for purchase are returned.
    ///
    /// Returns:
    /// - `Vec<ClothingItemDTO>`: A vector of DTOs for each available item in the catalog.
    pub fn get_items(&self) -> Vec<ClothingItemDTO> {
        info!("Handling a request view the catalog");
        let db = self.db.read().unwrap();
        let items = db.get_catalog();
        items.into_iter().filter(|item| item.stock > 0).map(ClothingItemDTO::from).collect()
    }

    /// Retrieves the stock quantity of a specific item in the catalog.
    ///
    /// This method searches the catalog database for an item with the given `item_id`.
    /// If the item exists, it returns the current stock quantity of that item.
    ///
    /// Arguments:
    /// - `item_id`: A `u32` identifier of the catalog item whose stock is being queried.
    ///
    /// Returns:
    /// - `Result<u32, ItemNotFoundError>`: On success, returns `Ok(u32)` representing the
    ///   stock quantity of the item. If the item is not found in the catalog, returns
    ///   `Err(ItemNotFoundError)`.
    ///
    /// Example:
    /// ```
    /// let service = CatalogService::new(mock_db);
    /// let stock = service.get_stock(123).expect("Item should exist");
    /// ```
    pub fn get_stock(&self, item_id: u32) -> Result<u32, ItemNotFoundError> {
        info!("Handling a request to get the stock of item: {}", item_id);
        let db = self.db.read().unwrap();
        let item = db.get_item(item_id);
        if item.is_none() {
            return Err(ItemNotFoundError);
        }

        Ok(item.unwrap().stock)
    }
}

/// `ClothingItemDTO` is a Data Transfer Object for `ClothingItem`.
///
/// This struct is used to communicate data about clothing items to clients,
/// excluding certain fields that are not necessary or should be kept private.
/// Specifically, it omits the `stock` field present in the `ClothingItem` struct.
/// The purpose of this struct is for transmitting it to the client so that they have
/// no knowledge of the stock of the item.
///
/// Fields:
/// - `id`: The unique identifier for the clothing item.
/// - `name`: The name of the clothing item.
/// - `description`: A description of the clothing item.
/// - `sizes`: A list of available sizes for the clothing item.
/// - `price`: The price of the clothing item.
/// - `images`: URLs to images of the clothing item.
/// - `video`: A URL to a video showcasing the clothing item.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClothingItemDTO {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub sizes: Vec<String>,
    pub price: f32,
    pub images: Vec<String>,
    pub video: String,
}

impl From<&ClothingItem> for ClothingItemDTO {
    fn from(item: &ClothingItem) -> Self {
        ClothingItemDTO {
            id: item.id.clone(),
            name: item.name.clone(),
            description: item.description.clone(),
            sizes: item.sizes.clone(),
            price: item.price.clone(),
            images: item.images.clone(),
            video: item.video.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ItemNotFoundError;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::catalog_db::MockCatalogDb;
    use event_bus::*;

    fn generate_random_item(item_id: u32, stock: u32) -> ClothingItem {
        ClothingItem {
            id: item_id,
            name: "random_item".to_string(),
            description: "desc".to_string(),
            sizes: vec!["S".to_string(), "M".to_string(), "L".to_string(), "XL".to_string()],
            price: 20.00,
            stock,
            images: vec![
                "https://example.com/t-shirt-front.jpg".to_string(),
                "https://example.com/t-shirt-back.jpg".to_string(),
            ],
            video: "https://example.com/t-shirt-video.mp4".to_string(),
        }
    }

    #[test]
    fn test_new_catalog_service() {
        // prepare
        let mock_event_listener = MockEventBus::new();
        let mut mock_catalog_db = MockCatalogDb::new();
        let t_shirt = generate_random_item(6, 50);
        mock_catalog_db.set_expected_get_item(Some(t_shirt.clone()));

        // act
        let sut = CatalogService::new(mock_catalog_db, mock_event_listener);

        // assert that db is mocked and accessible to confirm initialization
        assert_eq!(sut.get_stock(6).unwrap(), t_shirt.stock);
    }

    #[test]
    fn test_get_items() {
        // prepare
        let mock_event_listener = MockEventBus::new();
        let mut mock_catalog_db = MockCatalogDb::new();
        let vec = vec![generate_random_item(1, 25), generate_random_item(2, 50)];
        mock_catalog_db.set_expected_vec(vec);

        // act
        let sut = CatalogService::new(mock_catalog_db, mock_event_listener);

        // assert
        let result = sut.get_items();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, String::from("random_item"));
    }

    #[test]
    fn test_get_stock_success() {
        // prepare
        let mock_event_listener = MockEventBus::new();
        let mut mock_catalog_db = MockCatalogDb::new();
        let item = generate_random_item(1, 33);
        mock_catalog_db.set_expected_get_item(Some(item));

        // act
        let sut = CatalogService::new(mock_catalog_db, mock_event_listener);

        // assert
        let result = sut.get_stock(1);
        assert_eq!(result.unwrap(), 33);
    }

    #[test]
    fn test_get_stock_item_not_found() {
        // prepare
        let mock_event_listener = MockEventBus::new();
        let mut mock_catalog_db = MockCatalogDb::new();
        mock_catalog_db.set_expected_get_item(None);

        // act
        let sut = CatalogService::new(mock_catalog_db, mock_event_listener);

        // assert
        let result = sut.get_stock(1);
        assert_eq!(result.is_err(), true);
    }

    #[tokio::test]
    async fn test_start_event_listeners() {
        let mock_event_listener = MockEventBus::new();
        let mut mock_catalog_db = MockCatalogDb::new();
        mock_catalog_db.set_expected_get_item(None);

        // act
        let mut sut = CatalogService::new(mock_catalog_db, mock_event_listener);

        // assert
        sut.start_event_listeners();
    }
}
