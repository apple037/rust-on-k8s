use axum::http::HeaderMap;
use axum::{extract::Query, http::StatusCode, response::IntoResponse, Json};
use jsonwebtoken::errors::Error;
use tracing::{debug,error};
use serde_json::{json, Value, to_string};

use crate::models::user_models::{CommonResponse, CreateUserRequest, User, UserLoginRequest, UserLoginResponse, UserResponse, UserUpdateRequest, UserDeleteRequest};
use crate::db_connection::{get_db_connection, execute_query_user_by_email, execute_insert_user, fetch_insert_id, execute_update_user, execute_delete_query, DbConnection};
use crate::config::{db_connection_string,redis_connection_string};
use crate::services::jwt_service::{issue_jwt_token, get_info_from_token};
use crate::redis_instance::RedisInstance;

pub async fn register(
    req: Json<CreateUserRequest>,   
) -> (StatusCode, Json<CommonResponse>) {
    debug!("Registering user: {:?}", req);
    // Check if the email already exists
    let connection_str: String = db_connection_string().await;
    let connection = get_db_connection(&connection_str).await.unwrap();
    let rows = execute_query_user_by_email(&connection, &req.email).await.unwrap();
    if rows.len() > 0 {
        let response = CommonResponse::error("email already exists".to_string(), serde_json::from_str("{}").unwrap());
        return (StatusCode::BAD_REQUEST, Json(response));
    }
    // Insert the user into the database
    // Build user struct according to the request
    let mut user = User::new(0, req.name.clone(), req.email.clone(), req.age, req.pwd.clone());
    let rows = execute_insert_user(&connection, &user).await.unwrap();
    let id = fetch_insert_id(&rows).await.unwrap();
    user._id = id;

    let response = CommonResponse::success("User created successfully".to_string(), user.to_json());
    (StatusCode::CREATED, Json(response))
}

pub async fn login(
    req: Json<UserLoginRequest>,
) -> (StatusCode, Json<CommonResponse>) {
        // Check if the username exists
        let connection_str = db_connection_string().await;
        let connection = get_db_connection(&connection_str).await.unwrap();
        let rows = execute_query_user_by_email(&connection, &req.email).await.unwrap();
        if rows.len() == 0 {
            let response = CommonResponse::error("email not found".to_string(), serde_json::from_str("{}").unwrap());
            return (StatusCode::BAD_REQUEST, Json(response));
        }
        if rows.len() > 1 {
            let response = CommonResponse::error("multiple users found".to_string(), serde_json::from_str("{}").unwrap());
            return (StatusCode::BAD_REQUEST, Json(response));
        }
        // Check if the password is correct
        let row = rows.get(0).unwrap();
        let user = User::new(
            row.get(4),
            row.get(0),
            row.get(1),
            row.get(2),
            row.get(3),
        );
        if user.pwd != req.pwd {
            let response = CommonResponse::error("incorrect password".to_string(), serde_json::from_str("{}").unwrap());
            return (StatusCode::BAD_REQUEST, Json(response));
        }
        let redis_connection_str = redis_connection_string().await;
        let mut redis = RedisInstance::new(&redis_connection_str);
        let is_exist = redis.exists(&user.email).unwrap();
        debug!("is_exist: {:?}", is_exist);
        if is_exist {
            let token = redis.get(&user.email).unwrap();
            let user_login_res = UserLoginResponse::new(user._id, user.name, user.email, user.age, token);
            let response = CommonResponse::success("User logged in successfully".to_string(), user_login_res.to_json());
            return (StatusCode::OK, Json(response));
        }

        // exchange the user for a token
        let token = issue_jwt_token(&user.email, &user.pwd);
        // write into redis

        redis.set_with_expiration(&user.email, &token, 3600).unwrap();
        // build the response
        let user_login_res = UserLoginResponse::new(user._id, user.name, user.email, user.age, token);
        let response = CommonResponse::success("User logged in successfully".to_string(), user_login_res.to_json());
        (StatusCode::OK, Json(response))
}

pub async fn user_info (headers: HeaderMap) -> (StatusCode, Json<CommonResponse>) {
    let token = headers.get("authorization").unwrap().to_str().unwrap();
    let token = token.replace("Bearer ", "");
    let token = token.trim();
    let token_data = get_info_from_token(&token);
    match token_data {
        Ok(token_data) => {
            let mail = token_data.email;
            let connection_str = db_connection_string().await;
            let connection = get_db_connection(&connection_str).await.unwrap();
            let rows = execute_query_user_by_email(&connection, &mail).await.unwrap();
            if rows.len() == 0 {
                let response = CommonResponse::error("email not found".to_string(), serde_json::from_str("").unwrap());
                return (StatusCode::BAD_REQUEST, Json(response));
            }
            if rows.len() > 1 {
                let response = CommonResponse::error("multiple users found".to_string(), serde_json::from_str("").unwrap());
                return (StatusCode::BAD_REQUEST, Json(response));
            }
            let row = rows.get(0).unwrap();
            let user_response = UserResponse::new(
                row.get(4),
                row.get(0),
                row.get(1),
                row.get(2),
                token_data.exp as i64,
                token_data.iat as i64,
                token_data.iss,
                token_data.typ,
            );

            let response = CommonResponse::success("User info retrieved successfully".to_string(), serde_json::json!(user_response));
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("[UserInfo]Error: {:?}", e);
            let response = CommonResponse::error("Invalid token".to_string(), serde_json::from_str("{}").unwrap());
            (StatusCode::UNAUTHORIZED, Json(response))
        }
    }
}

