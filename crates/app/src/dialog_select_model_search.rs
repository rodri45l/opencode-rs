//! Model search matching (port of
//! packages/app/src/components/dialog-select-model-search.ts).

fn normalize(value: &str) -> String {
    let lower = value.to_lowercase();
    let mut out = String::new();
    let mut previous_space = false;
    for character in lower.chars() {
        if character.is_alphanumeric() {
            out.push(character);
            previous_space = false;
        } else if !previous_space && !out.is_empty() {
            out.push(' ');
            previous_space = true;
        }
    }
    out.trim().to_string()
}

fn compact(value: &str) -> String {
    normalize(value).replace(' ', "")
}

pub fn matches_model_search(search: &str, fields: &[&str]) -> bool {
    let query = normalize(search);
    if query.is_empty() {
        return true;
    }
    let compact_query = compact(search);
    fields
        .iter()
        .any(|field| normalize(field).contains(&query) || compact(field).contains(&compact_query))
}
