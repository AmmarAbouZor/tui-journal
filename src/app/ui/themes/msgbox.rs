use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MsgBoxColors {
    #[serde(default = "def_error")]
    pub error: Color,
    #[serde(default = "def_warning")]
    pub warning: Color,
    #[serde(default = "def_info")]
    pub info: Color,
    #[serde(default = "def_question")]
    pub question: Color,
}

impl Default for MsgBoxColors {
    fn default() -> Self {
        Self {
            error: def_error(),
            warning: def_warning(),
            info: def_info(),
            question: def_question(),
        }
    }
}

#[inline]
fn def_error() -> Color {
    Color::LightRed
}

#[inline]
fn def_warning() -> Color {
    Color::Yellow
}

#[inline]
fn def_info() -> Color {
    Color::LightGreen
}

#[inline]
fn def_question() -> Color {
    Color::LightBlue
}
