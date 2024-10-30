use std::time::Duration;


#[allow(dead_code)]
#[derive(Debug)]
pub enum PullResult {
    Success(f64, u64, Duration, u32),
    SuccessDirectory(f64, u64, Duration, u32),
    FailedAllPull(String),
}