use crate::{
    model::NoteModel,
    schema::{CreateNoteSchema, FilterOptions, EditNoteSchema},
    AppState,
};
use actix_web::{delete, get, patch, post, web::{Query,Data,Path,Json}, HttpResponse};
use chrono::prelude::Utc;
use serde_json::json;
use uuid::Uuid;

#[get("/notes")]
async fn note_list_handler(opts: Query<FilterOptions>, data: Data<AppState>) -> HttpResponse {
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    match sqlx::query_as!(
        NoteModel,
        "SELECT * FROM notes ORDER BY id LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await
    {
        Ok(notes) => HttpResponse::Ok().json(json!({
            "status": "success",
            "results": notes.len(),
            "data": notes
        })),
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch notes."
        })),
    }
}

#[get("/notes/{id}")]
async fn get_note_handler(id: Path<uuid::Uuid>, data: Data<AppState>) -> HttpResponse {
    match sqlx::query_as!(
        NoteModel,
        "SELECT * FROM notes WHERE id = $1",
        id.into_inner()
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(note) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": { "note": note }
        })),
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": "Failed to fetch note."
        })),
    }
}

#[post("/notes")]
async fn create_note_handler(body: Json<CreateNoteSchema>, data: Data<AppState>,) -> HttpResponse {
    match sqlx::query_as!(
        NoteModel,
        "INSERT INTO notes (title, content, category) VALUES ($1, $2, $3) RETURNING *",
        body.title,
        body.content,
        body.category.as_deref().unwrap_or("")
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(note) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": { "note": note }
        })),
        Err(e) if e.to_string().contains("duplicate key") => {
            HttpResponse::BadRequest().json(json!({
                "status": "fail",
                "message": "Note with that title already exists"
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": format!("{}", e)
        })),
    }
}

#[patch("/notes/{id}")]
async fn edit_note_handler(id: Path<Uuid>, body: Json<EditNoteSchema>, data: Data<AppState>) -> HttpResponse {
    match sqlx::query_as!(
        NoteModel, 
        "UPDATE notes SET title = $1, content = $2, category = $3, published = $4, updated_at = $5 WHERE id = $6 RETURNING *", 
        body.title, 
        body.content, 
        body.category.as_deref().unwrap_or(""),
        body.published,
        Utc::now(),
        id.into_inner(),
    )
    .fetch_one(&data.db)
    .await
    {
        Ok(note) => HttpResponse::Ok().json(json!({
            "status": "success", 
            "data": { "note": note } 
        })), 
        Err(_) => HttpResponse::InternalServerError().json(json!({
            "status": "error", 
            "message": "Failed to update note." 
        })),
    }
}

#[delete("/notes/{id}")]
async fn delete_note_handler(id: Path<Uuid>, data: Data<AppState>) -> HttpResponse {
    let rows_deleted = sqlx::query!("DELETE FROM notes WHERE id = $1", id.into_inner())
        .execute(&data.db)
        .await
        .map(|result| result.rows_affected())
        .unwrap_or(0);  // Возвращаем 0, если произошла ошибка выполнения запроса

    if rows_deleted == 0 {
        return HttpResponse::NotFound().json(json!({"status": "fail", "message": format!("Note with ID: not found")}));
    }

    HttpResponse::NoContent().finish()
}