use crate::errors::AppError;
use crate::services::email_provider::EmailProvider;
use anyhow::Result;
use std::sync::Arc;
use tracing::{info};

pub struct EmailService {
    email: Arc<dyn EmailProvider>,
}

impl EmailService {
    pub fn new(email: Arc<dyn EmailProvider>) -> Self {
        Self { email }
    }

    pub async fn send_email(&self, to: &str, password: &str) -> Result<()> {
        self.email
            .send_email(to, password)
            .await
            .map_err(AppError::Internal)?;

        info!(target: "email", to = %to, "Email sent");
        Ok(())
    }
}
