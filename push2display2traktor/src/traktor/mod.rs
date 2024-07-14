use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use serde::Serialize;
use services::{get_state, receive_fx_event};
use tokio::sync::Mutex;

mod services;

use crate::render::storage_buffer::TraktorStateStorageData;

#[derive(Serialize, Clone)]
pub struct TraktorState {
    fx_units: Vec<FXUnit>,
}
#[derive(Serialize, Clone)]
pub struct FXUnit {
    // Identifier in traktor [1,4]
    id: u8,
    // Each fx unit has four knobs (drywet + 3*effect)
    knobs: Vec<Knob>,
}

#[derive(Serialize, Clone)]
struct Knob {
    // Identifier in traktor [1,3]
    // 0 for dry wet
    id: u8,
    value: f64,
    fx_name: String,
}

impl Default for TraktorState {
    fn default() -> Self {
        let fx_units: Vec<FXUnit> = (1..5).into_iter().map(|i| FXUnit::new(i as u8)).collect();
        Self { fx_units }
    }
}
impl TraktorState {
    pub fn to_uniform(&self) -> TraktorStateStorageData {
        // for now hardcoded 8 knobs

        let n_knobs = 8;
        let knobs: [f32; 16] = self.all_knob_values();

        return TraktorStateStorageData::new(n_knobs, knobs);
    }

    fn iter_all_knobs(&self) -> impl Iterator<Item = &Knob> {
        self.fx_units.iter().flat_map(|unit| unit.knobs.iter())
    }

    fn all_knob_values(&self) -> [f32; 16] {
        let mut values: [f32; 16] = [0.0; 16]; // Initialize an array of 16 zeros
        for (i, knob) in self.iter_all_knobs().enumerate() {
            values[i] = knob.value as f32;
        }
        values
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
            })
            .collect();

        Self { id, knobs }
    }
}

/// App state is basically a wrapper for everything on the server
/// I decided to just hold one arc mutex
pub struct AppState {
    pub traktor: Arc<Mutex<TraktorState>>,
}

/// Creates a simple server that parses the http request from traktor
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
