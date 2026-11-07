//! Paths inside a project: relative to the `.sspj`'s directory, and `/`-separated on
//! every host.

pub const ASSET_PATH_SEPARATOR: char = '/';

pub fn normalize_asset_path(path: &str) -> String {
    let rooted = path.starts_with(ASSET_PATH_SEPARATOR);

    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split(ASSET_PATH_SEPARATOR) {
        match segment {
            "" | "." => {}
            ".." if segments.last().is_some_and(|last| *last != "..") => {
                segments.pop();
            }
            ".." if rooted => {}
            segment => segments.push(segment),
        }
    }

    let mut out = String::with_capacity(path.len());
    if rooted {
        out.push(ASSET_PATH_SEPARATOR);
    }
    for (i, segment) in segments.iter().enumerate() {
        if i > 0 {
            out.push(ASSET_PATH_SEPARATOR);
        }
        out.push_str(segment);
    }
    out
}

pub fn join_asset_path(base: &str, name: &str) -> String {
    if base.is_empty() {
        return normalize_asset_path(name);
    }
    normalize_asset_path(&format!("{}{}{}", base, ASSET_PATH_SEPARATOR, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_asset_path_always_uses_forward_slash() {
        assert_eq!(join_asset_path("images", "chara.png"), "images/chara.png");
        assert_eq!(join_asset_path("font/nested", "mplus_0.png"), "font/nested/mplus_0.png");
        // An empty base (a file sitting at the project root) must not introduce a leading separator.
        assert_eq!(join_asset_path("", "chara.png"), "chara.png");
        // A base already carrying a trailing separator must not double it.
        assert_eq!(join_asset_path("images/", "chara.png"), "images/chara.png");
        // A base the filesystem would resolve away must not survive into the key.
        assert_eq!(join_asset_path(".", "chara.png"), "chara.png");
        assert_eq!(join_asset_path("./images", "chara.png"), "images/chara.png");
        assert_eq!(join_asset_path("images//", "chara.png"), "images/chara.png");
    }

    #[test]
    fn normalize_asset_path_resolves_what_the_filesystem_would() {
        assert_eq!(normalize_asset_path("chara.png"), "chara.png");
        assert_eq!(normalize_asset_path("./chara.png"), "chara.png");
        assert_eq!(normalize_asset_path("images//chara.png"), "images/chara.png");
        assert_eq!(normalize_asset_path("images/./chara.png"), "images/chara.png");
        assert_eq!(normalize_asset_path("images/sub/../chara.png"), "images/chara.png");
        assert_eq!(normalize_asset_path(""), "");
        assert_eq!(normalize_asset_path("."), "");
    }

    #[test]
    fn normalize_asset_path_keeps_what_leaves_the_project() {
        assert_eq!(normalize_asset_path("../chara.png"), "../chara.png");
        assert_eq!(normalize_asset_path("images/../../chara.png"), "../chara.png");
        assert_eq!(normalize_asset_path("/chara.png"), "/chara.png");
        assert_eq!(normalize_asset_path("/../chara.png"), "/chara.png");
    }
}
