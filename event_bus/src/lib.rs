use crate::utilities::listeners;
use crate::utilities::listeners::KafkaListener;
use async_trait::async_trait;
use log::{error, info};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::error::KafkaError;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;
use std::time::Duration;

pub mod event;
pub mod events;
pub mod topic;
pub mod utilities;

pub struct EventBus {
    broker: String,
    producer: FutureProducer,
}

pub trait EventListener {
    /// Creates a new `KafkaListener` for the specified consumer group and topics.
    ///
    /// # Important Note
    ///
    /// You should only produce a KafkaListener when you are only listening to a single topic from a microservice.
    ///
    /// This function sets up a Kafka consumer and wraps it in a `KafkaListener` to facilitate
    /// asynchronous message handling. The `KafkaListener` will use a `StreamConsumer` to
    /// subscribe to the given topics and listen for messages of type `T`, which is determined
    /// by the caller. The messages received will be deserialized from JSON into type `T`.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type into which the JSON messages from Kafka will be deserialized.
    ///        `T` must implement the `serde::de::DeserializeOwned` trait.
    ///
    /// # Arguments
    ///
    /// * `group_id`: The consumer group ID to be used by the Kafka consumer.
    /// * `topics`: A slice of topic names to which the consumer should subscribe.
    ///
    /// # Returns
    ///
    /// Returns a `Result` which is `Ok` containing the `KafkaListener<T>` upon successful
    /// creation and configuration, or a `KafkaError` if an error occurs during the creation
    /// of the consumer or the listener.
    ///
    /// # Errors
    ///
    /// Returns `KafkaError` if there's an issue creating the `StreamConsumer`, or if
    /// there's a problem subscribing to the specified topics.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `service` is an instance with `create_listener` method.
    /// use event_bus::EventBus;
    /// let group_id = "my_consumer_group";
    /// let topics = ["my_topic"];
    ///
    /// match EventBus.create_event_listener::<MyMessageType>(group_id, &topics) {
    ///     Ok(listener) => {
    ///         // Use the listener here
    ///     }
    ///     Err(e) => eprintln!("Failed to create listener: {}", e),
    /// }
    /// ```
    fn create_event_listener<T>(
        &self,
        group_id: &str,
        topics: &[&str],
    ) -> Result<listeners::KafkaListener<T>, Box<dyn Error>>
    where
        T: Send + DeserializeOwned + 'static + Clone;
}

#[async_trait]
pub trait EventProducer {
    /// Broadcasts an event to a specified Kafka topic.
    ///
    /// This function serializes the given payload into a JSON string and sends it
    /// to the specified Kafka topic using the `produce` method. The payload must
    /// implement the `serde::Serialize` trait to enable serialization.
    ///
    /// # Type Parameters
    ///
    /// * `T`: The type of the payload to be broadcast. Must implement `serde::Serialize`.
    ///
    /// # Arguments
    ///
    /// * `payload`: The payload of the event, which will be serialized to JSON.
    /// * `topic_name`: The name of the Kafka topic to which the event will be sent.
    /// * `key`: A key associated with the event, used by Kafka for partitioning.
    /// * `source`: The source identifier of the event.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful broadcast of the event.
    /// Returns `Err(Box<dyn Error>)` if there is an error in serializing the payload
    /// or in sending the message via Kafka.
    ///
    /// # Errors
    ///
    /// This function can return errors in the following cases:
    /// - If serialization of the payload to JSON fails.
    /// - If sending the message through Kafka encounters an error.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(serde::Serialize)]
    /// struct MyPayload {
    ///     data: String,
    /// }
    ///
    /// let payload = MyPayload { data: "example data".to_string() };
    /// let topic = "my_topic";
    /// let key = "event_key";
    /// let source = "my_source";
    ///
    /// match event_bus.broadcast_event(payload, topic, key, source).await {
    ///     Ok(_) => println!("Event broadcasted successfully"),
    ///     Err(e) => eprintln!("Failed to broadcast event: {:?}", e),
    /// }
    /// ```
    async fn broadcast_event<T: serde::Serialize + Send>(
        &self,
        payload: T,
        topic_name: &str,
        key: &str,
    ) -> Result<(), Box<dyn Error>>;
}

impl EventListener for EventBus {
    fn create_event_listener<T>(
        &self,
        group_id: &str,
        topics: &[&str],
    ) -> Result<listeners::KafkaListener<T>, Box<dyn Error>>
    where
        T: Send + DeserializeOwned + 'static + Clone,
    {
        let consumer = self.create_consumer(group_id, topics).map_err(|e| Box::new(e) as Box<dyn Error>)?;
        Ok(listeners::KafkaListener::new(consumer, 100))
    }
}

