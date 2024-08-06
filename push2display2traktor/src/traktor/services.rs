use crate::traktor::{AppState, DeckContent, DeckID, FxUnitType};
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

// Very simple get function to view the state
#[get("/state")]
pub async fn get_state(data: web::Data<AppState>) -> HttpResponse {
    let data = data.traktor.lock().await;
    HttpResponse::Ok().json(data.clone())
}

#[post("/deckLoaded/{deck_id}")]
async fn receive_deck_loaded_event(
    deck_id: web::Path<String>,
    event: web::Json<DeckLoaded>,
    data: web::Data<AppState>,
) -> HttpResponse {

    let c = if let Some(c) = deck_id.chars().next() {
        c
    } else {
        return HttpResponse::Ok().finish();
    };

    let deck_id = if let Some(d) = DeckID::from_char(c) {
        d
    } else {
        return HttpResponse::Ok().finish();
    };

    {
        let mut state = data.traktor.lock().await;

        // find deck
        let deck_idx = if let Some(d) = state.decks.iter().position(|d| d.id == deck_id) {
            d
        } else {
            return HttpResponse::Ok().finish();
        };
        let deck = &mut (*state).decks[deck_idx];

        // Update all values
        deck.content = Some(event.into_inner().into());
    }

    HttpResponse::Ok().finish()
}

#[post("/updateDeck/{deck_id}")]
async fn receive_deck_update_event(
    deck_id: web::Path<String>,
    event: web::Json<DeckUpdate>,
    data: web::Data<AppState>,
) -> HttpResponse {

    let c = if let Some(c) = deck_id.chars().next() {
        c
    } else {
        return HttpResponse::Ok().finish();
    };

    let deck_id = if let Some(d) = DeckID::from_char(c) {
        d
    } else {
        return HttpResponse::Ok().finish();
    };


    {
        let mut state = data.traktor.lock().await;

        // find deck
        let deck_idx = if let Some(d) = state.decks.iter().position(|d| d.id == deck_id) {
            d
        } else {
            return HttpResponse::Ok().finish();
        };
        let deck = &mut (*state).decks[deck_idx];

        // Update all values
        match event.into_inner() {
            DeckUpdate::ResultingKey { resulting_key } => {
                if let Some(content) = &mut deck.content {
                    content.resulting_key = resulting_key;
                }
            }
            // For now do nothing
            _ => (),
        }
    }

    HttpResponse::Ok().finish()
}

#[derive(Debug, Deserialize)]
struct DeckLoaded {
    #[serde(rename = "filePath")]
    file_path: String,
    title: String,
    artist: String,
    album: String,
    genre: String,
    label: String,
    key: String,
    // after adjust
    #[serde(rename = "resultingKey")]
    resulting_key: String,
    #[serde(rename = "trackLength")]
    track_length: f64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DeckUpdate {
    IsPlaying {
        is_playing: bool,
        elapsed_time: f64,
        next_cue_pos: f64,
    },
    IsSynced {
        is_synced: bool,
    },
    IsKeyLockOn {
        is_key_lock_on: bool,
    },
    Tempo {
        tempo: f64,
    },
    ResultingKey {
        #[serde(rename = "resultingKey")]
        resulting_key: String,
    },
    ElapsedTime {
        elapsed_time: f64,
        next_cue_pos: f64,
    },
}

impl Into<DeckContent> for DeckLoaded {
    fn into(self) -> DeckContent {
        DeckContent {
            title: self.title,
            file_path: self.file_path,
            artist: self.artist,
            album: self.album,
            genre: self.genre,
            label: self.label,
            key: self.key,
            resulting_key: self.resulting_key,
            length: self.track_length,
        }
    }
}

#[post("/fx/{fx_unit_id}")]
async fn receive_fx_event(
    fx_unit_id: web::Path<usize>,
    event: web::Json<FxEvent>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let e: FxChangeEvent = match event.into_inner() {
        FxEvent::Type(e) => FxChangeEvent {
            fx_type: Some(e.description),
            ..Default::default()
        },
        FxEvent::Select(e) => FxChangeEvent {
            fx_id: fx_id_from_traktor_path(&e.path),
            name: Some(e.description),
            ..Default::default()
        },
        FxEvent::DryWet(e) => FxChangeEvent {
            fx_id: Some(0),
            value_description: Some(e.description),
            value: Some(e.value),
            name: Some("D/W".to_string()),
            ..Default::default()
        },
        FxEvent::Knob(e) => FxChangeEvent {
            fx_id: fx_id_from_traktor_path(&e.path),
            value: Some(e.value),
            ..Default::default()
        },
        FxEvent::Name(e) => FxChangeEvent {
            fx_id: fx_id_from_traktor_path(&e.path),
            name: Some(e.value),
            ..Default::default()
        },
        FxEvent::Param(e) => FxChangeEvent {
            fx_id: fx_id_from_traktor_path(&e.path),
            value: Some(e.value),
            ..Default::default()
        },
    };

    //println!("Got event {:#?}",e);

    // Update state
    {
        let mut state = data.traktor.lock().await;
        let fx_unit = &mut (*state).fx_units[fx_unit_id.into_inner() - 1];

        if let Some(fx_type) = e.fx_type {
            fx_unit.r#type = match fx_type.as_str() {
                "Group" => FxUnitType::Group,
                "Single" => FxUnitType::Single,
                _ => FxUnitType::UNK,
            };
        };

        if let Some(fx_id) = e.fx_id {
            let knob = &mut fx_unit.knobs[fx_id as usize];

            if let Some(val) = e.value {
                knob.value = val;
            };
            if let Some(d) = e.value_description {
                knob.value_description = d;
            };
            if let Some(n) = e.name {
                knob.fx_name = n;
            }
        }
    }

