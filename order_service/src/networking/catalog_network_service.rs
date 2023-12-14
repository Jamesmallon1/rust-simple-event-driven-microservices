use async_trait::async_trait;
use networking::NetworkError;

/// A client for interacting with the Catalog Microservice.
///
/// This client provides network operations to communicate with the
/// Catalog Microservice, handling tasks such as retrieving stock information.
///
/// # Fields
/// - `host`: The base URL or host address of the Catalog Microservice.
///
/// # Examples
///
/// ```
/// let api_client = CatalogApiClient {
///     host: "http://localhost:3000/".to_string(),
/// };
/// ```
pub struct CatalogApiClient {
    pub host: String,
}

/// Defines network service operations for interacting with the Catalog Microservice.
#[async_trait]
pub trait CatalogNetworkService {
    /// Asynchronously retrieves the amount of stock available for a specific clothing item.
    ///
    /// This method queries the Catalog Microservice to obtain the current stock
    /// level for the item specified by `item_id`.
    ///
    /// # Arguments
    ///
    /// * `item_id` - A unique identifier for the clothing item.
    ///
    /// # Returns
    ///
    /// Returns a `Result` which, on success, contains the stock amount (`u32`)
    /// of the specified item. On failure, returns a `NetworkError`.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn run() -> Result<(), NetworkError> {
    /// let api_client = CatalogApiClient {
    ///     host: "http://localhost:3000/".to_string(),
    /// };
    /// let stock = api_client.get_stock(123).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn get_stock(&self, item_id: u32) -> Result<u32, NetworkError>;
}

#[async_trait]
impl CatalogNetworkService for CatalogApiClient {
    async fn get_stock(&self, item_id: u32) -> Result<u32, NetworkError> {
        let url = self.host.clone() + &format!("/catalog/stock/{item_id}");
        return match networking::execute_get_request::<u32>(&url, None, None).await {
            Ok(response_data) => Ok(response_data),
            Err(e) => Err(e),
        };
    }
}
