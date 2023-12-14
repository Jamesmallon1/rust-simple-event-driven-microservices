use serde::Deserialize;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Deserialize)]
pub struct OrderRequest {
    pub item_id: u32,
    pub name: String,
    pub address: String,
    pub quantity: u32,
}

impl Display for OrderRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OrderReq = ItemId: {}, Quantity: {}", self.item_id, self.quantity)
    }
}
