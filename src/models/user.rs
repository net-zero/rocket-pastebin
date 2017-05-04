// This is required for NewUser
use models::schema::users;

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
