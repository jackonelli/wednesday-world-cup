#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::http::Method;
use rocket::response::status::NotFound;
use rocket_contrib::json::Json;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions, Error};
use wwc_data::lsv;

/// LSV data
///
/// Deprecated, this will not be handled by the server.
/// It will be handed by a database connection, but it is nice for mock data.
#[get("/<file_name>")]
fn get_lsv_json(file_name: String) -> Result<Json<lsv::Data>, NotFound<String>> {
    let data_json =
        match wwc_data::file_io::read_json_file_to_str(&format!("server/data/{}", file_name)) {
            Ok(data) => data,
            Err(err) => return Err(NotFound(err.to_string())),
        };
    let data: lsv::Data = serde_json::from_str(&data_json).expect("JSON format error.");
    Ok(Json(data))
}

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:8080",
        "http://127.0.0.1:8080",
        "http://localhost:8000",
        "http://0.0.0.0:8000",
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
        .mount("/", routes![get_lsv_json])
        .attach(make_cors())
}

fn main() -> Result<(), Error> {
    rocket().launch();
    Ok(())
}
