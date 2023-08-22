use std::collections::HashMap;

use crate::Actuator;

use rocket::{http::Status, serde::json::Json};
use serde::Serialize;

pub mod check;

#[derive(Serialize)]
pub enum IndicatorStatus {
    Up,
    Down,
}

#[derive(Serialize)]
pub struct IndicatorResult {
    status: IndicatorStatus,
    message: Option<String>,
}

impl From<Result<(), String>> for IndicatorResult {
    fn from(value: Result<(), String>) -> Self {
        match value {
            Ok(()) => Self {
                status: IndicatorStatus::Up,
                message: None,
            },
            Err(m) => Self {
                status: IndicatorStatus::Down,
                message: Some(m),
            },
        }
    }
}

#[async_trait::async_trait]
pub trait Indicator: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> Result<(), String>;
}

#[async_trait::async_trait]
pub trait Check {
    fn indicators(&self) -> &Vec<Box<dyn Indicator>>;

    async fn check(
        &self,
    ) -> rocket::response::status::Custom<Json<HashMap<&str, IndicatorResult>>> {
        let mut status = Status::Ok;

        let mut response: HashMap<&str, IndicatorResult> =
            HashMap::with_capacity(self.indicators().len());

        let check_futures =
            futures::future::join_all(self.indicators().iter().map(|i| i.check())).await;

        let indicator_results = self.indicators().iter().zip(check_futures.iter());

        for (i, result) in indicator_results {
            if let Err(_) = result {
                status = Status::ServiceUnavailable;
            };
            response.insert(i.name(), result.clone().into());
        }

        rocket::response::status::Custom(status, Json(response))
    }
}

impl Actuator {
    pub fn with_liveness(self, liveness: check::Liveness) -> Actuator {
        self.with_configurer(liveness)
    }

    pub fn with_readiness(self, readiness: check::Readiness) -> Actuator {
        self.with_configurer(readiness)
    }
}
