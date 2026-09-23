//! Shared recording transport for the typed-client tests.

#![allow(dead_code)]

use futures::future::BoxFuture;
use opencode_client::{ClientError, Transport, TransportRequest, TransportResponse};
use std::sync::{Arc, Mutex};

pub struct FakeTransport {
    routes: Vec<(String, String, u16, String)>,
    requests: Mutex<Vec<(String, String)>>,
}

impl FakeTransport {
    pub fn new(routes: Vec<(&str, &str, u16, &str)>) -> Arc<Self> {
        Arc::new(Self {
            routes: routes
                .into_iter()
                .map(|(method, url, status, body)| {
                    (
                        method.to_string(),
                        url.to_string(),
                        status,
                        body.to_string(),
                    )
                })
                .collect(),
            requests: Mutex::new(Vec::new()),
        })
    }

    pub fn recorded(&self) -> Vec<(String, String)> {
        self.requests.lock().unwrap().clone()
    }
}

impl Transport for FakeTransport {
    fn request(
        &self,
        request: TransportRequest,
    ) -> BoxFuture<'static, Result<TransportResponse, ClientError>> {
        self.requests
            .lock()
            .unwrap()
            .push((request.method.clone(), request.url.clone()));
        let found = self
            .routes
            .iter()
            .find(|(method, url, _, _)| *method == request.method && *url == request.url)
            .cloned();
        Box::pin(async move {
            match found {
                Some((_, _, status, body)) => Ok(TransportResponse { status, body }),
                None => Err(ClientError::Protocol(format!(
                    "no fake route for {} {}",
                    request.method, request.url
                ))),
            }
        })
    }
}

pub const BASE: &str = "http://localhost:3000";

pub const SESSION_JSON: &str = r#"{"id":"ses_test","projectID":"prj_test","cost":0.0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"time":{"created":1717171717000,"updated":1717171717000},"title":"Test","location":{"directory":"/tmp/project"}}"#;

pub const EVENT_JSON: &str = r#"{"id":"evt_model","type":"session.next.model.switched","durable":{"aggregateID":"ses_test","seq":1,"version":1},"data":{"timestamp":1717171717000,"sessionID":"ses_test","messageID":"msg_model","model":{"id":"claude","providerID":"anthropic"}}}"#;

pub const MESSAGE_JSON: &str = r#"{"id":"msg_model","type":"model-switched","time":{"created":1717171717000,"updated":1717171717000},"model":{"id":"claude","providerID":"anthropic"}}"#;

pub const ADMISSION_JSON: &str = r#"{"admittedSeq":1,"id":"msg_test","sessionID":"ses_test","prompt":{"text":"Hello"},"delivery":"immediate","timeCreated":1717171717000}"#;

pub fn session_page(has_more: bool) -> String {
    format!(r#"{{"data":[{SESSION_JSON}],"cursor":{{"next":"next"}},"hasMore":{has_more}}}"#)
}

pub fn history_page(has_more: bool) -> String {
    format!(r#"{{"data":[{EVENT_JSON}],"hasMore":{has_more}}}"#)
}

pub fn standard_routes() -> Vec<(&'static str, &'static str, u16, &'static str)> {
    vec![
        (
            "GET",
            "http://localhost:3000/api/session?limit=10",
            200,
            r#"{"data":[{"id":"ses_test","projectID":"prj_test","cost":0.0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"time":{"created":1717171717000,"updated":1717171717000},"title":"Test","location":{"directory":"/tmp/project"}}],"cursor":{"next":"next"}}"#,
        ),
        (
            "GET",
            "http://localhost:3000/api/session?limit=10&order=desc",
            200,
            r#"{"data":[{"id":"ses_test","projectID":"prj_test","cost":0.0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"time":{"created":1717171717000,"updated":1717171717000},"title":"Test","location":{"directory":"/tmp/project"}}],"cursor":{"next":"next"}}"#,
        ),
        (
            "GET",
            "http://localhost:3000/api/session/active",
            200,
            r#"{"ses_test":{"type":"running"}}"#,
        ),
        (
            "POST",
            "http://localhost:3000/api/session",
            200,
            r#"{"id":"ses_test","projectID":"prj_test","cost":0.0,"tokens":{"input":1,"output":2,"reasoning":0,"cache":{"read":0,"write":0}},"time":{"created":1717171717000,"updated":1717171717000},"title":"Test","location":{"directory":"/tmp/project"}}"#,
        ),
        ("POST", "http://localhost:3000/api/session/ses_test/agent", 200, "{}"),
        ("POST", "http://localhost:3000/api/session/ses_test/model", 200, "{}"),
        (
            "POST",
            "http://localhost:3000/api/session/ses_test/prompt",
            200,
            r#"{"admittedSeq":1,"id":"msg_test","sessionID":"ses_test","prompt":{"text":"Hello"},"delivery":"immediate","timeCreated":1717171717000}"#,
        ),
        (
            "POST",
            "http://localhost:3000/api/session/ses_test/compact",
            200,
            "{}",
        ),
        ("POST", "http://localhost:3000/api/session/ses_test/wait", 200, "{}"),
        (
            "GET",
            "http://localhost:3000/api/session/ses_test/context",
            200,
            "[]",
        ),
        (
            "GET",
            "http://localhost:3000/api/session/ses_test/history?limit=1&after=0",
            200,
            r#"{"data":[{"id":"evt_model","type":"session.next.model.switched","durable":{"aggregateID":"ses_test","seq":1,"version":1},"data":{"timestamp":1717171717000,"sessionID":"ses_test","messageID":"msg_model","model":{"id":"claude","providerID":"anthropic"}}}],"hasMore":true}"#,
        ),
        (
            "GET",
            "http://localhost:3000/api/session/ses_test/history?limit=2&after=1",
            200,
            r#"{"data":[{"id":"evt_model","type":"session.next.model.switched","durable":{"aggregateID":"ses_test","seq":1,"version":1},"data":{"timestamp":1717171717000,"sessionID":"ses_test","messageID":"msg_model","model":{"id":"claude","providerID":"anthropic"}}}],"hasMore":false}"#,
        ),
        (
            "GET",
            "http://localhost:3000/api/session/ses_test/event?after=0",
            200,
            "data: {\"id\":\"evt_model\",\"type\":\"session.next.model.switched\",\"durable\":{\"aggregateID\":\"ses_test\",\"seq\":1,\"version\":1},\"data\":{\"timestamp\":1717171717000,\"sessionID\":\"ses_test\",\"messageID\":\"msg_model\",\"model\":{\"id\":\"claude\",\"providerID\":\"anthropic\"}}}\n\n",
        ),
        (
            "POST",
            "http://localhost:3000/api/session/ses_test/interrupt",
            200,
            "{}",
        ),
        (
            "GET",
            "http://localhost:3000/api/session/ses_test/message/msg_model",
            200,
            r#"{"id":"msg_model","type":"model-switched","time":{"created":1717171717000,"updated":1717171717000},"model":{"id":"claude","providerID":"anthropic"}}"#,
        ),
    ]
}
