use crate::audio::DspSettings;
use crate::engine::EffectConfig;
use crate::store::{EngineState, Scene};
use crate::types::{Device, Virtual};
use serde::Serialize;
use specta::Type;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

// --- Data Structures ---

pub struct ActiveVirtual {
    pub effect: Option<Box<dyn crate::effects::Effect>>,
    pub config: Virtual,
    pub pixel_count: usize,
    pub r_channel: Vec<f32>,
    pub g_channel: Vec<f32>,
    pub b_channel: Vec<f32>,
}

#[derive(Serialize, Type, Clone)]
pub struct EffectInfo {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Type, Clone)]
pub struct PlaybackState {
    pub is_paused: bool,
}

#[derive(Serialize, Type, Clone)]
pub struct PresetCollection {
    pub user: HashMap<String, crate::engine::EffectConfig>,
    pub built_in: HashMap<String, crate::engine::EffectConfig>,
}

#[derive(Serialize, Type, Clone)]
pub struct ActiveEffectsState {
    pub active_scene_id: Option<String>,
    pub selected_effects: HashMap<String, String>,
    pub effect_settings: HashMap<String, HashMap<String, crate::engine::generated::EffectConfig>>,
    pub active_effects: HashMap<String, bool>,
}

// --- Communication Types ---

pub struct EngineStateTx(pub Sender<EngineRequest>);

pub enum EngineRequest {
    GetVirtuals(Sender<Vec<Virtual>>),
    GetDevices(Sender<Vec<Device>>),
    GetDspSettings(Sender<DspSettings>),
    GetPlaybackState(Sender<PlaybackState>),
    GetPresets(String, Sender<PresetCollection>),
    GetScenes(Sender<Vec<Scene>>),
    GetFullState(Sender<EngineState>),
    SavePreset {
        effect_id: String,
        preset_name: String,
        settings: EffectConfig,
        responder: Sender<()>,
    },
    DeletePreset {
        effect_id: String,
        preset_name: String,
        responder: Sender<()>,
    },
}
