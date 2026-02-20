// backend/api/src/incident_routes.rs
// Routes for disaster recovery incidents

use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{incident_handlers, state::AppState};

pub fn incident_routes() -> Router<AppState> {
    Router::new()
        .route("/api/incidents", post(incident_handlers::create_incident))
        .route("/api/incidents", get(incident_handlers::list_incidents))
        .route("/api/incidents/:id", put(incident_handlers::update_incident))
}