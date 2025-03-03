use serde::{Deserialize, Serialize};

use crate::protocol::Consent;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExampleConsentRequired {
    pub consent: Consent,
}
