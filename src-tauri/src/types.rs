use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Device {
    pub ip_address: String,
    pub name: String,
    pub led_count: u32,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct MatrixCell {
    pub device_id: String,
    pub pixel: u32,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Virtual {
    pub id: String,
    pub name: String,
    pub matrix_data: Vec<Vec<Option<MatrixCell>>>,
    #[serde(default)]
    pub is_device: Option<String>,
}