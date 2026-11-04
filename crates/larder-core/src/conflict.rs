use crate::paths::RelPath;

/// "report.pdf" -> "report (conflict, laptop, 2026-11-04).pdf", same folder.
pub fn conflict_name(path: &RelPath, device: &str, date: &str) -> RelPath {
    let raw = path.as_str();
    let (dir, file) = match raw.rsplit_once('/') {
        Some((dir, file)) => (Some(dir), file),
        None => (None, raw),
    };
    // Dotfiles like ".env" have no extension, the dot is part of the name.
    let (stem, ext) = match file.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem, Some(ext)),
        _ => (file, None),
    };
    let device: String = device.chars().filter(|c| c.is_alphanumeric() || *c == '-' || *c == ' ').collect();
    let mut name = format!("{stem} (conflict, {}, {date})", device.trim());
    if let Some(ext) = ext {
        name = format!("{name}.{ext}");
    }
    let full = match dir {
        Some(dir) => format!("{dir}/{name}"),
        None => name,
    };
    RelPath::parse(&full).expect("conflict name built from a valid path")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> RelPath {
        RelPath::parse(s).unwrap()
    }

    #[test]
    fn conflict_names_keep_folder_and_extension() {
        let name = conflict_name(&p("docs/report.final.pdf"), "laptop", "2026-11-04");
        assert_eq!(name.as_str(), "docs/report.final (conflict, laptop, 2026-11-04).pdf");
        assert_eq!(conflict_name(&p(".env"), "pc", "d").as_str(), ".env (conflict, pc, d)");
        assert_eq!(conflict_name(&p("Makefile"), "my:pc", "d").as_str(), "Makefile (conflict, mypc, d)");
    }
}
