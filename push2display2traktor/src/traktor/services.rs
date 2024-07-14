use crate::traktor::AppState;
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

// Very simple get function to view the state
#[get("/state")]
pub async fn get_state(data: web::Data<AppState>) -> HttpResponse {
    let data = data.traktor.lock().await;
    HttpResponse::Ok().json(data.clone())
}

#[post("/fx/{fx_unit_id}")]
async fn receive_fx_event(fx_unit_id: web::Path<usize>, event: web::Json<FxEvent>, data: web::Data<AppState>) -> HttpResponse {
    let (fx_id,value) : (Option<usize>, f64)= match event.into_inner() {
        FxEvent::Type(e) => {
            // Handle the TestStruct variant
            (fx_id_from_traktor_path(&e.path),e.value)
        }
        FxEvent::Select(e) => {
            // Handle the TestStruct variant
            (fx_id_from_traktor_path(&e.path),e.value)
        }
        FxEvent::DryWet(e) => {
            // Handle the TestStruct variant
            //  app.traktor.fx.1.dry_wet
            (Some(0), e.value)
        }
        FxEvent::Knob(e) => {
            // Handle the TestStruct variant
            (fx_id_from_traktor_path(&e.path),e.value)
        }
        FxEvent::Name(e) => {
            // Handle the TestStruct variant
            (fx_id_from_traktor_path(&e.path),e.value)
        }
        FxEvent::Param(e) => {
            // Handle the TestStruct variant
            (fx_id_from_traktor_path(&e.path),e.value)
        }
    };
    println!("Got fx event unit {}, fx_id {:?}, value {:.2}", fx_unit_id, fx_id, value);

    // Update data
    if let Some(fx_id) = fx_id {
        let mut traktor_state = data.traktor.lock().await;
        (*traktor_state).fx_units[fx_unit_id.into_inner()-1].knobs[fx_id].value = value;
    }

    HttpResponse::Ok().finish()
}


fn fx_id_from_traktor_path(path: &str) -> Option<usize> {
    // Split the string by dots
    let parts: Vec<&str> = path.split('.').collect();
    
    // Check if the split produced enough parts to avoid index out of bounds
    if let Some(last_part) = parts.last() {
        // Attempt to parse the last part as an integer
        if let Ok(number) = last_part.parse::<usize>() {
            return Some(number);
        }
    }
    
    // Return None if parsing failed or input was invalid
    None
}

#[derive(Deserialize)]
enum FxEvent {
    Type(FxEventJson),
    Select(FxEventJson),
    DryWet(FxEventJson),
    Knob(FxEventJson),
    Name(FxEventJson),
    Param(FxEventJson),
}

#[derive(Debug, Deserialize)]
struct ValueRange {
    min: f64,
    max: f64,
    def: i32,
    steps: i32,
    #[serde(rename = "type")]
    range_type: String,
    isFull: bool,
    isContinuous: bool,
    isDiscrete: bool,
}

// generic event data returned by the
// api layer

#[derive(Debug, Deserialize)]
struct FxEventJson {
    path: String,
    value: f64,
    description: String,
    enabled: bool,
    valueRange: ValueRange,
    valuesDescription: Vec<String>,
}

