pub trait ListenerService {
    /// Listens to relevant topics and reacts to possible events
    /// received from other services.
    fn start_event_listeners(&mut self);
}
