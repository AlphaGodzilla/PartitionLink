use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ts() -> anyhow::Result<u128> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(now.as_millis())
}
