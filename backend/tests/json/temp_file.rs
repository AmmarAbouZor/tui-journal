use std::{env, fs, path::PathBuf};
pub struct TempFile {
    pub file_path: PathBuf,
}

impl TempFile {
    pub fn new(file_name: &str) -> Self {
        let file_path = env::temp_dir().join(file_name);

        let temp_file = Self { file_path };
        temp_file.clean_up();

        temp_file
    }

    pub fn clean_up(&self) {
        if self
            .file_path
            .try_exists()
            .expect("Access to check the test file should be given")
        {
            fs::remove_file(&self.file_path)
                .expect("Access to delete the test file should be given");
        }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        self.clean_up();
    }
}

