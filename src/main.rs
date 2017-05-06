#![feature(plugin, custom_attribute, custom_derive)]
#![plugin(rocket_codegen)]
extern crate rocket;
extern crate rand;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
#[macro_use]
extern crate lazy_static;
extern crate ring;
extern crate jsonwebtoken as jwt;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate rocket_contrib;
extern crate time;

use diesel::pg::PgConnection;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

mod models;
mod services;
#[macro_use]
mod helpers;
mod controllers;
#[cfg(test)]
mod tests;

use controllers::auth;
use controllers::user;
use controllers::paste;

lazy_static! {
    pub static ref ENV: helpers::env::Env = helpers::env::load();
    pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = helpers::db::create_db_pool();
}
pub struct DBPool(Pool<ConnectionManager<PgConnection>>);

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/",
               routes![auth::login,
                       user::me,
                       user::get_users,
                       user::create_user,
                       user::get_user_by_id,
                       user::update_user_by_id,
                       user::delete_user_by_id,
                       paste::get_pastes,
                       paste::create_paste,
                       paste::get_paste_by_id,
                       paste::update_paste_by_id,
                       paste::delete_paste_by_id,
                       paste::get_pastes_by_user_id])
        .manage(DBPool(DB_POOL.clone()))
}

pub fn main() {
    rocket().launch();
}
