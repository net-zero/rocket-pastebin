// This is required for NewPaste
use models::schema::pastes;

#[derive(Queryable)]
#[belongs_to(User)]
pub struct Paste {
    pub id: i32,
    pub user_id: i32,
    pub data: String,
}

#[derive(Insertable)]
#[table_name="pastes"]
pub struct NewPaste {
    pub user_id: i32,
    pub data: String,
}
