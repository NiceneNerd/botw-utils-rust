use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

pub mod extensions;
pub mod hashes;

/// Convert a path relative to a BOTW content root into a [canonical resource
/// path](https://zeldamods.org/wiki/Canonical_resource_path). Example:
///
/// ```
/// use botw_utils::get_canon_name;
/// assert_eq!(
///    get_canon_name("content\\Actor\\Pack\\Enemy_Lizalfos_Senior.sbactorpack").unwrap(),
///    "Actor/Pack/Enemy_Lizalfos_Senior.bactorpack"
/// );
/// ```
///
/// # Arguments
///
/// * `file_path` - The path of the BOTW game file relative to the root folder
///
/// # Returns
///
/// Returns an Option with the canonical resource path as a String or None if the path does not
/// appear valid
pub fn get_canon_name<P: AsRef<Path>>(file_path: P) -> Option<String> {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            "(?i)(((Content|(atmosphere/(titles|contents)/)?01007EF00011E000/romfs)/)|\
        ((Aoc(/0010)?|(atmosphere/(titles|contents)/)?01007EF00011[ef]00[0-2]/romfs)/))",
        )
        .unwrap()
    });
    let mut normalized = file_path
        .as_ref()
        .to_string_lossy()
        .replace('\\', "/")
        .replace(".s", ".");
    normalized = RE
        .replace_all(&normalized, |caps: &regex::Captures| {
            if caps[0].starts_with("content") || caps[0].contains("01007EF00011E000") {
                "content/"
            } else {
                "aoc/0010/"
            }
        })
        .to_string();
    if normalized.starts_with("aoc/") {
        Some(
            normalized
                .replace("aoc/content", "Aoc")
                .replace("aoc", "Aoc"),
        )
    } else if normalized.starts_with("content") && !normalized.contains("/aoc") {
        Some(normalized.replace("content/", ""))
    } else {
        None
    }
}

/// Convert a BOTW game resource path without a root folder into a [canonical resource
/// path](https://zeldamods.org/wiki/Canonical_resource_path). Most useful for normalizing paths
/// to resources inside of SARC archives. Example:
///
/// ```
/// use botw_utils::get_canon_name_without_root;
/// assert_eq!(
///    get_canon_name_without_root("Actor/Pack/GameROMPlayer.sbactorpack"),
///    "Actor/Pack/GameROMPlayer.bactorpack"
/// );
/// ```
///
/// # Arguments
///
/// * `file_path` - The path of the BOTW game file, not relative to a root folder but bare. In most
///   cases this would only come from a path inside of a SARC.
///
/// # Returns
///
/// Returns the canonical resource path as a String.
pub fn get_canon_name_without_root<P: AsRef<Path>>(file_path: P) -> String {
    file_path
        .as_ref()
        .to_string_lossy()
        .replace('\\', "/")
        .replace(".s", ".")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canon_names() {
        assert_eq!(
            get_canon_name("content\\Actor\\Pack\\Enemy_Lizal_Senior.sbactorpack",).unwrap(),
            "Actor/Pack/Enemy_Lizal_Senior.bactorpack"
        );
        assert_eq!(
            get_canon_name("aoc/0010/Map/MainField/A-1/A-1_Dynamic.smubin",).unwrap(),
            "Aoc/0010/Map/MainField/A-1/A-1_Dynamic.mubin"
        );
        assert_eq!(
            get_canon_name(
                "atmosphere/contents/01007EF00011E000/romfs/Actor/ActorInfo.product.sbyml",
            )
            .unwrap(),
            "Actor/ActorInfo.product.byml"
        );
        assert_eq!(
            get_canon_name("atmosphere/contents/01007EF00011F001/romfs/Pack/AocMainField.pack",)
                .unwrap(),
            "Aoc/0010/Pack/AocMainField.pack"
        );
        assert_eq!(get_canon_name("Hellow/Sweetie.tardis"), None);
        assert_eq!(
            get_canon_name_without_root("Event/EventInfo.product.sbyml"),
            "Event/EventInfo.product.byml"
        )
    }
}
