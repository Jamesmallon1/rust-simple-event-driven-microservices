use log::info;
use std::collections::HashMap;

/// `CatalogDbClient` is a mock database structure used for simulating
/// a catalog database in a testing or development environment.
///
/// This struct is not intended for production use but rather for testing
/// purposes where a lightweight and simple database simulation is needed.
///
/// Fields:
/// - `items`: A hashmap of `ClothingItem` objects representing the items in the catalog.
pub struct CatalogDbClient {
    items: HashMap<u32, ClothingItem>,
}

// cannot mock trait automatically due to explicit lifetimes use manual mocking in tests
pub trait CatalogDb<'a> {
    /// Creates a new instance of the implementing type.
    ///
    /// This method should initialize the database, typically setting up
    /// an empty data structure or preloading it with mock data for testing.
    fn new() -> Self;

    /// Retrieves a mutable reference to a `ClothingItem` by its ID.
    ///
    /// This method allows for modifying a specific item in the catalog.
    ///
    /// Arguments:
    /// - `id`: The unique identifier of the clothing item.
    ///
    /// Returns:
    /// - `Option<&'a mut ClothingItem>`: A mutable reference to the clothing item if found, or `None` if not.
    fn get_mut_item(&'a mut self, id: u32) -> Option<&'a mut ClothingItem>;

    /// Retrieves an immutable reference to a `ClothingItem` by its ID.
    ///
    /// This method is used for accessing details of a specific item in the catalog without modifying it.
    ///
    /// Arguments:
    /// - `id`: The unique identifier of the clothing item.
    ///
    /// Returns:
    /// - `Option<&'a ClothingItem>`: An immutable reference to the clothing item if found, or `None` if not.
    fn get_item(&'a self, id: u32) -> Option<&'a ClothingItem>;

    /// Adds a new `ClothingItem` to the catalog.
    ///
    /// This method is used for inserting a new item into the catalog database.
    ///
    /// Arguments:
    /// - `item`: The `ClothingItem` to be added to the catalog.
    fn add_item(&mut self, item: ClothingItem);

    /// Retrieves the entire catalog as a vector of immutable references to `ClothingItem` objects.
    ///
    /// This method is used for accessing all items in the catalog.
    ///
    /// Returns:
    /// - `Vec<&'a ClothingItem>`: A vector containing immutable references to all the items in the catalog.
    fn get_catalog(&'a self) -> Vec<&'a ClothingItem>;
}

impl<'a> CatalogDb<'a> for CatalogDbClient {
    fn new() -> CatalogDbClient {
        let mut mock_db = CatalogDbClient { items: HashMap::new() };
        // as this is a mock db encapsulate all initialization within new
        let t_shirt = ClothingItem {
            id: 1,
            name: "T-Shirt".to_string(),
            description: "Comfortable cotton t-shirt, perfect for everyday wear.".to_string(),
            sizes: vec!["S".to_string(), "M".to_string(), "L".to_string(), "XL".to_string()],
            price: 20.00,
            stock: 100,
            images: vec![
                "https://example.com/t-shirt-front.jpg".to_string(),
                "https://example.com/t-shirt-back.jpg".to_string(),
            ],
            video: "https://example.com/t-shirt-video.mp4".to_string(),
        };
        mock_db.add_item(t_shirt);

        let jeans = ClothingItem {
            id: 2,
            name: "Jeans".to_string(),
            description: "Classic blue denim jeans, versatile and durable.".to_string(),
            sizes: vec!["30".to_string(), "32".to_string(), "34".to_string()],
            price: 40.00,
            stock: 50,
            images: vec![
                "https://example.com/jeans-front.jpg".to_string(),
                "https://example.com/jeans-back.jpg".to_string(),
            ],
            video: "https://example.com/jeans-video.mp4".to_string(),
        };
        mock_db.add_item(jeans);

        let jacket = ClothingItem {
            id: 3,
            name: "Jacket".to_string(),
            description: "Stylish and warm jacket, suitable for cold weather.".to_string(),
            sizes: vec!["M".to_string(), "L".to_string(), "XL".to_string()],
            price: 60.00,
            stock: 30,
            images: vec![
                "https://example.com/jacket-front.jpg".to_string(),
                "https://example.com/jacket-back.jpg".to_string(),
            ],
            video: "https://example.com/jacket-video.mp4".to_string(),
        };
        mock_db.add_item(jacket);

        let sneakers = ClothingItem {
            id: 4,
            name: "Sneakers".to_string(),
            description: "Trendy and comfortable sneakers for casual outings.".to_string(),
            sizes: vec!["8".to_string(), "9".to_string(), "10".to_string(), "11".to_string()],
            price: 50.00,
            stock: 75,
            images: vec![
                "https://example.com/sneakers-front.jpg".to_string(),
                "https://example.com/sneakers-side.jpg".to_string(),
            ],
            video: "https://example.com/sneakers-video.mp4".to_string(),
        };
        mock_db.add_item(sneakers);

        let cap = ClothingItem {
            id: 5,
            name: "Cap".to_string(),
            description: "Cool and stylish baseball cap, great for sunny days.".to_string(),
            sizes: vec!["One Size".to_string()],
            price: 15.00,
            stock: 1,
            images: vec![
                "https://example.com/cap-front.jpg".to_string(),
                "https://example.com/cap-back.jpg".to_string(),
            ],
            video: "https://example.com/cap-video.mp4".to_string(),
        };

        mock_db.add_item(cap);
        info!("Mock database has been initialized");
        mock_db
    }

    fn get_mut_item(&'a mut self, id: u32) -> Option<&'a mut ClothingItem> {
        self.items.get_mut(&id)
    }

    fn get_item(&'a self, id: u32) -> Option<&'a ClothingItem> {
        self.items.get(&id)
    }

    fn add_item(&mut self, item: ClothingItem) {
        self.items.insert(item.id, item);
    }

    fn get_catalog(&'a self) -> Vec<&'a ClothingItem> {
        self.items.values().collect()
    }
}

#[derive(Clone)]
pub struct ClothingItem {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub sizes: Vec<String>,
    pub price: f32,
    pub stock: u32,
    pub images: Vec<String>,
    pub video: String,
}

// mock db for testing
pub struct MockCatalogDb {
    expected_get_item: Option<ClothingItem>,
    expected_vec: Vec<ClothingItem>,
}

// mocks
impl MockCatalogDb {
    pub fn new() -> Self {
        MockCatalogDb {
            expected_get_item: None,
            expected_vec: vec![],
        }
    }

    pub fn set_expected_get_item(&mut self, item: Option<ClothingItem>) {
        self.expected_get_item = item;
    }

    pub fn set_expected_vec(&mut self, items: Vec<ClothingItem>) {
        self.expected_vec = items;
    }
}

impl<'a> CatalogDb<'a> for MockCatalogDb {
    fn new() -> Self {
        MockCatalogDb::new()
    }

    fn get_mut_item(&mut self, id: u32) -> Option<&mut ClothingItem> {
        self.expected_get_item.as_mut()
    }

    fn get_item(&self, id: u32) -> Option<&ClothingItem> {
        self.expected_get_item.as_ref()
    }

    fn add_item(&mut self, item: ClothingItem) {}

    fn get_catalog(&self) -> Vec<&ClothingItem> {
        self.expected_vec.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve_item() {
        let mut db = CatalogDbClient::new();
        let test_item = ClothingItem {
            id: 10,
            name: "Test Item".to_string(),
            description: "A test item".to_string(),
            sizes: vec!["M".to_string()],
            price: 10.00,
            stock: 20,
            images: vec!["https://example.com/test-item.jpg".to_string()],
            video: "https://example.com/test-item-video.mp4".to_string(),
        };

        db.add_item(test_item);

        let retrieved_item = db.get_item(10).unwrap();
        assert_eq!(retrieved_item.name, "Test Item");
        assert_eq!(retrieved_item.stock, 20);
    }

    #[test]
    fn test_get_non_existent_item() {
        let db = CatalogDbClient::new();
        assert!(db.get_item(100).is_none());
    }

    #[test]
    fn test_get_mut_item() {
        let mut db = CatalogDbClient::new();
        if let Some(item) = db.get_mut_item(1) {
            item.stock += 1;
        }
        assert_eq!(db.get_item(1).unwrap().stock, 101);
    }

    #[test]
    fn test_get_catalog() {
        let db = CatalogDbClient::new();
        let catalog = db.get_catalog();
        assert!(!catalog.is_empty());
        assert_eq!(catalog.len(), 5);
    }
}
