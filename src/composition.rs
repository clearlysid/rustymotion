use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Composition {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub durationInFrames: u32,
    pub id: String,
    pub serializedResolvedPropsWithCustomSchema: String,
    pub serializedDefaultPropsWithCustomSchema: String,
}

pub fn derive(json: String) -> Vec<Composition> {
    let comps: Vec<Composition> = serde_json::from_str(&json).unwrap();
    comps
}
