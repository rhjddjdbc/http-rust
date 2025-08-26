use chrono::Utc;
use std::path::{Component, Path, PathBuf};
use std::env;
pub fn resolve_safe_path<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let path = path.as_ref();

    let sanitized = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(c) => Some(c),
            _ => None,
        })
        .collect::<PathBuf>();

    let base = env::current_dir().unwrap().join("public");
    let resolved_path = base.join(sanitized);

    if resolved_path.exists() {
        Some(resolved_path)
    } else {
        None
    }
}

pub fn http_date() -> String {
    Utc::now().to_rfc2822()
}
