use dotenv::dotenv;
use std::env;

pub struct AppConfig {
    pub database_url: String,
}

pub fn load() -> AppConfig {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    //Open a connection pool

    AppConfig { database_url }
}
