use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Composition {
    pub id: String,
    pub fps: u32,
    pub width: u32,
    pub height: u32,

    #[serde(rename = "durationInFrames")]
    pub duration_in_frames: u32,
    #[serde(rename = "serializedResolvedPropsWithCustomSchema")]
    pub serialized_resolved_props_with_custom_schema: String,
    #[serde(rename = "serializedDefaultPropsWithCustomSchema")]
    pub serialized_default_props_with_custom_schema: String,
}

// pub fn derive_composition_list(json: String) -> Vec<Composition> {
//     let comps: Vec<Composition> = serde_json::from_str(&json).unwrap();
//     comps
// }

pub fn derive_composition(json: String) -> Composition {
    let comp: Composition = serde_json::from_str(&json).unwrap();
    comp
}
