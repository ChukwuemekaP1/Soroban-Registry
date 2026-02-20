// backend/api/src/incident_handlers.rs
// Handlers for disaster recovery incidents

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    error::ApiResult,
    state::AppState,
};

#[derive(Serialize)]
pub struct Incident {
    pub id: Uuid,
    pub contract_id: Option<Uuid>,
    pub incident_type: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub rto_achieved: Option<String>, // as interval string
    pub rpo_achieved: Option<String>,
    pub lessons_learned: Option<String>,
    pub notified_users: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateIncidentRequest {
    pub contract_id: Option<Uuid>,
    pub incident_type: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct UpdateIncidentRequest {
    pub end_time: Option<DateTime<Utc>>,
    pub rto_achieved: Option<String>,
    pub rpo_achieved: Option<String>,
    pub lessons_learned: Option<String>,
    pub notified_users: Option<bool>,
}

pub async fn create_incident(
    State(state): State<AppState>,
    Json(req): Json<CreateIncidentRequest>,
) -> ApiResult<Json<Incident>> {
    let incident: Incident = sqlx::query_as!(
        Incident,
        r#"
        INSERT INTO incidents (contract_id, incident_type, description, start_time)
        VALUES ($1, $2, $3, $4)
        RETURNING id, contract_id, incident_type, description, start_time, end_time,
                  rto_achieved, rpo_achieved, lessons_learned, notified_users, created_at, updated_at
        "#,
        req.contract_id,
        req.incident_type,
        req.description,
        req.start_time,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| crate::handlers::db_internal_error("create_incident", e))?;

    // Log for notification (assume external system handles actual notifications)
    tracing::info!("Incident created: {} - {}", incident.incident_type, incident.description);

    Ok(Json(incident))
}

pub async fn update_incident(
    State(state): State<AppState>,
    Path(incident_id): Path<Uuid>,
    Json(req): Json<UpdateIncidentRequest>,
) -> ApiResult<Json<Incident>> {
    let incident: Incident = sqlx::query_as!(
        Incident,
        r#"
        UPDATE incidents
        SET end_time = $1, rto_achieved = $2, rpo_achieved = $3, lessons_learned = $4, notified_users = COALESCE($5, notified_users)
        WHERE id = $6
        RETURNING id, contract_id, incident_type, description, start_time, end_time,
                  rto_achieved, rpo_achieved, lessons_learned, notified_users, created_at, updated_at
        "#,
        req.end_time,
        req.rto_achieved,
        req.rpo_achieved,
        req.lessons_learned,
        req.notified_users,
        incident_id,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| crate::handlers::db_internal_error("update_incident", e))?;

    Ok(Json(incident))
}

pub async fn list_incidents(
    State(state): State<AppState>,
    Query(params): Query<ListIncidentsQuery>,
) -> ApiResult<Json<Vec<Incident>>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let incidents: Vec<Incident> = sqlx::query_as!(
        Incident,
        r#"
        SELECT id, contract_id, incident_type, description, start_time, end_time,
               rto_achieved, rpo_achieved, lessons_learned, notified_users, created_at, updated_at
        FROM incidents
        ORDER BY start_time DESC
        LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| crate::handlers::db_internal_error("list_incidents", e))?;

    Ok(Json(incidents))
}

#[derive(Deserialize)]
pub struct ListIncidentsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}