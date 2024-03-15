use dotenvy::dotenv;
use std::net::{Ipv4Addr, SocketAddrV4};

pub struct ServerConfig {
    pub port: u16,
    pub database_url: String,
    pub salt: Box<[u8]>,
}

impl ServerConfig {
    pub fn load() -> Self {
        dotenv().ok();

        Self {
            port: {
                std::env::var("WEB_PORT")
                    .expect("Please set env: WEB_PORT")
                    .parse::<u16>()
                    .expect("Invalid WEB_PORT")
            },
            database_url: {
                std::env::var("CLIENT_DB_URL").expect("Please set env: CLIENT_DB_URL")
            },
            salt: {
                std::env::var("ARGON_SALT")
                    .expect("Please set env: ARGON_SALT")
                    .into_bytes()
                    .into_boxed_slice()
            },
        }
    }

    pub fn socket_addr(&self) -> SocketAddrV4 {
        SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), self.port)
    }
}
