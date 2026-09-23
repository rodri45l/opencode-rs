//! Port of packages/opencode/test/cli/run/question.shared.test.ts (upstream 18ef3cc).
//!
//! RED-first: the run question-body state machine in
//! `cli/cmd/run/question.shared` is not implemented in this crate. Reference
//! transitions are pinned against local typed stubs.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuestionOption {
    label: String,
    description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Question {
    question: String,
    header: String,
    options: Vec<QuestionOption>,
    multiple: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuestionRequest {
    id: String,
    session_id: String,
    questions: Vec<Question>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuestionBodyState {
    request_id: String,
    tab: usize,
    answers: Vec<Vec<String>>,
    editing: bool,
    selected: usize,
    custom: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QuestionReply {
    request_id: String,
    answers: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectResult {
    state: QuestionBodyState,
    reply: Option<QuestionReply>,
}

fn create_question_body_state(id: &str) -> QuestionBodyState {
    QuestionBodyState {
        request_id: id.to_string(),
        tab: 0,
        answers: Vec::new(),
        editing: false,
        selected: 0,
        custom: Vec::new(),
    }
}

fn question_single(req: &QuestionRequest) -> bool {
    req.questions.len() == 1 && !req.questions[0].multiple
}

fn question_info<'a>(req: &'a QuestionRequest, state: &QuestionBodyState) -> Option<&'a Question> {
    req.questions.get(state.tab)
}

fn question_custom(req: &QuestionRequest, state: &QuestionBodyState) -> bool {
    question_info(req, state).is_some()
}

fn question_total(req: &QuestionRequest, state: &QuestionBodyState) -> usize {
    match question_info(req, state) {
        None => 0,
        Some(info) => info.options.len() + if question_custom(req, state) { 1 } else { 0 },
    }
}

fn question_other(req: &QuestionRequest, state: &QuestionBodyState) -> bool {
    match question_info(req, state) {
        Some(info) => state.selected == info.options.len(),
        None => false,
    }
}

fn question_input(state: &QuestionBodyState) -> String {
    state.custom.get(state.tab).cloned().unwrap_or_default()
}

fn question_set_tab(state: QuestionBodyState, tab: usize) -> QuestionBodyState {
    QuestionBodyState {
        tab,
        selected: 0,
        editing: false,
        ..state
    }
}

fn question_set_editing(state: QuestionBodyState, editing: bool) -> QuestionBodyState {
    QuestionBodyState { editing, ..state }
}

fn question_store_answers(
    state: QuestionBodyState,
    tab: usize,
    list: Vec<String>,
) -> QuestionBodyState {
    let mut answers = state.answers.clone();
    if answers.len() <= tab {
        answers.resize(tab + 1, Vec::new());
    }
    answers[tab] = list;
    QuestionBodyState { answers, ..state }
}

fn question_pick(
    state: QuestionBodyState,
    req: &QuestionRequest,
    answer: String,
    custom: bool,
) -> SelectResult {
    let tab = state.tab;
    let mut next = question_store_answers(state, tab, vec![answer.clone()]);
    next.editing = false;
    if custom {
        let mut list = next.custom.clone();
        if list.len() <= next.tab {
            list.resize(next.tab + 1, String::new());
        }
        list[next.tab] = answer.clone();
        next.custom = list;
    }
    if question_single(req) {
        return SelectResult {
            state: next,
            reply: Some(QuestionReply {
                request_id: req.id.clone(),
                answers: vec![vec![answer]],
            }),
        };
    }
    let tab = next.tab + 1;
    SelectResult {
        state: question_set_tab(next, tab),
        reply: None,
    }
}

fn question_toggle(state: QuestionBodyState, answer: &str) -> QuestionBodyState {
    let mut list = state.answers.get(state.tab).cloned().unwrap_or_default();
    if let Some(index) = list.iter().position(|item| item == answer) {
        list.remove(index);
    } else {
        list.push(answer.to_string());
    }
    let tab = state.tab;
    question_store_answers(state, tab, list)
}

fn question_select(state: QuestionBodyState, req: &QuestionRequest) -> SelectResult {
    let Some(info) = question_info(req, &state).cloned() else {
        return SelectResult { state, reply: None };
    };
    if question_other(req, &state) {
        if !info.multiple {
            return SelectResult {
                state: question_set_editing(state, true),
                reply: None,
            };
        }
        let value = question_input(&state);
        let picked = !value.is_empty()
            && state
                .answers
                .get(state.tab)
                .map(|answers| answers.contains(&value))
                .unwrap_or(false);
        if picked {
            return SelectResult {
                state: question_toggle(state, &value),
                reply: None,
            };
        }
        return SelectResult {
            state: question_set_editing(state, true),
            reply: None,
        };
    }
    let Some(option) = info.options.get(state.selected) else {
        return SelectResult { state, reply: None };
    };
    if info.multiple {
        return SelectResult {
            state: question_toggle(state, &option.label),
            reply: None,
        };
    }
    question_pick(state, req, option.label.clone(), false)
}

fn question_set_selected(state: QuestionBodyState, index: usize) -> QuestionBodyState {
    QuestionBodyState {
        selected: index,
        ..state
    }
}

fn question_confirm(req: &QuestionRequest, state: &QuestionBodyState) -> bool {
    !question_single(req) && state.tab == req.questions.len()
}

fn question_submit(req: &QuestionRequest, state: &QuestionBodyState) -> Option<QuestionReply> {
    let answers: Vec<Vec<String>> = (0..req.questions.len())
        .map(|index| state.answers.get(index).cloned().unwrap_or_default())
        .collect();
    Some(QuestionReply {
        request_id: req.id.clone(),
        answers,
    })
}

fn question_store_custom(state: QuestionBodyState, index: usize, text: &str) -> QuestionBodyState {
    let mut custom = state.custom.clone();
    if custom.len() <= index {
        custom.resize(index + 1, String::new());
    }
    custom[index] = text.to_string();
    QuestionBodyState { custom, ..state }
}

fn question_save(state: QuestionBodyState, req: &QuestionRequest) -> SelectResult {
    let Some(info) = question_info(req, &state).cloned() else {
        return SelectResult { state, reply: None };
    };
    let value = question_input(&state).trim().to_string();
    let prev = state.custom.get(state.tab).cloned().unwrap_or_default();
    if value.is_empty() {
        if prev.is_empty() {
            return SelectResult {
                state: question_set_editing(state, false),
                reply: None,
            };
        }
        let tab = state.tab;
        let next = question_store_custom(state, tab, "");
        let remaining = next
            .answers
            .get(next.tab)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|item| item != &prev)
            .collect();
        let tab = next.tab;
        return SelectResult {
            state: question_set_editing(question_store_answers(next, tab, remaining), false),
            reply: None,
        };
    }
    if info.multiple {
        let mut answers = state.answers.get(state.tab).cloned().unwrap_or_default();
        if !prev.is_empty() {
            if let Some(index) = answers.iter().position(|item| item == &prev) {
                answers.remove(index);
            }
        }
        if !answers.contains(&value) {
            answers.push(value.clone());
        }
        let tab = state.tab;
        let next = question_store_custom(state, tab, &value);
        return SelectResult {
            state: question_set_editing(question_store_answers(next, tab, answers), false),
            reply: None,
        };
    }
    question_pick(state, req, value, true)
}

fn question_sync(state: QuestionBodyState, id: &str) -> QuestionBodyState {
    if state.request_id == id {
        state
    } else {
        create_question_body_state(id)
    }
}

fn question_reject(req: &QuestionRequest) -> QuestionReply {
    QuestionReply {
        request_id: req.id.clone(),
        answers: Vec::new(),
    }
}

fn opt(label: &str, description: &str) -> QuestionOption {
    QuestionOption {
        label: label.to_string(),
        description: description.to_string(),
    }
}

fn base_req() -> QuestionRequest {
    QuestionRequest {
        id: "question-1".to_string(),
        session_id: "session-1".to_string(),
        questions: vec![Question {
            question: "Mode?".to_string(),
            header: "Mode".to_string(),
            options: vec![opt("chunked", "Incremental output")],
            multiple: false,
        }],
    }
}

#[test]
fn replies_immediately_for_a_single_select_question() {
    let out = question_select(create_question_body_state("question-1"), &base_req());
    assert_eq!(
        out.reply,
        Some(QuestionReply {
            request_id: "question-1".to_string(),
            answers: vec![vec!["chunked".to_string()]],
        })
    );
}

#[test]
fn advances_multi_question_flows_and_submits_from_confirm() {
    let ask = QuestionRequest {
        id: "question-1".to_string(),
        session_id: "session-1".to_string(),
        questions: vec![
            Question {
                question: "Mode?".to_string(),
                header: "Mode".to_string(),
                options: vec![opt("chunked", "Incremental output")],
                multiple: false,
            },
            Question {
                question: "Output?".to_string(),
                header: "Output".to_string(),
                options: vec![
                    opt("yes", "Show tool output"),
                    opt("no", "Hide tool output"),
                ],
                multiple: false,
            },
        ],
    };

    let mut state = question_select(create_question_body_state("question-1"), &ask).state;
    assert_eq!(state.tab, 1);

    state = question_set_selected(state, 1);
    state = question_select(state, &ask).state;
    assert!(question_confirm(&ask, &state));
    assert_eq!(
        question_submit(&ask, &state),
        Some(QuestionReply {
            request_id: "question-1".to_string(),
            answers: vec![vec!["chunked".to_string()], vec!["no".to_string()]],
        })
    );
}

#[test]
fn toggles_answers_for_multiple_choice_questions() {
    let ask = QuestionRequest {
        id: "question-1".to_string(),
        session_id: "session-1".to_string(),
        questions: vec![Question {
            question: "Tags?".to_string(),
            header: "Tags".to_string(),
            options: vec![opt("bug", "Bug fix")],
            multiple: true,
        }],
    };

    let mut state = question_select(create_question_body_state("question-1"), &ask).state;
    assert_eq!(state.answers, vec![vec!["bug".to_string()]]);

    state = question_select(state, &ask).state;
    assert_eq!(state.answers, vec![Vec::<String>::new()]);
}

#[test]
fn stores_and_submits_custom_answers() {
    let state = question_set_selected(create_question_body_state("question-1"), 1);
    let next = question_select(state, &base_req());
    assert!(next.state.editing);

    let state = question_store_custom(next.state, 0, "  custom mode  ");
    let next = question_save(state, &base_req());
    assert_eq!(
        next.reply,
        Some(QuestionReply {
            request_id: "question-1".to_string(),
            answers: vec![vec!["custom mode".to_string()]],
        })
    );
}

#[test]
fn resets_state_when_the_request_id_changes_and_builds_reject_payloads() {
    let state = question_set_selected(create_question_body_state("question-1"), 1);

    assert_eq!(question_sync(state.clone(), "question-1"), state);
    assert_eq!(
        question_sync(state, "question-2"),
        create_question_body_state("question-2")
    );
    assert_eq!(
        question_reject(&base_req()),
        QuestionReply {
            request_id: "question-1".to_string(),
            answers: Vec::new(),
        }
    );
}
