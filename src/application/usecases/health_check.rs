use tracing::info;

pub struct HealthCheck;

impl HealthCheck {
    pub fn execute(&self) -> &'static str {
        info!("Health check requested");
        "OK"
    }
}
