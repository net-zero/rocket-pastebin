// This is required for NewUser
use super::schema::users;

pub struct RawUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}

pub struct RawUpdatedUser<'a> {
    pub username: Option<&'a str>,
    pub email: Option<&'a str>,
    pub password: Option<&'a str>,
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: &'a str,
    pub password_digest: Vec<u8>,
}

#[derive(Queryable)]
#[has_many(pastes)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_digest: Vec<u8>,
    pub admin: bool,
}
