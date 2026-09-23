//! Optimistic message overlays (port of packages/app/src/context/sync.tsx).

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub created: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub id: String,
    pub session_id: String,
    pub message_id: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptimisticAdd {
    pub session_id: String,
    pub message: Message,
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptimisticRemove {
    pub session_id: String,
    pub message_id: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Draft {
    pub message: BTreeMap<String, Vec<Message>>,
    pub part: BTreeMap<String, Vec<Part>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PagePart {
    pub id: String,
    pub part: Vec<Part>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FetchedPage {
    pub session: Vec<Message>,
    pub part: Vec<PagePart>,
    pub complete: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MergedPage {
    pub session: Vec<Message>,
    pub part: Vec<PagePart>,
    pub confirmed: Vec<String>,
    pub complete: bool,
}

fn message_key(message: &Message) -> (i64, &str) {
    (message.created, message.id.as_str())
}

fn sort_parts(mut parts: Vec<Part>) -> Vec<Part> {
    parts.retain(|part| !part.id.is_empty());
    parts.sort_by(|a, b| a.id.cmp(&b.id));
    parts
}

fn insert_sorted(messages: &mut Vec<Message>, message: Message) {
    let key = message_key(&message);
    let index = messages
        .iter()
        .position(|item| message_key(item) > key)
        .unwrap_or(messages.len());
    messages.insert(index, message);
}

pub fn apply_optimistic_add(draft: &mut Draft, input: OptimisticAdd) {
    let messages = draft.message.entry(input.session_id.clone()).or_default();
    insert_sorted(messages, input.message.clone());
    draft
        .part
        .insert(input.message.id.clone(), sort_parts(input.parts));
}

pub fn apply_optimistic_remove(draft: &mut Draft, input: OptimisticRemove) {
    if let Some(messages) = draft.message.get_mut(&input.session_id) {
        messages.retain(|message| message.id != input.message_id);
    }
    draft.part.remove(&input.message_id);
}

fn has_parts(parts: Option<&Vec<Part>>, want: &[Part]) -> bool {
    match parts {
        None => want.is_empty(),
        Some(parts) => want
            .iter()
            .all(|part| parts.iter().any(|item| item.id == part.id)),
    }
}

fn merge_parts(parts: Option<&Vec<Part>>, want: &[Part]) -> Vec<Part> {
    let mut next = match parts {
        None => return sort_parts(want.to_vec()),
        Some(parts) => parts.clone(),
    };
    for part in want {
        if next.iter().any(|item| item.id == part.id) {
            continue;
        }
        next.push(part.clone());
    }
    sort_parts(next)
}

pub fn merge_optimistic_page(page: FetchedPage, items: &[OptimisticAdd]) -> MergedPage {
    if items.is_empty() {
        return MergedPage {
            session: page.session,
            part: page.part,
            confirmed: Vec::new(),
            complete: page.complete,
        };
    }

    let mut session = page.session;
    let mut part: BTreeMap<String, Vec<Part>> = page
        .part
        .into_iter()
        .map(|item| (item.id, sort_parts(item.part)))
        .collect();
    let mut confirmed = Vec::new();

    for item in items {
        let key = message_key(&item.message);
        let found = session.iter().any(|message| message_key(message) == key);
        if !found {
            insert_sorted(&mut session, item.message.clone());
        }

        let current = part.get(&item.message.id);
        if found && has_parts(current, &item.parts) {
            confirmed.push(item.message.id.clone());
            continue;
        }
        part.insert(item.message.id.clone(), merge_parts(current, &item.parts));
    }

    MergedPage {
        session,
        part: part
            .into_iter()
            .map(|(id, part)| PagePart { id, part })
            .collect(),
        confirmed,
        complete: page.complete,
    }
}
