use std::collections::BTreeMap;

use backend::Entry;

/// A node in the tag-based folder hierarchy.
///
/// Tags are split on `.` to build the tree. An entry with tag `linux.ubuntu`
/// contributes to the node at path `["linux", "ubuntu"]`.
///
/// **Placement rule:** An entry is placed only at the **deepest** matching
/// node for each of its tags. Entries with no tags are placed at the root.
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
            if entry.tags.is_empty() {
                // Entries with no tags live at the root level.
                root.entry_ids.push(entry.id);
            } else {
                for tag in &entry.tags {
                    let segments: Vec<&str> = tag.split('.').collect();
                    root.insert_entry(entry.id, &segments);
                }
            }
        }

        root
    }

    /// Insert `entry_id` at the deepest node described by `path`.
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

    fn make_entry(id: u32, tags: Vec<&str>) -> Entry {
        Entry::new(
            id,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            format!("Entry {id}"),
            String::new(),
            tags.into_iter().map(String::from).collect(),
            None,
        )
    }

    #[test]
    fn no_tags_entry_goes_to_root() {
        let entry = make_entry(1, vec![]);
        let tree = TagTree::build(std::iter::once(&entry));

        assert_eq!(tree.entry_ids, vec![1]);
        assert!(tree.subfolders.is_empty());
    }

    #[test]
    fn single_segment_tag_creates_top_level_folder() {
        let entry = make_entry(1, vec!["rust"]);
        let tree = TagTree::build(std::iter::once(&entry));

        assert!(tree.entry_ids.is_empty());
        let rust = tree.subfolders.get("rust").unwrap();
        assert_eq!(rust.entry_ids, vec![1]);
    }

    #[test]
    fn dotted_tag_places_entry_at_deepest_level_only() {
        let entry = make_entry(1, vec!["linux.ubuntu"]);
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
    fn entry_with_multiple_tags_appears_in_each_deepest_folder() {
        let entry = make_entry(1, vec!["linux.ubuntu", "rust"]);
        let tree = TagTree::build(std::iter::once(&entry));

        let ubuntu = tree
            .subfolders
            .get("linux")
            .and_then(|l| l.subfolders.get("ubuntu"))
            .unwrap();
        assert_eq!(ubuntu.entry_ids, vec![1]);

        let rust = tree.subfolders.get("rust").unwrap();
        assert_eq!(rust.entry_ids, vec![1]);
    }

    #[test]
    fn get_node_returns_correct_subtree() {
        let entry = make_entry(42, vec!["a.b.c"]);
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
}
