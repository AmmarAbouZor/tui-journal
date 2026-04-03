use std::collections::BTreeMap;

use backend::Entry;

/// A node in the folder-based hierarchy.
///
/// Folders are split on `/` to build the tree. An entry with folder `work/project`
/// contributes to the node at path `["work", "project"]`.
///
/// **Placement rule:** An entry is placed exactly at the node described by its
/// `folder` field. Entries with an empty folder are placed at the root.
#[derive(Debug, Default)]
pub struct TagTree {
    /// Ordered sub-folders at this level.
    pub subfolders: BTreeMap<String, TagTree>,
    /// IDs of entries placed directly at this node.
    pub entry_ids: Vec<u32>,
}

impl TagTree {
    /// Build a `TagTree` from an iterator of entries.
    pub fn build<'a>(entries: impl Iterator<Item = &'a Entry>) -> Self {
        let mut root = TagTree::default();

        for entry in entries {
            let segments: Vec<&str> = entry
                .folder
                .split('/')
                .filter(|s| !s.is_empty())
                .collect();

            if segments.is_empty() {
                // Entries with no folder (or only slashes) live at the root level.
                root.entry_ids.push(entry.id);
            } else {
                root.insert_entry(entry.id, &segments);
            }
        }

        root
    }

    /// Insert `entry_id` at the deepest node described by `path`.
    ///
    /// The `path` is a slice of folder segments (e.g., `["work", "project"]`).
    fn insert_entry(&mut self, entry_id: u32, path: &[&str]) {
        match path {
            [] => {
                self.entry_ids.push(entry_id);
            }
            [segment] => {
                // Deepest level — place the entry here.
                let node = self.subfolders.entry((*segment).to_owned()).or_default();
                node.entry_ids.push(entry_id);
            }
            [segment, rest @ ..] => {
                // Not yet at the deepest level — descend.
                let node = self.subfolders.entry((*segment).to_owned()).or_default();
                node.insert_entry(entry_id, rest);
            }
        }
    }

    /// Navigate to a descendant node given a slice of folder-name segments.
    /// Returns `None` if the path doesn't exist.
    pub fn get_node(&self, path: &[String]) -> Option<&TagTree> {
        if path.is_empty() {
            return Some(self);
        }
        self.subfolders
            .get(&path[0])
            .and_then(|child| child.get_node(&path[1..]))
    }

    /// Sorted subfolder names at this node.
    pub fn subfolder_names(&self) -> Vec<&str> {
        self.subfolders.keys().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn make_entry(id: u32, folder: &str) -> Entry {
        Entry::new(
            id,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            format!("Entry {id}"),
            String::new(),
            vec![],
            None,
            folder.to_string(),
        )
    }

    #[test]
    fn no_folder_entry_goes_to_root() {
        let entry = make_entry(1, "");
        let tree = TagTree::build(std::iter::once(&entry));

        assert_eq!(tree.entry_ids, vec![1]);
        assert!(tree.subfolders.is_empty());
    }

    #[test]
    fn single_segment_folder_creates_top_level_folder() {
        let entry = make_entry(1, "rust");
        let tree = TagTree::build(std::iter::once(&entry));

        assert!(tree.entry_ids.is_empty());
        let rust = tree.subfolders.get("rust").unwrap();
        assert_eq!(rust.entry_ids, vec![1]);
    }

    #[test]
    fn nested_folder_places_entry_at_deepest_level_only() {
        let entry = make_entry(1, "linux/ubuntu");
        let tree = TagTree::build(std::iter::once(&entry));

        // Root: no entries, one folder "linux"
        assert!(tree.entry_ids.is_empty());
        let linux = tree.subfolders.get("linux").unwrap();
        // "linux" folder: no entries (entry is deeper), one subfolder "ubuntu"
        assert!(linux.entry_ids.is_empty());
        let ubuntu = linux.subfolders.get("ubuntu").unwrap();
        assert_eq!(ubuntu.entry_ids, vec![1]);
    }

    #[test]
    fn entries_in_different_folders_are_separated() {
        let entry1 = make_entry(1, "rust");
        let entry2 = make_entry(2, "linux/ubuntu");
        let tree = TagTree::build(vec![&entry1, &entry2].into_iter());

        assert_eq!(tree.subfolders.get("rust").unwrap().entry_ids, vec![1]);
        assert_eq!(
            tree.subfolders
                .get("linux")
                .unwrap()
                .subfolders
                .get("ubuntu")
                .unwrap()
                .entry_ids,
            vec![2]
        );
    }

    #[test]
    fn get_node_returns_correct_subtree() {
        let entry = make_entry(42, "a/b/c");
        let tree = TagTree::build(std::iter::once(&entry));

        let node = tree.get_node(&["a".into(), "b".into(), "c".into()]);
        assert!(node.is_some());
        assert_eq!(node.unwrap().entry_ids, vec![42]);

        assert!(tree.get_node(&["a".into(), "b".into()]).is_some());
        assert!(
            tree.get_node(&["a".into(), "b".into()])
                .unwrap()
                .entry_ids
                .is_empty()
        );
    }

    #[test]
    fn build_handles_redundant_slashes() {
        let entry = make_entry(1, "work//project/");
        let tree = TagTree::build(std::iter::once(&entry));

        let work = tree.subfolders.get("work").expect("work node exists");
        let project = work.subfolders.get("project").expect("project node exists");
        assert_eq!(project.entry_ids, vec![1]);
    }
}
