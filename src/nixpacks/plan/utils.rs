/// Removes all the `"..."`'s or `"@auto"`'s from the `original`
pub fn remove_autos_from_vec(original: Vec<String>) -> Vec<String> {
    original
        .into_iter()
        .filter(|x| x != "@auto" && x != "...")
        .collect::<Vec<_>>()
}

/// Fills in the `"..."`'s or `"@auto"`'s in `replacer` with the values from the `original`
///
/// ```
/// let arr = fill_auto_in_vec(
///   Some(vec!["a", "b", "c"]),
///   Some(vec!["x", "...", "z"])
/// );
/// assert_eq!(Some(vec!["x", "...", "a", "b", "c", "z"]), arr);
/// ```
pub fn fill_auto_in_vec(
    original: Option<Vec<String>>,
    replacer: Option<Vec<String>>,
) -> Option<Vec<String>> {
    if let Some(replacer) = replacer {
        let original = original.unwrap_or_default();
        let modified = replacer
            .into_iter()
            .flat_map(|x| {
                let v = x.clone();
                if v == *"@auto" || v == *"..." {
                    let mut fill = vec![v];
                    fill.append(&mut original.clone());
                    fill
                } else {
                    vec![x]
                }
            })
            .collect::<Vec<_>>();

        Some(modified)
    } else {
        original
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn vs(v: Vec<&str>) -> Vec<String> {
        v.into_iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    #[test]
    fn test_remove_autos_from_vec() {
        assert_eq!(
            vs(vec!["a", "b", "c"]),
            remove_autos_from_vec(vs(vec!["a", "b", "c"]))
        );
        assert_eq!(
            vs(vec!["a", "c"]),
            remove_autos_from_vec(vs(vec!["a", "...", "c"]))
        );
        assert_eq!(
            vs(vec!["a", "c"]),
            remove_autos_from_vec(vs(vec!["@auto", "a", "...", "c", "@auto"]))
        );
    }

    #[test]
    fn test_fill_auto_in_vec() {
        assert_eq!(
            vec!["x", "...", "z"],
            fill_auto_in_vec(None, Some(vs(vec!["x", "...", "z"]))).unwrap()
        );
        assert_eq!(
            vec!["a", "b", "c"],
            fill_auto_in_vec(Some(vs(vec!["a", "b", "c"])), None).unwrap()
        );
        assert_eq!(
            vec!["x", "...", "a", "b", "c", "z"],
            fill_auto_in_vec(
                Some(vs(vec!["a", "b", "c"])),
                Some(vs(vec!["x", "...", "z"]))
            )
            .unwrap()
        );
    }
}
