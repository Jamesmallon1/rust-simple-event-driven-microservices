use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderPlacedEvent {
    pub item_id: u32,
    pub quantity: u32,
}
