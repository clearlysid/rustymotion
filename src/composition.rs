use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Composition {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration_in_frames: u32,
    pub id: String,
    pub serialized_resolved_props_with_custom_schema: String,
    pub serialized_default_props_with_custom_schema: String,
}

pub fn derive(json: String) -> Vec<Composition> {
    let comps: Vec<Composition> = serde_json::from_str(&json).unwrap();
    comps
}