#[async_trait]
impl EventProducer for EventBus {
    async fn broadcast_event<T: serde::Serialize + Send>(
        &self,
        payload: T,
        topic_name: &str,
        key: &str,
    ) -> Result<(), Box<dyn Error>> {
        // serialize the event object to JSON
        let message = serde_json::to_string(&payload).map_err(|e| {
            error!("Error serializing message: {:?}", e);
            Box::new(e) as Box<dyn Error>
        })?;

        // broadcast the event to kafka via our single producer
        self.produce(topic_name, &message, key).await.map_err(|e| {
            error!("Error sending message to Kafka: {:?}", e);
            Box::new(e) as Box<dyn Error>
        })
    }
}

impl EventBus {
    /// Creates a new instance of `EventBus`.
    ///
    /// This function initializes a new `EventBus` with a Kafka producer configured
    /// to connect to the provided Kafka broker. It also initializes an empty collection
    /// of Kafka consumers. The `EventBus` can be used to produce messages to Kafka
    /// topics and to create consumers for various topics.
    ///
    /// # Arguments
    ///
    /// * `broker` - A string slice that holds the reference to the broker's address.
    ///              This address is used to configure the Kafka producer.
    ///
    /// # Returns
    ///
    /// Returns an instance of `EventBus`.
    ///
    /// # Examples
    ///
    /// ```
    /// let event_bus = EventBus::new("localhost:9092");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the Kafka producer cannot be created, typically due to incorrect
    /// broker configuration or Kafka service unavailability.
    ///
    ///
    /// # Final Notes
    ///
    /// This implementation of an event bus is extremely simplistic and should not be used in production.
    /// The event bus only makes use of 1 broker which limits the scalability of kafka and can cause a bottleneck
    /// it also removes fault tolerance from the event bus system.
    ///
    /// Additionally, there is only a single producer in this event bus. You could improve the design by implementing
    /// a multiple producer pattern.
    pub fn new(broker: &str) -> Self {
        let producer: FutureProducer =
            ClientConfig::new().set("bootstrap.servers", broker).create().expect("Producer creation error");

        EventBus {
            broker: broker.to_string(),
            producer,
        }
    }

    // sends a raw message via kafka using the event bus' single producer
    async fn produce(&self, topic_name: &str, message: &str, key: &str) -> Result<(), KafkaError> {
        let record = FutureRecord::to(topic_name).payload(message).key(key);

        self.producer
            .send(record, Duration::from_secs(0))
            .await
            .map(|_| info!("Message with topic: {topic_name} and key: {key} sent successfully to Kafka"))
            .map_err(|(e, _)| {
                error!("Error sending message to Kafka: {:?}", e);
                e
            })
    }

    // creates and configures the raw kafka consumer
    fn create_consumer(&self, group_id: &str, topics: &[&str]) -> Result<StreamConsumer, KafkaError> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", &self.broker)
            .set("auto.offset.reset", "earliest")
            .create()?;

        consumer.subscribe(topics)?;
        Ok(consumer)
    }
}

pub struct MockEventBus {
    produces_error: bool,
}

impl EventListener for MockEventBus {
    #[allow(unused_variables)]
    fn create_event_listener<T>(
        &self,
        group_id: &str,
        topics: &[&str],
    ) -> Result<listeners::KafkaListener<T>, Box<dyn Error>>
    where
        T: Send + DeserializeOwned + 'static + Clone,
    {
        return if self.produces_error {
            Err(Box::new(KafkaError::Canceled) as Box<dyn Error>)
        } else {
            Ok(KafkaListener::mock())
        };
    }
}

#[async_trait]
impl EventProducer for MockEventBus {
    #[allow(unused_variables)]
    async fn broadcast_event<T: Serialize + Send>(
        &self,
        payload: T,
        topic_name: &str,
        key: &str,
    ) -> Result<(), Box<dyn Error>> {
        return if self.produces_error {
            Err(Box::new(KafkaError::Canceled) as Box<dyn Error>)
        } else {
            Ok(())
        };
    }
}

impl MockEventBus {
    pub fn new() -> Self {
        MockEventBus { produces_error: false }
    }

    pub fn set_produces_error(&mut self, does_produce_error: bool) {
        self.produces_error = does_produce_error;
    }
}
