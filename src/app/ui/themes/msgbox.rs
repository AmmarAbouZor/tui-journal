use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgBoxColors {
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub question: Color,
}

impl Default for MsgBoxColors {
    fn default() -> Self {
        Self {
            error: Color::LightRed,
            warning: Color::Yellow,
            info: Color::LightGreen,
            question: Color::LightBlue,
        }
    }
}
