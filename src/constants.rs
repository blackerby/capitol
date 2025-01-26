use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

const FIRST_CONGRESS: u64 = 1789;
static CURRENT_YEAR: LazyLock<u64> = LazyLock::new(|| {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap() // TODO: change to expect?
        .as_secs()
        / 31536000 // seconds in year
        + 1970 // UNIX_EPOCH year
});
pub static CURRENT_CONGRESS: LazyLock<u64> =
    LazyLock::new(|| (*CURRENT_YEAR - FIRST_CONGRESS) / 2 + 1);
//const BASE_URL: &str = "https://www.congress.gov";

pub const BILL_VERSIONS: [&[u8]; 37] = [
    b"as", b"ash", b"ath", b"ats", b"cdh", b"cds", b"cph", b"cps", b"eah", b"eas", b"eh", b"enr",
    b"es", b"fph", b"fps", b"hds", b"ih", b"iph", b"ips", b"is", b"lth", b"lts", b"pap", b"pcs",
    b"pp", b"rch", b"rcs", b"rds", b"rfh", b"rfs", b"rh", b"rhuc", b"rih", b"rs", b"rth", b"rts",
    b"sc",
];
