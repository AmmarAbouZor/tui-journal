use crate::file::header::FileHeader;
use crate::file::path::EntryFilePathBuf;

pub(super) struct Entry {
    path: EntryFilePathBuf,
    header: FileHeader,
    content: String,
}

impl Entry {
    pub fn from_entry(entry: &crate::Entry) -> Result<Self, EntryError> {
        todo!()
    }

    pub fn as_entry(&self) -> Result<crate::Entry, EntryError> {
        todo!()
    }

    pub async fn load(path: EntryFilePathBuf) -> Result<Self, EntryError> {
        todo!()
    }

    pub async fn write_to_disk(&self) -> Result<(), EntryError> {
        todo!()
    }

    pub(super) fn path(&self) -> &EntryFilePathBuf {
        &self.path
    }

    pub(super) fn header(&self) -> &FileHeader {
        &self.header
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EntryError {

}