    HttpResponse::Ok().finish()
}

fn fx_id_from_traktor_path(path: &str) -> Option<u8> {
    // Split the string by dots
    let parts: Vec<&str> = path.split('.').collect();

    // Check if the split produced enough parts to avoid index out of bounds
    // ignore app.traktor.fx.[n]
    if parts.len() > 5 {
        // Is always the 6th pos
        if let Ok(number) = parts[5].parse::<u8>() {
            return Some(number);
        }
    }

    // Return None if parsing failed or input was invalid
    None
}

#[derive(Deserialize)]
enum FxEvent {
    //{'Type': {'objectName': '', 'path': 'app.traktor.fx.1.type', 'value': 0, 'description': 'Group', 'enabled': True, 'valueRange': {'objectName': '', 'min': 0, 'max': 2, 'def': 0, 'steps': 3, 'type': 'Discrete', 'isFull': False, 'isContinuous': False, 'isDiscrete': True}, 'valuesDescription': ['Group', 'Single', 'Pattern Player']}}
    Type(FxEventJsonParam),
    //{'Select': {'objectName': '', 'path': 'app.traktor.fx.1.select.3', 'value': 20, 'description': 'Auto Bouncer', 'enabled': True, 'valueRange': {'objectName': '', 'min': 0, 'max': 31, 'def': 0, 'steps': 32, 'type': 'Discrete', 'isFull': False, 'isContinuous': False, 'isDiscrete': True}, 'valuesDescription': ['No Effect', 'Delay', 'Reverb', 'Flanger', 'Gater', 'Beatmasher 2', 'Delay T3', 'Filter:92', 'Phaser', 'Reverb T3', 'Ringmodulator', 'Digital LoFi', 'Mulholland Drive', 'Transpose Stretch', 'BeatSlicer', 'Formant Filter', 'Bouncer', 'Peak Filter', 'Tape Delay', 'Ramp Delay', 'Auto Bouncer', '¶ WormHole', '¶ LaserSlicer', '¶ GranuPhase', '¶ Bass-o-Matic', '¶ PolarWind', '¶ EventHorizon', '¶ Zzzurp', '¶ FlightTest', '¶ Strrretch (Slow)', '¶ Strrretch (Fast)', '¶ DarkMatter']}}
    Select(FxEventJsonParam),
    //{'DryWet': {'objectName': '', 'path': 'app.traktor.fx.1.dry_wet', 'value': 1, 'description': '100%', 'enabled': True, 'valueRange': {'objectName': '', 'min': 0, 'max': 1, 'def': 0, 'steps': 0, 'type': 'Continuous', 'isFull': False, 'isContinuous': True, 'isDiscrete': False}, 'valuesDescription': []}}
    DryWet(FxEventJsonParam),
    //{'Knob': {'objectName': '', 'path': 'app.traktor.fx.1.knobs.1', 'value': 0, 'description': 'float', 'enabled': True, 'valueRange': {'objectName': '', 'min': 0, 'max': 1, 'def': 0, 'steps': 0, 'type': 'Continuous', 'isFull': False, 'isContinuous': True, 'isDiscrete': False}, 'valuesDescription': []}}
    Knob(FxEventJsonParam),
    //{'Name': {'objectName': '', 'path': 'app.traktor.fx.1.knobs.3.name', 'value': 'LEN', 'description': 'LEN', 'enabled': True, 'valueRange': {'objectName': '', 'min': '', 'max': '', 'def': '', 'steps': 0, 'type': 'Full', 'isFull': True, 'isContinuous': False, 'isDiscrete': False}, 'valuesDescription': []}}
    Name(FxEventJsonName),
    //{'Param': {'objectName': '', 'path': 'app.traktor.fx.1.parameters.1', 'value': 0.5019609928131104, 'description': '0', 'enabled': True, 'valueRange': {'objectName': '', 'min': -3.4028234663852886e+38, 'max': 3.4028234663852886e+38, 'def': 0, 'steps': 0, 'type': 'Full', 'isFull': True, 'isContinuous': False, 'isDiscrete': False}, 'valuesDescription': []}}
    Param(FxEventJsonParam),
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
struct FxEventJsonParam {
    path: String,
    value: f64,
    description: String,
    enabled: bool,
    valueRange: ValueRange,
    valuesDescription: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FxEventJsonName {
    path: String,
    value: String,
    description: String,
    enabled: bool,
}

// for internal parsing
// indicates a change in one or multiple
//of these values
#[derive(Default, Debug)]
struct FxChangeEvent {
    fx_type: Option<String>,

    // these are all bound to fx_id
    // was too lazy to extract this into another enum
    fx_id: Option<u8>,
    value: Option<f64>,
    value_description: Option<String>,
    name: Option<String>,
}
