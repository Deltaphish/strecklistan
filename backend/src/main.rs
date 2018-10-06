#![feature(custom_derive)]
#![feature(plugin)]
#![feature(specialization)]
#![feature(extern_prelude)]
#![plugin(rocket_codegen)]

// Disable warnings caused by nightly rust phasing out this feature
#![allow(proc_macro_derive_resolution_fallback)]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
extern crate chrono;

mod schema;
mod database;
pub mod models;
pub mod util;
pub mod routes;

use rocket::Request;
use rocket::http::Status;
use self::util::ErrorJson;
use self::routes::{
    event,
    signup,
};


#[catch(404)]
pub fn not_found(_: &Request) -> ErrorJson { ErrorJson {
    status: Status::NotFound.into(),
    description: "Route Not Found".into(),
}}


fn main() {
    rocket::ignite()
        .catch(catchers![
            not_found,
        ])
        .mount("/", routes![
               event::get_events,
               event::get_event,
               event::post_event,
               signup::get_event_signups,
               signup::get_signup,
               signup::post_signup,
        ]).launch();
}
