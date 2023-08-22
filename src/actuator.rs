use rocket::{Build, Rocket};

use crate::Actuator;

pub trait RocketConfigurerer {
    fn configure(self, rocket: Rocket<Build>) -> Rocket<Build>;
}

impl Actuator {
    pub fn new(rocket: Rocket<Build>) -> Self {
        Self { rocket }
    }

    pub fn with_configurer<T: RocketConfigurerer>(mut self, configurer: T) -> Actuator {
        self.rocket = configurer.configure(self.rocket);
        self
    }

    pub fn get(self) -> Rocket<Build> {
        self.rocket
    }
}
