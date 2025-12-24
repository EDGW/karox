//! Paging

use crate::{arch::mm::config::Paging, mm::PagingMode};

/// Initializes the paging system.
///
/// Calls the architecture-specific paging initialization.
pub fn init_paging() {
    Paging::init();
}
