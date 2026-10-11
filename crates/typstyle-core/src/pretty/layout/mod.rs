//! Generic layout engines that the converters drive.
//!
//! Each submodule provides a stylist that collects items and trivia, then prints
//! them into one document, applying folding, spacing, and comment handling.

pub mod chain;
pub mod flow;
pub mod list;
pub mod plain;
pub mod table;
