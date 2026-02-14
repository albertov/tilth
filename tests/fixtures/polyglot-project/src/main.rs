pub struct Config {
    pub name: String,
}

pub fn initialize() -> Config {
    Config {
        name: String::new(),
    }
}

pub enum Status {
    Active,
    Inactive,
}
