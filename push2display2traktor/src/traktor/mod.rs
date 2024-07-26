use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use serde::Serialize;
use services::{get_state, receive_fx_event};
use tokio::sync::Mutex;

mod services;

#[derive(Serialize, Clone)]
pub struct TraktorState {
    fx_units: Vec<FXUnit>,
}
#[derive(Serialize, Clone)]
pub struct FXUnit {
    // Identifier in traktor [1,4]
    id: u8,
    r#type:FxUnitType,
    // Each fx unit has four knobs (drywet + 3*effect)
    knobs: Vec<Knob>,
}

#[derive(Serialize, Clone)]
pub enum FxUnitType {
    Group,
    Single,
    UNK
}

#[derive(Serialize, Clone)]
struct Knob {
    // Identifier in traktor [1,3]
    // 0 for dry wet
    id: u8,
    value: f64,
    value_description: String,
    fx_name: String,
}

impl Default for TraktorState {
    fn default() -> Self {
        let fx_units: Vec<FXUnit> = (1..5).into_iter().map(|i| FXUnit::new(i as u8)).collect();
        Self { fx_units }
    }
}
impl TraktorState {
    fn iter_all_knobs(&self) -> impl Iterator<Item = &Knob> {
        self.fx_units.iter().flat_map(|unit| unit.knobs.iter())
    }

    pub fn iter_knob_fx_names(&self) -> impl Iterator<Item = &String> {
        self.iter_all_knobs().map(|k| &k.fx_name)
    }

    pub fn iter_knob_values(&self) -> impl Iterator<Item = &f64> {
        self.iter_all_knobs().map(|k| &k.value)
    }
}

impl FXUnit {
    pub fn new(id: u8) -> Self {
        let knobs: Vec<Knob> = (0..4)
            .into_iter()
            .map(|i| Knob {
                id: i as u8,
                value: 0.5,
                fx_name: String::from("UNK"),
                value_description: "UNK".to_string(),
            })
            .collect();

        Self { id, knobs, r#type:FxUnitType::UNK }
    }
}

/// App state is basically a wrapper for everything on the server
/// I decided to just hold one arc mutex
pub struct AppState {
    pub traktor: Arc<Mutex<TraktorState>>,
}

/// Creates a simple server that parses the http request from traktor to our
/// state
pub async fn create_server(state: Arc<Mutex<TraktorState>>) -> std::io::Result<()> {
    let state = web::Data::new(AppState { traktor: state });

    // Start HTTP server
    HttpServer::new(move || {
        //Move state into closure
        App::new()
            .app_data(state.clone())
            .service(get_state)
            .service(receive_fx_event)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
