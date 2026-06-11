use std::time::{Duration, Instant};

use crate::platform::{
    IPlatformBackend, PlatformError, PlatformKind, PlatformResult,
};

pub struct MockPlatform {
    start_time: Instant,
    virtual_memory_mb: f64,
    virtual_cpu_cores: u32,
}

impl MockPlatform {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            virtual_memory_mb: 16.0,
            virtual_cpu_cores: 4,
        }
    }

    fn virtual_time(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}

impl Default for MockPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl IPlatformBackend for MockPlatform {
    fn platform_name(&self) -> PlatformResult<String> {
        Ok("Mock Virtual Platform".to_string())
    }

    fn system_version(&self) -> PlatformResult<String> {
        Ok("v0.1.0-simulated".to_string())
    }

    fn cpu_core_count(&self) -> PlatformResult<u32> {
        Ok(self.virtual_cpu_cores)
    }

    fn high_precision_time(&self) -> PlatformResult<f64> {
        Ok(self.virtual_time())
    }

    fn frame_interval_sample(&self, frames: u32) -> PlatformResult<f64> {
        if frames == 0 {
            return Err(PlatformError("frame count must be > 0".to_string()));
        }
        let simulated_interval = 1.0 / 60.0;
        let jitter = (self.virtual_time().fract() - 0.5) * 0.002;
        Ok((simulated_interval + jitter).max(0.001) / frames as f64 * frames as f64)
    }

    fn memory_usage_mb(&self) -> PlatformResult<f64> {
        let growth = (self.virtual_time() * 0.01).sin().abs() * 2.0;
        Ok(self.virtual_memory_mb + growth)
    }

    fn platform_kind(&self) -> PlatformKind {
        PlatformKind::Virtual
    }

    fn sleep(&self, duration: Duration) -> PlatformResult<()> {
        std::thread::sleep(duration);
        Ok(())
    }
}
