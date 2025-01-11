#![allow(dead_code)]

use chrono::Datelike;
use std::sync::LazyLock;

pub(crate) const FIRST_CONGRESS: usize = 1789;
static CURRENT_YEAR: LazyLock<usize> = LazyLock::new(|| chrono::Utc::now().year() as usize);
pub(crate) static CURRENT_CONGRESS: LazyLock<usize> =
    LazyLock::new(|| *CURRENT_YEAR - FIRST_CONGRESS / 2 + 1);

mod legislation;
