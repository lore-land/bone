// routes/websocket.rs

use actix_web::{web, HttpRequest, Responder};
use actix_ws::{Message, CloseReason};
use bytes::Bytes;
use dashmap::DashMap;
use futures_util::StreamExt;
use std::sync::Arc;
use regex::RegexSet;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use uuid::Uuid;
use tracing::{error, info};
use tokio_util::sync::CancellationToken;

// Data Structures
// Import the data structures
use crate::models::data_structures::*;

// Main handler for new WebSocket connections
pub async fn lore_exchange(
    req: HttpRequest,
    stream: web::Payload,
    boof_path: web::Path<String>,
    data: web::Data<Bone>,
) -> actix_web::Result<impl Responder> {
    let boof_path = boof_path.into_inner();
    let (response, bonk, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let bonk_id = Uuid::new_v4();
    let bone = data.get_ref().clone();

    let honk = bone
        .entry(boof_path.clone())
        .or_insert_with(|| Arc::new(Honk::new()))
        .clone();

    honk.bonks.insert(bonk_id, bonk);

    // Update room size and count of active bonks
    {
        let mut room_info = honk.room_info.lock().await;
        room_info.active_bonks = honk.bonks.len(); // Update active bonks count
    }

    // Send existing Boons to the new client
    if let Err(e) = send_bane_to_bonk(&honk, &bonk_id).await {
        error!("Failed to send bane to client: {}", e);
    }

    // Handle incoming messages
    let honk_clone = honk.clone();
    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            if let Err(e) = handle_message(&honk_clone, &bonk_id, msg).await {
                error!("Error handling message: {}", e);
                break;
            }
        }

        // Remove Bonk when disconnected
        honk_clone.bonks.remove(&bonk_id);
        {
            let mut room_info = honk_clone.room_info.lock().await;
            room_info.active_bonks = honk_clone.bonks.len(); // Update active bonks count
        }

        if honk_clone.bonks.is_empty() {
            // Signal cancellation
            honk_clone.cancellation_token.cancel();

            bone.remove(&boof_path);
        }
    });

    Ok(response)
}

// Update room info
impl Honk {
    fn new() -> Self {
        // Example regex patterns to match Boof messages
        let patterns = vec![
            r"^ERROR:.*",
            r"^WARN:.*",
            r"^INFO:.*",
        ];
        let regex_set = RegexSet::new(&patterns).unwrap();

        Self {
            bane: DashMap::new(),
            bonks: DashMap::new(),
            room_info: Mutex::new(RoomInfo {
                width: 800.0,  // Example width
                height: 600.0, // Example height
                active_bonks: 0,
                focal_point: None,
                focal_range: None,
            }),
            cancellation_token: CancellationToken::new(),
            boof_queue: Mutex::new(Vec::new()),
            regex_set,
        }
    }
}

// Send the existing Bane to the specified Bonk
async fn send_bane_to_bonk(honk: &Arc<Honk>, bonk_id: &Uuid) -> BoneHonk<()> {
    let bane: Vec<Boon> = honk.bane.iter().map(|entry| entry.value().clone()).collect();
    let bane_json = serde_json::to_string(&bane)?;

    if let Some(bonk) = honk.bonks.get(bonk_id) {
        bonk.clone().text(bane_json).await?;
    }

    Ok(())
}

// Broadcast a boon to all connected Bonks
async fn broadcast_boon_to_bonks(honk: &Arc<Honk>, boon_json: String) -> BoneHonk<()> {
    for mut bonk_entry in honk.bonks.iter_mut() {
        if let Err(e) = bonk_entry.value_mut().text(boon_json.clone()).await {
            error!("Failed to send boon to bonk {}: {}", bonk_entry.key(), e);
            honk.bonks.remove(bonk_entry.key());
        }
    }
    Ok(())
}

