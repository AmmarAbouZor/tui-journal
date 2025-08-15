/// FileHeader is an entry...
///
/// * without the "content" part
/// * without the ID (which is stored in the file name)
/// * without the date (which is stored in the file name)
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FileHeader {
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub priority: Option<u32>,
}
