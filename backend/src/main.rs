#![feature(specialization)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate diesel;

mod database;
pub mod models;
pub mod routes;
mod schema;
pub mod util;

use crate::database::create_pool;
use crate::models::user::JWTConfig;
use crate::routes::{graphql, session};
use crate::util::catchers::catchers;
use chrono::Duration;
use diesel_migrations::{run_pending_migrations, setup_database};
use dotenv::dotenv;
use frank_jwt::Algorithm;
use rocket::routes;
use std::env;

fn main() {
    dotenv().ok();

    let db_pool = create_pool().expect("Could not create database pool");

    let jwt_config = JWTConfig {
        algorithm: Algorithm::HS512,
        secret: env::var("JWT_SECRET").expect("JWT_SECRET not set"),
        token_lifetime: Duration::weeks(2),
    };

    let run_migrations = env::var("RUN_MIGRATIONS")
        .map(|s| s.parse().unwrap_or(false))
        .unwrap_or(false);
    if run_migrations {
        let connection = db_pool.get().expect("Could not connect to database");

        setup_database(&connection).expect("Could not set up database");

        run_pending_migrations(&connection).expect("Could not run database migrations");
    }

    let mut rocket = rocket::ignite()
        .manage(db_pool)
        .manage(graphql::create_schema())
        .manage(jwt_config)
        .register(catchers())
        .mount(
            "/",
            routes![
                session::user_info,
                session::no_user,
                session::login,
                graphql::graphiql,
                graphql::post_graphql_handler_auth,
                graphql::post_graphql_handler,
            ],
        );
    let config = rocket.config();

    // Mount dev-only routes
    if config.environment.is_dev() {
        rocket = rocket.mount("/", routes![session::register]);
    }

    rocket.launch();
}
