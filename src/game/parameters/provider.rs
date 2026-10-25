//! Where a set of parameters comes from.

use super::GameParameters;

/// Where a [`GameParameters`] came from, so callers can report and choose it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterSource {
    /// Values transcribed from the decompiled original into this file.
    TranscribedOriginal,
}

/// Anything that can supply a complete parameter set (hardcoded today, the SWF later).
pub trait ParameterProvider {
    fn source(&self) -> ParameterSource;
    fn parameters(&self) -> GameParameters;
}

pub struct TranscribedOriginal;

impl ParameterProvider for TranscribedOriginal {
    fn source(&self) -> ParameterSource {
        ParameterSource::TranscribedOriginal
    }

    fn parameters(&self) -> GameParameters {
        GameParameters::ORIGINAL
    }
}
