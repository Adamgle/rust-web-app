// TODO: Delegate the database schemas to separate module/file.
#[derive(Debug)]
pub struct DatabaseSession {
    pub id: sqlx::types::uuid::Uuid,
    pub user_id: i32,
    pub created_at: chrono::NaiveDate,
    pub expires_at: chrono::NaiveDate,
}

#[derive(Debug)]

pub struct DatabaseUser {
    // TODO: Map the full user schema here.
    pub id: i32,
    pub created_at: chrono::NaiveDate,
    pub account_id: i32,
    pub balance: f32,
    pub delta: f32,
    pub email: String,
    pub password_hash: String,
    pub password_salt: String,
}

// NOTE: That table is useless, we can just generate another row in the session with the same user_id.
// pub struct UserSessionsJunction {
//     user_id: i32,
//     session_id: sqlx::types::uuid::Uuid,
//     // Primary key is (user_id, session_id), not sure if we need to represent that here.
// }

// NOTE: Maybe that should be isolated into separate module as well.
// NOTE: I would be nice if there would be From conversion mapping database types -> client types,
// as doing it the opposite way does not apply, but not sure if we need
// ### Client-facing types

/// Stripped from sensitive info about the user
#[derive(serde::Serialize, Debug)]
pub struct ClientUser {
    pub id: i32,
    pub balance: f32,
    pub delta: f32,
}

impl From<DatabaseUser> for ClientUser {
    fn from(user: DatabaseUser) -> Self {
        Self {
            id: user.id,
            balance: user.balance,
            delta: user.delta,
        }
    }
}
