// This is required for NewPaste
use models::schema::pastes;
use models::user::User;

#[derive(Queryable, Associations, Identifiable, Serialize, Deserialize, PartialEq, Debug, FromForm)]
#[belongs_to(User)]
pub struct Paste {
    pub id: i32,
    pub user_id: i32,
    pub data: String,
}

#[derive(Insertable, FromForm)]
#[table_name="pastes"]
pub struct NewPaste {
    pub user_id: i32,
    pub data: String,
}
