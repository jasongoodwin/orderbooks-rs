//! contains results, errors, and useful type aliases.
use std::result::Result as StdResult;

/// Result is an type alias for a Result<T, Error> to reduce type noise.
/// This is a common pattern. It really does simplify the types in your project!
pub type Result<T> = StdResult<T, Box<dyn std::error::Error + Send + Sync>>;
