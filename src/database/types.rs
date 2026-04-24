#[derive(Debug)]
pub struct DatabaseSession {
    pub id: sqlx::types::uuid::Uuid,
    pub user_id: i32,
    pub created_at: chrono::NaiveDate,
    pub expires_at: chrono::NaiveDate,
}

#[derive(Debug)]

pub struct DatabaseUser {
    pub id: i32,
    pub created_at: chrono::NaiveDate,
    pub account_id: i32,
    pub balance: f32,
    pub change_percent: f32,
    pub change: f32,
    pub email: String,
    pub password_hash: String,
}

pub struct DatabaseAccount {
    pub id: i32,
    pub created_at: chrono::NaiveDate,
}

// NOTE: That table is useless, we can just generate another row in the session with the same user_id.
// pub struct UserSessionsJunction {
//     user_id: i32,
//     session_id: sqlx::types::uuid::Uuid,
//     // Primary key is (user_id, session_id), not sure if we need to represent that here.
// }

// NOTE: I would be nice if there would be From conversion mapping database types -> client types,
// as doing it the opposite way does not apply, but not sure if we need
// ### Client-facing types

/// Stripped from sensitive info about the user
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ClientUser {
    pub id: i32,
    pub balance: f32,
    pub change: f32,
    pub email: String,
    pub created_at: chrono::NaiveDate,
}

impl From<DatabaseUser> for ClientUser {
    fn from(user: DatabaseUser) -> Self {
        Self {
            id: user.id,
            balance: user.balance,
            change: user.change,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

// https://docs.rs/sqlx/latest/sqlx/postgres/types/index.html#types
#[derive(serde::Serialize)]

pub struct Stock {
    pub id: i32, // That should be unsigned, but it fails converting to u32, as postgres does not have unsigned, like a [1, 2^31 - 1]
    pub abbreviation: String,
    pub company: String,
    pub since: chrono::NaiveDate, // DATE
    pub price: f32,
    pub change_percent: f32,
    pub change: f32,
    // pub delta: f32,
    pub last_update: chrono::NaiveDate, // TIMESTAMP
    pub created_at: chrono::NaiveDate,  // TIMESTAMP
}
