//! Utility functions and modules
mod checksum;
mod files;
mod fingerprint;
mod hash;
mod normalize;
mod normalize;
mod secure_deletion;

pub use checksum::*;
pub use files::*;
pub use fingerprint::*;
pub use hash::*;
pub use normalize::*;
pub use normalize::*;
pub use secure_deletion::*;

use crate::analyze::AnalyzeError;
use std::path::PathBuf;
