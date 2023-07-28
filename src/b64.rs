use crate::Result;
use std::ops::Not;

use base64::Engine;

pub fn decode(encoded: &str) -> Result<Vec<u8>> {
    let mut encoded = encoded.to_owned();
    encoded.retain(|ch| ch.is_whitespace().not());

    use base64::engine::general_purpose::STANDARD;

    STANDARD.decode(encoded).map_err(Into::into)
}
