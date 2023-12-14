use serde::{Deserialize, Serialize};
use std::collections;
use std::time::SystemTime;

/// Represents an event to be sent across an event bus in a microservices architecture.
///
/// This struct encapsulates all the necessary information for an event, including its type,
/// payload, source, and optional metadata and correlation ID for tracking and additional context.
///
/// # Type Parameters
///
/// * `T`: The type of the payload. This can be any type that is serializable and deserializable.
///
/// # Fields
///
/// * `event_type`: A `String` that specifies the type of the event. This is typically used
///   for routing and handling the event appropriately.
///
/// * `payload`: The actual data associated with the event. Its type `T` is generic and
///   can be any type that is serializable and deserializable.
///
/// * `timestamp`: A `SystemTime` value indicating when the event was created. Useful for
///   logging, debugging, and time-based processing.
///
/// * `source`: A `String` identifying the source of the event, such as the name of the
///   microservice or system component that generated it.
///
/// * `correlation_id`: An optional `String` used for correlating related events in a
///   distributed system. Useful for tracing and debugging complex flows.
///
/// * `metadata`: An optional `HashMap<String, String>` providing additional, free-form
///   metadata about the event. Can be used for adding any extra information that is
///   relevant to the event or its handling.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event<T> {
    pub event_type: String,
    pub payload: T,
    pub timestamp: SystemTime,
    pub source: String,
    pub correlation_id: Option<String>,
    pub metadata: Option<collections::HashMap<String, String>>,
}

impl<T> Event<T> {
    /// Creates a new `Event` with the specified properties.
    ///
    /// Instantiates an `Event` with a given type, payload, source, optional correlation ID, and
    /// optional metadata. The timestamp is set to the current system time.
    ///
    /// # Arguments
    ///
    /// * `event_type`: The type of the event.
    /// * `payload`: The payload of the event.
    /// * `source`: The source identifier of the event.
    /// * `correlation_id`: An optional correlation ID for the event.
    /// * `metadata`: Optional metadata for the event.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `Event<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// let event = Event::new(
    ///     "user_created".to_string(),
    ///     UserPayload { name: "John Doe".to_string(), age: 30 },
    ///     "user_service".to_string(),
    ///     Some("12345".to_string()),
    ///     None,
    /// );
    /// ```
    pub fn new(
        event_type: String,
        payload: T,
        source: String,
        correlation_id: Option<String>,
        metadata: Option<collections::HashMap<String, String>>,
    ) -> Self {
        Event {
            event_type,
            payload,
            timestamp: SystemTime::now(),
            source,
            correlation_id,
            metadata,
        }
    }
}
