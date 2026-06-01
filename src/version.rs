use std::env::consts;

pub const VERSION: &str = "0.2.1";
pub const BUILD_TIME: &str = "2026-05-31";

pub fn string() -> String {
    format!(
        "backup version v{}\nbuild: {}\nrust: {}\nplatform: {}/{}",
        VERSION,
        BUILD_TIME,
        env!("RUSTC_VERSION"),
        consts::OS,
        consts::ARCH
    )
}

pub fn short() -> String {
    format!("v{}", VERSION)
}
