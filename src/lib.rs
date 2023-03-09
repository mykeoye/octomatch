use std::error::Error;

pub mod core;

pub struct EngineConfig {}

pub trait Engine {
    fn start(config: EngineConfig) -> Result<(), Box<dyn Error>>;

    fn stop();
}
