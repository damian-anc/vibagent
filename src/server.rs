use axum::{
    extract::Json,
    response::sse::{Event, Sse},
    routing::post,
    Router,
};
use futures::stream::Stream;
use std::{convert::Infallible, pin::Pin, sync::Arc};
use crate::agent::Agent;
use crate::models::InputEvent;
use crate::tools::{CalculatorTool, RunCommand, WebSearchTool, GeocodingTool, StationLookupTool, ClimateDataTool};
use tracing::info;

pub fn app(api_key: String, model: String) -> Router {
    let state = Arc::new(ServerState { api_key, model });
    let cors = tower_http::cors::CorsLayer::permissive();
    
    Router::new()
        .route("/agent", post(handle_agent_request))
        .layer(cors)
        .with_state(state)
}

struct ServerState {
    api_key: String,
    model: String,
}

async fn handle_agent_request(
    axum::extract::State(state): axum::extract::State<Arc<ServerState>>,
    Json(input): Json<InputEvent>,
) -> Sse<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>> {
    info!("Handling agent request: {:?}", input);
    
    let agent = Agent::new(
        state.api_key.clone(),
        state.model.clone(),
        vec![
            Box::new(CalculatorTool),
            Box::new(RunCommand),
            Box::new(WebSearchTool),
            Box::new(GeocodingTool),
            Box::new(StationLookupTool::new("data/ghcnd_stations.db")),
            Box::new(ClimateDataTool::new("data/climate_data.db", "/Volumes/Data/ghcn-data")),
        ],
    );

    let stream = match agent.run(input).await {
        Ok(s) => s,
        Err(e) => {
            // Return an error event if agent fails to start
            let error_event = Event::default().data(format!("Error: {}", e));
            return Sse::new(Box::pin(futures::stream::once(async move { Ok(error_event) })));
        }
    };

    let event_stream = futures::stream::StreamExt::map(stream, |output_event| {
        let data = serde_json::to_string(&output_event).unwrap_or_else(|_| "error serializing event".to_string());
        Ok(Event::default().data(data))
    });

    Sse::new(Box::pin(event_stream))
}
