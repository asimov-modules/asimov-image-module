// src/core.rs
// This is free and unencumbered software released into the public domain.

use std::error::Error;

/// Helper to build a boxed error from a string.
pub fn err_msg<M: Into<String>>(m: M) -> Box<dyn Error> {
    m.into().into()
}
