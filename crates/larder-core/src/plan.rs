use std::collections::{BTreeMap, BTreeSet};

use crate::paths::RelPath;
use crate::scan::Manifest;

/// What the client remembers about each file from the last time it was in
/// sync with the host: path -> hash. Without this we couldn't tell "I deleted
/// it" apart from "the host has a new file I haven't downloaded yet".
pub type Base = BTreeMap<RelPath, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Send the local file up. `base` is the hash we think the host has,
    /// so the host can refuse if someone else got there first.
    Upload { path: RelPath, base: Option<String> },
    Download { path: RelPath, hash: String },
    DeleteLocal { path: RelPath },
    DeleteRemote { path: RelPath, base: String },
    /// Both sides changed the file. Keep the host version under the real
    /// name and move ours aside as a conflict copy.
    Conflict { path: RelPath, remote_hash: String },
    /// Nothing to transfer, just record that both sides agree now.
    Settle { path: RelPath, hash: Option<String> },
}

pub fn plan(local: &Manifest, remote: &Manifest, base: &Base) -> Vec<Action> {
    let paths: BTreeSet<&RelPath> = local.keys().chain(remote.keys()).chain(base.keys()).collect();
    paths
        .into_iter()
        .filter_map(|path| {
            let l = local.get(path).map(|e| e.hash.as_str());
            let r = remote.get(path).map(|e| e.hash.as_str());
            let b = base.get(path).map(String::as_str);
            decide(path, l, r, b)
        })
        .collect()
}

fn decide(path: &RelPath, l: Option<&str>, r: Option<&str>, b: Option<&str>) -> Option<Action> {
    let path = path.clone();
    let owned = |h: Option<&str>| h.map(str::to_string);

    if l == r {
        return (b != l).then(|| Action::Settle { path, hash: owned(l) });
    }
    if l == b {
        return Some(match r {
            Some(hash) => Action::Download { path, hash: hash.into() },
            None => Action::DeleteLocal { path },
        });
    }
    if r == b {
        return Some(match (l, b) {
            (None, Some(base)) => Action::DeleteRemote { path, base: base.into() },
            _ => Action::Upload { path, base: owned(b) },
        });
    }
    // Both sides moved. An edit always beats a delete, because a delete you
    // didn't want is much worse than a file coming back.
    Some(match (l, r) {
        (Some(_), Some(remote)) => Action::Conflict { path, remote_hash: remote.into() },
        (None, Some(remote)) => Action::Download { path, hash: remote.into() },
        _ => Action::Upload { path, base: None },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::FileEntry;

    fn p(s: &str) -> RelPath {
        RelPath::parse(s).unwrap()
    }

    fn files(items: &[(&str, &str)]) -> Manifest {
        items
            .iter()
            .map(|(path, hash)| (p(path), FileEntry { size: 1, modified: 0, hash: hash.to_string() }))
            .collect()
    }

    fn base(items: &[(&str, &str)]) -> Base {
        items.iter().map(|(path, hash)| (p(path), hash.to_string())).collect()
    }

    fn one(local: &[(&str, &str)], remote: &[(&str, &str)], last: &[(&str, &str)]) -> Option<Action> {
        let actions = plan(&files(local), &files(remote), &base(last));
        assert!(actions.len() <= 1, "expected at most one action, got {actions:?}");
        actions.into_iter().next()
    }

    #[test]
    fn in_sync_does_nothing() {
        assert_eq!(one(&[("a", "1")], &[("a", "1")], &[("a", "1")]), None);
    }

    #[test]
    fn new_local_file_uploads() {
        assert_eq!(one(&[("a", "1")], &[], &[]), Some(Action::Upload { path: p("a"), base: None }));
    }

    #[test]
    fn new_remote_file_downloads() {
        assert_eq!(one(&[], &[("a", "1")], &[]), Some(Action::Download { path: p("a"), hash: "1".into() }));
    }

    #[test]
    fn local_edit_uploads_with_base() {
        let got = one(&[("a", "2")], &[("a", "1")], &[("a", "1")]);
        assert_eq!(got, Some(Action::Upload { path: p("a"), base: Some("1".into()) }));
    }

    #[test]
    fn remote_edit_downloads() {
        let got = one(&[("a", "1")], &[("a", "2")], &[("a", "1")]);
        assert_eq!(got, Some(Action::Download { path: p("a"), hash: "2".into() }));
    }

    #[test]
    fn local_delete_goes_to_host() {
        let got = one(&[], &[("a", "1")], &[("a", "1")]);
        assert_eq!(got, Some(Action::DeleteRemote { path: p("a"), base: "1".into() }));
    }

    #[test]
    fn remote_delete_comes_down() {
        assert_eq!(one(&[("a", "1")], &[], &[("a", "1")]), Some(Action::DeleteLocal { path: p("a") }));
    }

    #[test]
    fn both_edited_is_a_conflict() {
        let got = one(&[("a", "2")], &[("a", "3")], &[("a", "1")]);
        assert_eq!(got, Some(Action::Conflict { path: p("a"), remote_hash: "3".into() }));
    }

    #[test]
    fn same_new_file_on_both_sides_just_settles() {
        let got = one(&[("a", "9")], &[("a", "9")], &[]);
        assert_eq!(got, Some(Action::Settle { path: p("a"), hash: Some("9".into()) }));
    }

    #[test]
    fn deleted_on_both_sides_forgets_base() {
        assert_eq!(one(&[], &[], &[("a", "1")]), Some(Action::Settle { path: p("a"), hash: None }));
    }

    #[test]
    fn remote_edit_beats_local_delete() {
        let got = one(&[], &[("a", "2")], &[("a", "1")]);
        assert_eq!(got, Some(Action::Download { path: p("a"), hash: "2".into() }));
    }

    #[test]
    fn local_edit_beats_remote_delete() {
        let got = one(&[("a", "2")], &[], &[("a", "1")]);
        assert_eq!(got, Some(Action::Upload { path: p("a"), base: None }));
    }

    #[test]
    fn plans_every_path_independently() {
        let actions = plan(
            &files(&[("keep", "1"), ("mine", "1")]),
            &files(&[("keep", "1"), ("theirs", "1")]),
            &base(&[("keep", "1")]),
        );
        assert_eq!(actions.len(), 2);
    }
}
