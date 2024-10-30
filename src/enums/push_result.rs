use std::time::Duration;

#[derive(Debug, Clone)]
pub enum PushResult {
    Skip,
    Success(f64, u64, Duration, usize), // transfer_rate, bytes, duration, file_count
    SuccessDirectory(f64, u64, Duration, usize),
    FailedAllPush(String),
}

impl std::fmt::Display for PushResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PushResult::Skip => write!(f, "0 files pushed. 1 file skipped."),
            PushResult::Success(transfer_rate, bytes_transferred, duration, _) => write!(
                f,
                "1 file pushed. {:.1} MB/s ({} bytes in {:.3}s)",
                transfer_rate,
                bytes_transferred,
                duration.as_secs_f64()
            ),
            PushResult::SuccessDirectory(transfer_rate, bytes_transferred, duration, file_count) => write!(
                f,
                "{} files pushed. {:.1} MB/s ({} bytes in {:.3}s)",
                file_count,
                transfer_rate,
                bytes_transferred,
                duration.as_secs_f64()
            ),
            PushResult::FailedAllPush(err_msg) => write!(f, "{}", err_msg)
        }
    }
}