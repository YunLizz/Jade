use std::time::Duration;

pub type PlatformResult<T> = Result<T, PlatformError>;

#[derive(Debug, Clone)]
pub struct PlatformError(pub String);

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Platform error: {}", self.0)
    }
}

impl std::error::Error for PlatformError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformKind {
    Virtual,
    RealWindows,
    RealLinux,
}

impl PlatformKind {
    pub fn label(&self) -> &'static str {
        match self {
            PlatformKind::Virtual => "Virtual Platform (Mock)",
            PlatformKind::RealWindows => "Windows (Real)",
            PlatformKind::RealLinux => "Linux (Real)",
        }
    }
}

pub trait IPlatformBackend: Send + Sync {
    fn platform_name(&self) -> PlatformResult<String>;
    fn system_version(&self) -> PlatformResult<String>;
    fn cpu_core_count(&self) -> PlatformResult<u32>;

    fn high_precision_time(&self) -> PlatformResult<f64>;
    fn frame_interval_sample(&self, frames: u32) -> PlatformResult<f64>;

    fn memory_usage_mb(&self) -> PlatformResult<f64>;
    fn platform_kind(&self) -> PlatformKind;

    fn sleep(&self, duration: Duration) -> PlatformResult<()>;
}
