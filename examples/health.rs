use std::sync::Arc;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    SqliteConnection,
};
use valensas_actuator::{actuate, health};

// Custom health indicator
struct MyHealthIndicator {
    up: bool,
}

#[async_trait::async_trait]
impl health::Indicator for MyHealthIndicator {
    fn name(&self) -> &str {
        "my_custom_health_indicator"
    }

    async fn check(&self) -> Result<(), String> {
        if self.up {
            Ok(())
        } else {
            Err("Something fishy is going on".to_string())
        }
    }
}

#[tokio::main]
async fn main() {
    let rocket = rocket::build();

    let my_indicator_up = Box::new(MyHealthIndicator { up: true });
    let my_indicator_down = Box::new(MyHealthIndicator { up: false });

    let manager = ConnectionManager::<SqliteConnection>::new("test.db");
    let pool = Arc::new(
        Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .unwrap(),
    );

    let diesel_indicator =
        valensas_actuator::health::check::Diesel::new("diesel".to_string(), move || {
            Ok(pool.get().map_err(|e| e.to_string())?)
        });

    actuate(rocket)
        .with_liveness(health::check::Liveness::new(vec![
            my_indicator_up,
            Box::new(diesel_indicator.clone()),
        ]))
        .with_readiness(health::check::Readiness::new(vec![
            my_indicator_down,
            Box::new(diesel_indicator),
        ]))
        .get()
        .ignite()
        .await
        .unwrap()
        .launch()
        .await
        .unwrap();
}
