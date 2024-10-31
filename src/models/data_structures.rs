use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use regex::RegexSet;

#[derive(Serialize, Deserialize, Clone)]
pub struct Boonhonk {
    pub description: String,
    pub level: i64,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Boon {
    pub id: i64,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub boonhonk: Boonhonk,
    pub image_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomInfo {
    pub width: f64,
    pub height: f64,
    pub active_bonks: usize,
    pub focal_point: Option<(f64, f64)>,
    pub focal_range: Option<(f64, f64, f64, f64)>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FocalPointMessage {
    pub r#type: String, // The type of the message, e.g., "focal-point"
    pub position: Position, // Position struct containing the x and y coordinates
    pub bounds: Bounds, // Bounds struct containing the rectangle coordinates
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Position {
    pub x: f64, // x coordinate of the focal point
    pub y: f64, // y coordinate of the focal point
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Bounds {
    pub x1: f64, // x coordinate for the top-left corner of the focal range
    pub y1: f64, // y coordinate for the top-left corner of the focal range
    pub x2: f64, // x coordinate for the bottom-right corner of the focal range
    pub y2: f64, // y coordinate for the bottom-right corner of the focal range
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Boof {
    pub content: String,
}

pub type Bone = Arc<DashMap<String, Arc<Honk>>>;

// Honk structure
pub struct Honk {
    pub bane: DashMap<i64, Boon>,
    pub bonks: DashMap<Uuid, actix_ws::Session>,
    pub room_info: Mutex<RoomInfo>,
    pub cancellation_token: CancellationToken,
    pub boof_queue: Mutex<Vec<Boof>>,
    pub regex_set: RegexSet,
}

// Custom result type
pub type BoneHonk<T> = anyhow::Result<T>;
