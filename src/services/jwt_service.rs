use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, get_current_timestamp};
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::Algorithm;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

#[derive(Deserialize)]
pub struct Config {
    pub jwt: JWTConfig,
}

#[derive(Deserialize)]
pub struct JWTConfig {
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
    pub iss: String,
    pub typ: String,
    pub email: String
}

pub fn load_config() -> JWTConfig {
    let config_value: String = std::fs::read_to_string("config.toml")
        .expect("Unable to read config file");
    let config: Config = toml::from_str(&config_value).expect("Unable to parse config file");
    let jwt_config = config.jwt;
    jwt_config
}

pub fn issue_jwt_token(email: &str, password: &str) -> String {
    // TODO: check email and password to determine subject
    let typ;
    if email.ends_with("@colond.com") {
        typ = ":D";
    } else {
        typ = "guest";
    }
    // TODO Validate email and password
    debug!("[IssueToken] A user login: {} with password: {}", email, password);
    let jwt_config = load_config();
    let secret = jwt_config.secret;
    // set iat to current timestamp and exp to 30 seconds later
    let iat = get_current_timestamp();
    let exp = iat + 3600;
    let claims = Claims {
        sub: "Colon D Face :)".to_string(), // Subject: to what the token refers to
        iat: iat,
        exp: exp,
        email: email.to_owned(),
        iss: "ColonD".to_owned(), // Issuer
        typ: typ.to_owned(), // Type
    };
    debug!("[IssueToken]Claims: {:?}", claims);
    // Custom header
    let mut header = Header::new(Algorithm::HS256);
    header.typ = Some("JWT".to_owned());
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap();
    token
}

pub fn get_info_from_token(_token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    // parse email from token
    let jwt_config = load_config();
    let secret = jwt_config.secret;
    // Use Validation to validate claims
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&["ColonD"]);
    let token_data = match decode::<Claims>(
        _token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    ) {
        Ok(c) => c.claims,
        Err(err) => match *err.kind() {
            // Return error message not panic
            ErrorKind::InvalidToken => {
                error!("[GetInfoFromToken]Invalid token");
                return Err(err);
            }
            ErrorKind::InvalidIssuer => {
                error!("[GetInfoFromToken]Invalid issuer");
                return Err(err);
            }
            ErrorKind::InvalidSubject => {
                error!("[GetInfoFromToken]Invalid subject");
                return Err(err);
            }
            ErrorKind::ExpiredSignature => {
                error!("[GetInfoFromToken]Expired signature");
                return Err(err);
            }
            ErrorKind::InvalidAudience => {
                error!("[GetInfoFromToken]Invalid audience");
                return Err(err);
            }
            _ => {
                error!("[GetInfoFromToken]Unknown error: {}", err);
                return Err(err);
            }
        },
    };

    Ok(token_data)
}
pub fn revoke_token(_token: &str) -> bool {
    // TODO
    true
}
