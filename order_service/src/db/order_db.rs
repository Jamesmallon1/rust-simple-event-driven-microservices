use crate::model::OrderRequest;
use std::collections::HashMap;

/// `OrderDbClient` is a mock database structure used for simulating
/// a order database in a testing or development environment.
///
/// This struct provides functionalities to add and retrieve orders,
/// using a HashMap to store them. Each order is associated with a unique
/// order ID, which is automatically incremented for each new order.
///
/// # Fields
/// - `latest_order_id`: The ID to be assigned to the next added order.
/// - `orders`: A HashMap storing orders with their corresponding order ID as the key.
pub struct OrderDbClient {
    latest_order_id: u32,
    orders: HashMap<u32, Order>,
}

// cannot mock trait automatically due to explicit lifetimes use manual mocking in tests
pub trait OrderDb<'a> {
    /// Creates a new instance of the implementing type.
    ///
    /// This method initializes a new order database client or similar
    /// structure, setting up any necessary internal data structures like a HashMap.
    ///
    /// # Returns
    /// Returns a new instance of the implementor of the `OrderDb` trait.
    ///
    /// # Examples
    /// ```
    /// use your_crate::OrderDb;
    /// use your_crate::OrderDbClient;
    ///
    /// let db_client = OrderDbClient::new();
    /// ```
    fn new() -> Self;

    /// Adds a new order to the database.
    ///
    /// This method takes an `OrderRequest` and creates a new `Order` object,
    /// assigning it a unique order ID before storing it in the database.
    ///
    /// # Arguments
    /// * `order_request` - The details of the order to be added.
    ///
    /// # Examples
    /// ```
    /// use your_crate::{OrderDb, OrderDbClient, model::OrderRequest};
    ///
    /// let mut db_client = OrderDbClient::new();
    /// let order_request = OrderRequest { /* ... */ };
    /// db_client.add_order(order_request);
    /// ```
    fn add_order(&mut self, order_request: OrderRequest);

    /// Retrieves an order by its ID.
    ///
    /// Given an `order_id`, this method looks up and returns a reference to the
    /// corresponding `Order` in the database, if it exists.
    ///
    /// # Arguments
    /// * `order_id` - The unique identifier of the order to retrieve.
    ///
    /// # Returns
    /// Returns an `Option<&'a Order>`. If an order with the given ID exists,
    /// it returns `Some(&Order)`, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// use your_crate::{OrderDb, OrderDbClient};
    ///
    /// let mut db_client = OrderDbClient::new();
    /// // Assuming an order with ID 1 has been added...
    /// let order = db_client.get_order(1);
    /// ```
    fn get_order(&'a self, order_id: u32) -> Option<&'a Order>;
}

impl<'a> OrderDb<'a> for OrderDbClient {
    fn new() -> Self {
        OrderDbClient {
            latest_order_id: 0,
            orders: HashMap::new(),
        }
    }

    fn add_order(&mut self, order_request: OrderRequest) {
        self.latest_order_id += 1;
        let order = Order::new(self.latest_order_id, order_request);
        self.orders.insert(order.order_id, order);
    }

    fn get_order(&'a self, order_id: u32) -> Option<&'a Order> {
        self.orders.get(&order_id)
    }
}

/// Represents an order in the order database.
///
/// This struct encapsulates the details of an order, including its ID,
/// the ID of the item ordered, the name of the customer, and the delivery address.
///
/// # Fields
/// - `order_id`: A unique identifier for the order.
/// - `item_id`: The ID of the item ordered.
/// - `name`: The name of the customer who placed the order.
/// - `address`: The delivery address for the order.
///
/// # Examples
///
/// ```
/// use your_crate::model::Order;
///
/// let order = Order::new(1, /* OrderRequest instance */);
/// ```
#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: u32,
    pub item_id: u32,
    pub name: String,
    pub address: String,
}

impl Order {
    pub fn new(order_id: u32, order_request: OrderRequest) -> Self {
        Order {
            order_id,
            item_id: order_request.item_id,
            name: order_request.name,
            address: order_request.address,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn produce_fake_order_request() -> OrderRequest {
        OrderRequest {
            item_id: 123,
            name: "James".to_string(),
            address: "23 Bugs Bunny Street, London, E1 4AH".to_string(),
            quantity: 5,
        }
    }

    #[test]
    fn test_new_order_db_client() {
        // act
        let client = OrderDbClient::new();

        // assert
        assert_eq!(client.latest_order_id, 0);
        assert!(client.orders.is_empty());
    }

    #[test]
    fn test_add_order() {
        // prepare
        let mut client = OrderDbClient::new();
        let order_request = produce_fake_order_request();

        // act
        client.add_order(order_request.clone());

        // assert
        assert_eq!(client.latest_order_id, 1);
        assert_eq!(client.orders.len(), 1);
        assert!(client.orders.contains_key(&1));
    }

    #[test]
    fn test_get_order() {
        // prepare
        let mut client = OrderDbClient::new();
        let order_request = produce_fake_order_request();
        client.add_order(order_request);

        // act
        let order = client.get_order(1);
        let non_existent_order = client.get_order(2);

        // assert
        assert!(order.is_some());
        assert!(non_existent_order.is_none());
    }
}
