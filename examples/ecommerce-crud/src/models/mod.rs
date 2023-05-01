use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub image_path: String,
    pub price: Decimal,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cart {
    pub id: Uuid,
    pub user_id: Uuid,
    pub items: Vec<Cart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub id: Uuid,
    pub quantity: u32,
    pub cart_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub items: Vec<OrderItem>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub id: Uuid,
    pub item_id: Uuid,
    pub order_id: Uuid,
    pub buy_price: Decimal,
    pub quantity: u32,
}
