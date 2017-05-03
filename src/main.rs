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

use std::io;
use std::path::Path;
use std::fs::File;

use rocket::Data;

use diesel::pg::PgConnection;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

mod models;
mod services;
mod helpers;
mod controllers;
#[cfg(test)]
mod tests;

mod paste_id;
use paste_id::PasteID;

use controllers::auth;
use controllers::user;
use controllers::paste;

lazy_static! {
    pub static ref ENV: helpers::env::Env = helpers::env::load();
    pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = helpers::db::create_db_pool();
}


#[get("/")]
fn index() -> &'static str {
    "
    USAGE

        POST /

            accepts raw data in the body of the request and responds with a URL of
            a page containing the body's content

        GET /<id>

            retrieves the content for the paste with id `<id>`
    "
}

#[post("/", data = "<paste>")]
fn upload(paste: Data) -> io::Result<String> {
    let id = PasteID::new(3);
    let filename = format!("upload/{id}", id = id);
    let url = format!("{host}/{id}\n", host = "http://localhost:8000", id = id);

    // Write the paste out to the file and return the URL.
    paste.stream_to_file(Path::new(&filename))?;
    Ok(url)
}

#[get("/<id>")]
fn retrieve(id: PasteID) -> Option<File> {
    let filename = format!("upload/{id}", id = id);
    File::open(&filename).ok()
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/",
                           routes![index,
                                   upload,
                                   retrieve,
                                   auth::login,
                                   user::me,
                                   user::get_users,
                                   user::create_user,
                                   user::get_user_by_id,
                                   user::update_user_by_id,
                                   user::delete_user_by_id,
                                   paste::get_pastes,
                                   paste::create_paste,
                                   paste::get_paste_by_id])
}

pub fn main() {
    rocket().launch();
}
