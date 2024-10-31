// api.rs

use actix_web::{get, post, delete, put, web, HttpResponse, Responder};
use serde_json::json;
use crate::models::data_structures::{Bone, Boon};

// Route to get all Boons
#[get("/boons")]
async fn get_boons(data: web::Data<Bone>) -> impl Responder {
    let boons: Vec<Boon> = data
        .iter()
        .flat_map(|entry| entry.value().bane.iter().map(|b| b.value().clone()).collect::<Vec<Boon>>()) // Collect inner iterator here
        .collect();
    HttpResponse::Ok().json(boons)
}

// Route to get a specific Boon by ID
#[get("/boons/{id}")]
async fn get_boon_by_id(data: web::Data<Bone>, boon_id: web::Path<i64>) -> impl Responder {
    let boon_id = *boon_id;
    for honk in data.iter() {
        if let Some(boon) = honk.value().bane.get(&boon_id) {
            return HttpResponse::Ok().json(boon.value().clone());
        }
    }
    HttpResponse::NotFound().json(json!({ "error": "Boon not found" }))
}

// Route to add a new Boon
#[post("/boons")]
async fn create_boon(data: web::Data<Bone>, boon: web::Json<Boon>) -> impl Responder {
    let boon = boon.into_inner();
    for honk in data.iter() {
        honk.value().bane.insert(boon.id, boon.clone());
    }
    HttpResponse::Created().json(boon)
}

// Route to update an existing Boon by ID
#[put("/boons/{id}")]
async fn update_boon(data: web::Data<Bone>, boon_id: web::Path<i64>, boon: web::Json<Boon>) -> impl Responder {
    let boon_id = *boon_id;
    let boon = boon.into_inner();
    for honk in data.iter() {
        if honk.value().bane.contains_key(&boon_id) {
            honk.value().bane.insert(boon_id, boon.clone());
            return HttpResponse::Ok().json(boon);
        }
    }
    HttpResponse::NotFound().json(json!({ "error": "Boon not found" }))
}

// Route to delete a Boon by ID
#[delete("/boons/{id}")]
async fn delete_boon(data: web::Data<Bone>, boon_id: web::Path<i64>) -> impl Responder {
    let boon_id = *boon_id;
    for honk in data.iter() {
        if honk.value().bane.remove(&boon_id).is_some() {
            return HttpResponse::NoContent().finish();
        }
    }
    HttpResponse::NotFound().json(json!({ "error": "Boon not found" }))
}

// Route to get all active Honks
#[get("/honks")]
async fn get_honks(data: web::Data<Bone>) -> impl Responder {
    let honk_ids: Vec<String> = data.iter().map(|entry| entry.key().clone()).collect();
    HttpResponse::Ok().json(honk_ids)
}

// Route to get specific Honk details by ID
#[get("/honks/{id}")]
async fn get_honk_by_id(data: web::Data<Bone>, honk_id: web::Path<String>) -> impl Responder {
    let honk_id = honk_id.into_inner();
    if let Some(honk) = data.get(&honk_id) {
        let honk_info = json!({
            "active_bonks": honk.bonks.len(),
            "room_info": honk.room_info.lock().await.clone(),
        });
        return HttpResponse::Ok().json(honk_info);
    }
    HttpResponse::NotFound().json(json!({ "error": "Honk not found" }))
}

// Initialize API routes
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_boons);
    cfg.service(get_boon_by_id);
    cfg.service(create_boon);
    cfg.service(update_boon);
    cfg.service(delete_boon);
    cfg.service(get_honks);
    cfg.service(get_honk_by_id);
}
