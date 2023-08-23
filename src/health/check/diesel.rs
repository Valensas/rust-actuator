use ::diesel::Connection;

use crate::health::{self};

use super::Diesel;

impl<C: Connection, T: (Fn() -> Result<C, String>) + Sync + Send> Diesel<C, T> {
    /// Creates a new diesel health indicator using "select 1" as query.
    pub fn new(name: String, connection_provider: T) -> Self {
        Self {
            name,
            query: "select 1".to_string(),
            connection_provider,
        }
    }

    /// Creates a new diesel health indicator with a custom query.
    pub fn new_with_query(name: String, query: String, connection_provider: T) -> Self {
        Self {
            name,
            query,
            connection_provider,
        }
    }
}

#[async_trait::async_trait]
impl<C: Connection, T: (Fn() -> Result<C, String>) + Sync + Send> health::Indicator
    for Diesel<C, T>
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn check(&self) -> Result<(), String> {
        let mut conn = (self.connection_provider)()?;

        conn.batch_execute(&self.query).map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl<C: Connection, T: (Fn() -> Result<C, String>) + Sync + Send + Clone> Clone for Diesel<C, T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            query: self.query.clone(),
            connection_provider: self.connection_provider.clone(),
        }
    }
}
