use crate::settings::Account;
use chrono::{Local, Timelike};

pub mod qrcode;

use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha1, DEFAULT_STEP};

pub fn maybe_update_otps<'i>(
    otps: impl Iterator<Item = (&'i String, &'i mut Account)>,
) -> (f64, f64) {
    let now = Local::now();
    let mut seconds =
        ((now.time().second() * 1000) + (now.time().nanosecond() / 1000000)) as f64 / 1000.0;
    if seconds > 30.0 {
        seconds -= 30.0
    }
    if seconds < 2.0 {
        // println!("here");
        update_otps(otps);
    }
    (seconds, 100.0 - (100.0 / 30.0) * seconds)
}

pub fn update_otps<'i>(otps: impl Iterator<Item = (&'i String, &'i mut Account)>) {
    otps.for_each(|(_, account)| set_otp(account));
}

pub fn set_otp(account: &mut Account) {
    account.code = get_otp(account);
}

pub fn get_otp(account: &Account) -> String {
    totp_custom::<Sha1>(
        DEFAULT_STEP,
        account.length,
        &account.secret.1,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time sice Unix epoch")
            .as_secs(),
    )
}
