use redis::{Client, Commands, Connection, RedisResult};

pub struct RedisInstance {
    pub connection: Connection,
}

impl RedisInstance {
    pub fn new(connection_str: &str) -> RedisInstance {
        let connection = Client::open(connection_str)
            .expect("Unable to open redis connection")
            .get_connection()
            .expect("Unable to get redis connection");
        RedisInstance {
            connection,
        }
    }
    // Define some methods to interact with Redis
    pub fn set(&mut self, key: &str, value: &str) -> RedisResult<()> {
        self.connection.set(key, value)
    }

    pub fn get(&mut self, key: &str) -> RedisResult<String> {
        self.connection.get(key)
    }

    pub fn del(&mut self, key: &str) -> RedisResult<()> {
        self.connection.del(key)
    }

    pub fn exists(&mut self, key: &str) -> RedisResult<bool> {
        self.connection.exists(key)
    }

    // Set with expiration time
    pub fn set_with_expiration(&mut self, key: &str, value: &str, expiration: u64) -> RedisResult<()> {
        self.connection.set_ex(key, value, expiration)
    }

    // nx: only set the key if it does not already exist
    pub fn set_nx(&mut self, key: &str, value: &str) -> RedisResult<bool> {
        self.connection.set_nx(key, value)
    }
}