pub async fn logout(headers: HeaderMap) -> (StatusCode, Json<CommonResponse>) {
    let token = headers.get("authorization").unwrap().to_str().unwrap();
    let token = token.replace("Bearer ", "");
    let token = token.trim();
    let token_data = get_info_from_token(&token);
    match token_data {
        Ok(token_data) => {
            let mail = token_data.email;
            let redis_connection_str = redis_connection_string().await;
            let mut redis = RedisInstance::new(&redis_connection_str);
            let is_exist = redis.exists(&mail).unwrap();
            if is_exist {
                redis.del(&mail).unwrap();
                let response = CommonResponse::success("User logged out successfully".to_string(), serde_json::from_str("{}").unwrap());
                (StatusCode::OK, Json(response))
            } else {
                let response = CommonResponse::error("User not found".to_string(), serde_json::from_str("{}").unwrap());
                (StatusCode::BAD_REQUEST, Json(response))
            }
        }
        Err(e) => {
            let response = CommonResponse::error("Invalid token".to_string(), serde_json::from_str("{}").unwrap());
            (StatusCode::UNAUTHORIZED, Json(response))
        }
    }
}

pub async fn update_user_info(
    headers: HeaderMap,
    req: Json<UserUpdateRequest>,
) -> (StatusCode, Json<CommonResponse>) {
    let token = match headers.get("authorization").and_then(|v| v.to_str().ok()) {
        Some(token) => token.replace("Bearer ", "").trim().to_string(),
        None => {
            let response = CommonResponse::error("Authorization token missing or invalid".to_string(), json!({}));
            return (StatusCode::UNAUTHORIZED, Json(response));
        }
    };    
    let token = token.trim();
    let token_data = get_info_from_token(&token);
    match token_data {
        Ok(token_data) => {
            let mail = token_data.email;
            let connection_str = db_connection_string().await;
            let connection = get_db_connection(&connection_str).await.unwrap();
            let rows = execute_query_user_by_email(&connection, &mail).await.unwrap();
            if rows.len() == 0 {
                let response = CommonResponse::error("email not found".to_string(), serde_json::from_str("{}").unwrap());
                return (StatusCode::BAD_REQUEST, Json(response));
            }
            if rows.len() > 1 {
                let response = CommonResponse::error("multiple users found".to_string(), serde_json::from_str("{}").unwrap());
                return (StatusCode::BAD_REQUEST, Json(response));
            }
            let row = rows.get(0).unwrap();
            let mut user = User::new(
                row.get(4),
                row.get(0),
                row.get(1),
                row.get(2),
                row.get(3),
            );
            user.name = req.name.clone();
            user.age = req.age;
            let res = execute_update_user(&connection, &user).await.unwrap();
            if res == 1 {
                let response = CommonResponse::success("User info updated successfully".to_string(), user.to_json());
                (StatusCode::OK, Json(response))
            }
            else {
                let response = CommonResponse::error("User info update failed".to_string(), serde_json::from_str("{}").unwrap());
                (StatusCode::BAD_REQUEST, Json(response))
            }
        }
        Err(e) => {
            let response = CommonResponse::error("Invalid token".to_string(), serde_json::from_str("{}").unwrap());
            (StatusCode::UNAUTHORIZED, Json(response))
        }
    }
}

pub async fn delete_user(
    headers: HeaderMap,
    req: Json<UserDeleteRequest>,
) -> (StatusCode, Json<CommonResponse>) {
    let token = match headers.get("authorization").and_then(|v| v.to_str().ok()) {
        Some(token) => token.replace("Bearer ", "").trim().to_string(),
        None => {
            let response = CommonResponse::error("Authorization token missing or invalid".to_string(), json!({}));
            return (StatusCode::UNAUTHORIZED, Json(response));
        }
    };    
    let token = token.trim();
    let token_data = get_info_from_token(&token);
    match token_data {
        Ok(token_data) => {
            let mail = token_data.email;
            let connection_str = db_connection_string().await;
            let mut connection = get_db_connection(&connection_str).await.unwrap();
            let rows = execute_query_user_by_email(&connection, &mail).await.unwrap();
            if rows.len() == 0 {
                let response = CommonResponse::error("email not found".to_string(), serde_json::from_str("{}").unwrap());
                return (StatusCode::BAD_REQUEST, Json(response));
            }
            if rows.len() > 1 {
                let response = CommonResponse::error("multiple users found".to_string(), serde_json::from_str("{}").unwrap());
                return (StatusCode::BAD_REQUEST, Json(response));
            }
            let row = rows.get(0).unwrap();
            let user = User::new(
                row.get(4),
                row.get(0),
                row.get(1),
                row.get(2),
                row.get(3),
            );
            if user.pwd != req.pwd {
                let response = CommonResponse::error("incorrect password".to_string(), serde_json::from_str("{}").unwrap());
                return (StatusCode::BAD_REQUEST, Json(response));
            }
         
            let res = connection.execute_delete_query_with_rollback(&user._id).await;
            match &res {
                Ok(rows_affected) => {
                    if *rows_affected == 1 {
                        let response = CommonResponse::success("User deleted successfully".to_string(), serde_json::from_str("{}").unwrap());
                        (StatusCode::OK, Json(response))
                    } else {
                        let response = CommonResponse::error("User deletion failed".to_string(), serde_json::from_str("{}").unwrap());
                        (StatusCode::BAD_REQUEST, Json(response))
                    }
                }
                Err(e) => {
                    let response = CommonResponse::error("User deletion failed".to_string(), serde_json::from_str("{}").unwrap());
                    (StatusCode::BAD_REQUEST, Json(response))
                }
            }
        }
        Err(e) => {
            let response = CommonResponse::error("Invalid token".to_string(), serde_json::from_str("{}").unwrap());
            (StatusCode::UNAUTHORIZED, Json(response))
        }
    }
}