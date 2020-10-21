#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::http::Method;
use rocket::response::status::NotFound;
use rocket_contrib::json::Json;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions, Error};

#[get("/get_teams")]
fn get_teams() -> Result<Json<Vec<wwc_core::Team>>, NotFound<String>> {
    let teams: Vec<wwc_core::Team> = wwc_db::get_teams().collect();
    Ok(Json(teams))
}

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:8080",
        "http://127.0.0.1:8888",
        "http://localhost:8888",
        "http://0.0.0.0:8888",
    ]);

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin",
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![get_teams])
        .attach(make_cors())
}

fn main() -> Result<(), Error> {
    rocket().launch();
    Ok(())
}
