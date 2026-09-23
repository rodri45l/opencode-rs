//! mDNS publish decisions for the server listener.
//!
//! Re-derived from `packages/opencode/src/server/server.ts` and
//! `packages/opencode/src/server/mdns.ts` (upstream 18ef3cc). The Bonjour
//! transport is not ported; this module carries the observable decision of
//! whether a listener should advertise itself.

/// The loopback hostnames that never publish an mDNS record.
const LOOPBACK_HOSTNAMES: &[&str] = &["127.0.0.1", "localhost", "::1"];

/// Whether a listener with the given options should publish an mDNS record.
///
/// Requires `mdns` to be enabled, a bound port, and a non-loopback hostname.
pub fn should_publish(mdns: bool, hostname: &str, port: u16) -> bool {
    mdns && port != 0 && !LOOPBACK_HOSTNAMES.contains(&hostname)
}

/// The advertised service name for a bound port.
pub fn service_name(port: u16) -> String {
    format!("opencode-{port}")
}
