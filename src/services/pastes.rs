use diesel;
use diesel::result;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use models::schema;
use models::pastes::*;

use self::schema::pastes;

pub fn create_paste<'a>(paste: &'a NewPaste,
                        conn: &'a PgConnection)
                        -> Result<Paste, result::Error> {
    diesel::insert(paste)
        .into(pastes::table)
        .get_result(conn)
}

pub fn update_paste(paste: Paste, conn: &PgConnection) -> Result<Paste, result::Error> {
    diesel::update(pastes::table.find(paste.id))
        .set((pastes::data.eq(paste.data)))
        .get_result(conn)
}

pub fn get_paste_by_id(id: i32, conn: &PgConnection) -> Result<Paste, result::Error> {
    pastes::table.find(id).get_result::<Paste>(conn)
}

pub fn get_pastes(conn: &PgConnection) -> Result<Vec<Paste>, result::Error> {
    pastes::table.limit(20).load::<Paste>(conn)
}

pub fn get_pastes_by_user_id(user_id: i32,
                             conn: &PgConnection)
                             -> Result<Vec<Paste>, result::Error> {
    pastes::table
        .filter(pastes::user_id.eq(user_id))
        .limit(20)
        .load::<Paste>(conn)
}

pub fn delete_paste(id: i32, conn: &PgConnection) -> Result<usize, result::Error> {
    diesel::delete(pastes::table.filter(pastes::id.eq(id))).execute(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::pg::PgConnection;
    use r2d2::Pool;
    use r2d2_diesel::ConnectionManager;

    use DB_POOL;
    use tests::helpers::testdata;


    #[test]
    fn test_create_paste() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        let paste_data = "test paste data";

        let user_id = testdata::recreate().user.id;

        let new_paste = NewPaste {
            user_id: user_id,
            data: paste_data.to_string(),
        };
        let paste = create_paste(&new_paste, conn).unwrap();

        assert_eq!(new_paste.user_id, user_id);
        assert_eq!(new_paste.data, paste_data);
    }

    #[test]
    fn test_update_paste() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        let updated_data = "updated paste data";

        let mut paste = testdata::recreate().paste;
        let updated_paste = Paste {
            id: paste.id,
            user_id: paste.user_id,
            data: updated_data.to_string(),
        };
        paste = update_paste(updated_paste, conn).unwrap();
        assert_eq!(paste.data, updated_data);
    }

    #[test]
    fn test_get_paste_by_id() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();

        let test_paste = testdata::recreate().paste;
        let fetched_paste = get_paste_by_id(test_paste.id, conn).unwrap();
        assert_eq!(fetched_paste.id, test_paste.id);
        assert_eq!(fetched_paste.user_id, test_paste.user_id);
        assert_eq!(fetched_paste.data, test_paste.data);
    }

    #[test]
    fn test_delete_paste() {
        let conn: &PgConnection = &DB_POOL.get().unwrap();
        let paste_data = "test paste data";

        let user_id = testdata::recreate().user.id;
        let paste = NewPaste {
            user_id,
            data: paste_data.to_string(),
        };
        let paste_id = create_paste(&paste, conn).unwrap().id;
        assert_eq!(delete_paste(paste_id, conn), Ok(1));
    }
}
