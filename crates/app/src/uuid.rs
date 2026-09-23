//! UUID generation with insecure-context fallback
//! (port of packages/app/src/utils/uuid.ts).

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RandomUuid {
    Available(&'static str),
    Throws,
    Missing,
}

/// Emulate `Math.random().toString(16).slice(2)` for the fallback value.
fn fallback(random: f64) -> String {
    let integer = random.trunc();
    let mut fraction = random.fract();
    if fraction == 0.0 {
        return String::new();
    }
    let mut out = String::new();
    for _ in 0..13 {
        fraction *= 16.0;
        let digit = fraction.trunc();
        out.push(std::char::from_digit(digit as u32, 16).unwrap_or('0'));
        fraction -= digit;
        if fraction == 0.0 {
            break;
        }
    }
    let _ = integer;
    out
}

/// Resolve a UUID given the ambient runtime capability.
pub fn uuid(secure: bool, random_uuid: RandomUuid, random: f64) -> String {
    match random_uuid {
        RandomUuid::Available(value) => {
            if secure {
                value.to_string()
            } else {
                fallback(random)
            }
        }
        RandomUuid::Throws | RandomUuid::Missing => fallback(random),
    }
}
