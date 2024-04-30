use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub _id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub pwd: String,
}

impl User {
    pub fn new(_id: i32, name: String, email: String, age: i32, pwd: String) -> User {
        User {
            _id,
            name,
            email,
            age,
            pwd,
        }
    }
    pub fn to_string(&self) -> String {
        format!("User: id: {}, name: {}, email: {}, age: {}", self._id, self.name, self.email, self.age)
    }
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!(self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub typ: String,
}

impl UserResponse {
    pub fn new(id: i32, name: String, email: String, age: i32, exp: i64, iat: i64, iss: String, typ: String) -> UserResponse {
        UserResponse {
            id,
            name,
            email,
            age,
            exp,
            iat,
            iss,
            typ,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub age: i32,
    pub pwd: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserLoginRequest {
    pub email: String,
    pub pwd: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserLoginResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub token: String,
}

impl UserLoginResponse {
    pub fn new(id: i32, name: String, email: String, age: i32, token: String) -> UserLoginResponse {
        UserLoginResponse {
            id,
            name,
            email,
            age,
            token,
        }
    }
    pub fn to_string(&self) -> String {
        format!("User: id: {}, name: {}, email: {}, age: {}, token: {}", self.id, self.name, self.email, self.age, self.token)
    }
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!(self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
    pub iss: String,
    pub typ: String,
    pub email: String,
    pub name: String,
    pub age: i32,
    pub id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserUpdateRequest {
    pub name: String,
    pub age: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserDeleteRequest {
    pub pwd: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub data: Value,
}

impl Data {
    pub fn new(data: Value) -> Data {
        Data {
            data,
        }
    }
    
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommonResponse {
    pub message: String,
    pub code: String,
    #[serde(flatten)] // 使用 flatten 属性将 data 字段扁平化
    pub data: Data,
}

impl CommonResponse {
    pub fn new(message: String, code: String, data: Value) -> CommonResponse {
        let data = Data::new(data);
        CommonResponse {
            message,
            code,
            data,
        }
    }

    pub fn success(message: String, data: Value) -> CommonResponse {
        let data = Data::new(data);
        CommonResponse {
            message,
            code: "200".to_string(),
            data: data,
        }
    }

    pub fn error(message: String, data: Value) -> CommonResponse {
        let data = Data::new(data);
        CommonResponse {
            message,
            code: "500".to_string(),
            data,
        }
    }
}