// Broadcast a boof message to all connected Bonks
async fn broadcast_boof_to_bonks(honk: &Arc<Honk>, boof_json: String) -> BoneHonk<()> {
    for mut bonk_entry in honk.bonks.iter_mut() {
        if let Err(e) = bonk_entry.value_mut().text(boof_json.clone()).await {
            error!("Failed to send boof to bonk {}: {}", bonk_entry.key(), e);
            honk.bonks.remove(bonk_entry.key());
        }
    }
    Ok(())
}

// Handle messages from the client
async fn handle_message(honk: &Arc<Honk>, bonk_id: &Uuid, msg: Message) -> BoneHonk<()> {
    match msg {
        Message::Ping(bytes) => handle_ping(honk, bonk_id, bytes).await,
        Message::Text(text) => handle_text(honk, text.to_string()).await,
        Message::Binary(data) => handle_binary(honk, data).await,
        Message::Close(reason) => {
            handle_close(honk, bonk_id, reason).await?;
            Err(anyhow::anyhow!("Connection closed"))
        }
        _ => Ok(()),
    }
}

// Handle a Ping message
async fn handle_ping(honk: &Arc<Honk>, bonk_id: &Uuid, bytes: Bytes) -> BoneHonk<()> {
    if let Some(mut bonk) = honk.bonks.get_mut(bonk_id) {
        bonk.pong(&bytes).await?;
    }
    Ok(())
}

// Handle a Text message containing a Boon or Boof
async fn handle_text(honk: &Arc<Honk>, text: String) -> BoneHonk<()> {
    // Handle focal point message
    if let Ok(focal_point_msg) = serde_json::from_str::<FocalPointMessage>(&text) {
        let mut room_info = honk.room_info.lock().await;
        room_info.focal_point = Some((focal_point_msg.position.x, focal_point_msg.position.y));
        room_info.focal_range = Some((
            focal_point_msg.bounds.x1,
            focal_point_msg.bounds.y1,
            focal_point_msg.bounds.x2,
            focal_point_msg.bounds.y2,
        ));
        info!("Focal point updated: {:?}", room_info.focal_point);
        return Ok(());
    }

    // Try to parse as Boon
    if let Ok(boon) = serde_json::from_str::<Boon>(&text) {
        // Update or add the Boon to the Bane
        honk.bane.insert(boon.id, boon.clone());
        let boon_json = serde_json::to_string(&boon)?;
        broadcast_boon_to_bonks(honk, boon_json).await?;
        return Ok(());
    }

    // Otherwise, treat as Boof message
    let boof = Boof { content: text.clone() };
    let boof_content = &boof.content;

    // Apply regex patterns to sort the Boof into a queue
    let matches: Vec<_> = honk.regex_set.matches(boof_content).into_iter().collect();

    if !matches.is_empty() {
        let mut boof_queue = honk.boof_queue.lock().await;
        boof_queue.push(boof.clone());
        info!("Boof message matched patterns and was added to the queue.");
    } else {
        info!("Boof message did not match any patterns.");
    }

    // Broadcast the Boof message to all connected clients
    let boof_json = serde_json::to_string(&boof)?;
    broadcast_boof_to_bonks(honk, boof_json).await?;

    Ok(())
}

// Handle a Binary message containing image data
async fn handle_binary(honk: &Arc<Honk>, data: Bytes) -> BoneHonk<()> {
    let image_id = Uuid::new_v4().to_string();
    let image_dir = std::env::var("IMAGE_STORAGE_PATH").unwrap_or_else(|_| "./images".to_string());
    let image_path = format!("{}/{}.png", image_dir, image_id);

    fs::create_dir_all(&image_dir).await?;
    let mut file = File::create(&image_path).await?;
    file.write_all(&data).await?;

    let message = serde_json::json!({ "image_id": image_id }).to_string();
    broadcast_boon_to_bonks(honk, message).await
}

// Handle a Close message
async fn handle_close(honk: &Arc<Honk>, bonk_id: &Uuid, reason: Option<CloseReason>) -> BoneHonk<()> {
    if let Some(bonk) = honk.bonks.get_mut(bonk_id) {
        bonk.clone().close(reason).await?;
    }
    Ok(())
}
