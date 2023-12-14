use log::error;
use rdkafka::consumer::StreamConsumer;
use rdkafka::message::Message;
use serde::de::DeserializeOwned;
use serde_json;
use tokio::sync::broadcast;

/// A Kafka listener that asynchronously listens to messages from a Kafka topic and broadcasts them.
///
/// This struct wraps a Tokio broadcast channel sender to allow multiple parts of your application
/// to receive messages concurrently. It listens to a Kafka topic, deserializes each message into type `T`,
/// and then sends it across the broadcast channel.
///
/// # Type Parameters
///
/// * `T`: The type of the message payload. Must be deserializable from JSON, cloneable, and safe to send across threads.
///
/// # Fields
///
/// * `tx`: The broadcast channel sender used to send messages to receivers.
pub struct KafkaListener<T>
where
    T: DeserializeOwned + Send + 'static,
{
    tx: broadcast::Sender<T>,
}

impl<T> KafkaListener<T>
where
    T: DeserializeOwned + Send + 'static + Clone,
{
    /// Creates a new `KafkaListener`.
    ///
    /// Initializes a Tokio broadcast channel and spawns an asynchronous task that listens to messages from a Kafka topic.
    /// Each message is deserialized into type `T` and sent across the broadcast channel to all subscribed receivers.
    ///
    /// # Arguments
    ///
    /// * `consumer`: The Kafka `StreamConsumer` to listen for messages.
    /// * `buffer_size`: The size of the broadcast channel buffer.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `KafkaListener<T>`.
    ///
    /// # Panics
    ///
    /// Panics if there is a JSON parsing error for the Kafka messages, or if the broadcast channel's sender fails.
    pub fn new(consumer: StreamConsumer, buffer_size: usize) -> Self {
        let (tx, _) = broadcast::channel::<T>(buffer_size);

        // safe to clone as channel is retained, only handler is different
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            loop {
                match consumer.recv().await {
                    Ok(borrowed_message) => {
                        if let Some(payload) = borrowed_message.payload() {
                            match serde_json::from_slice::<T>(payload) {
                                Ok(parsed_message) => {
                                    if tx_clone.send(parsed_message).is_err() {
                                        error!("Could not send message across the broadcast channel");
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("JSON parsing error: {:?}", e);
                                    panic!("Could not parse the kafka message");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("A Kafka error occurred: {:?}", e);
                    }
                }
            }
        });

        KafkaListener { tx }
    }

    /// Retrieves a receiver for the broadcast channel.
    ///
    /// This method allows multiple parts of the application to concurrently receive messages broadcast by the `KafkaListener`.
    ///
    /// # Returns
    ///
    /// Returns a `broadcast::Receiver<T>`, which can be used to receive messages broadcast by the Kafka listener.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `listener` is an instance of `KafkaListener<MyType>`
    /// let receiver = listener.get_receiver();
    /// // Use `receiver` to asynchronously receive messages of type `MyType`
    /// ```
    pub fn get_receiver(&self) -> broadcast::Receiver<T> {
        self.tx.subscribe()
    }

    // mock method necessary for testing
    pub fn mock() -> Self {
        let (tx, _) = broadcast::channel::<T>(1);
        KafkaListener { tx }
    }
}
