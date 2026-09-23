//! Terminal WebSocket URL builder
//! (port of packages/app/src/utils/terminal-websocket-url.ts).

use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Protocol {
    V1,
    V2,
}

pub struct WsInput {
    pub protocol: Protocol,
    pub url: String,
    pub id: String,
    pub directory: String,
    pub cursor: i64,
    pub same_origin: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth_token: bool,
    pub ticket: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WsUrl {
    pub protocol: String,
    pub username: String,
    pub password: String,
    pub pathname: String,
    pub params: BTreeMap<String, String>,
}

pub fn terminal_websocket_url(input: WsInput) -> WsUrl {
    let is_v1 = input.protocol == Protocol::V1;
    let pathname = if is_v1 {
        format!("/pty/{}/connect", input.id)
    } else {
        format!("/api/pty/{}/connect", input.id)
    };
    let protocol = if input.url.starts_with("https://") {
        "wss:"
    } else {
        "ws:"
    };

    let mut params = BTreeMap::new();
    if is_v1 {
        params.insert("directory".to_string(), input.directory.clone());
    } else {
        params.insert("location[directory]".to_string(), input.directory.clone());
    }
    params.insert("cursor".to_string(), input.cursor.to_string());

    if let Some(ticket) = input.ticket.clone() {
        params.insert("ticket".to_string(), ticket);
        return WsUrl {
            protocol: protocol.to_string(),
            username: String::new(),
            password: String::new(),
            pathname,
            params,
        };
    }

    if is_v1 {
        if let Some(password) = &input.password {
            if !input.same_origin || input.auth_token {
                let username = input
                    .username
                    .clone()
                    .unwrap_or_else(|| "opencode".to_string());
                let token = URL_SAFE_NO_PAD.encode(format!("{username}:{password}"));
                params.insert("auth_token".to_string(), token);
            }
        }
    }

    WsUrl {
        protocol: protocol.to_string(),
        username: String::new(),
        password: String::new(),
        pathname,
        params,
    }
}
