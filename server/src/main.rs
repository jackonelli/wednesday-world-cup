#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::http::Method; // 1.
use rocket::response::status::NotFound;
use rocket_contrib::json::Json;
use rocket_cors::{
    AllowedHeaders,
    AllowedOrigins,
    Cors,
    CorsOptions, // 3.
    Error,       // 2.
};
use wwc_data::lsv;

#[get("/<file_name>")]
fn index(file_name: String) -> Result<Json<lsv::Data>, NotFound<String>> {
    let data_json =
        match wwc_data::file_io::read_json_file_to_str(&format!("server/data/{}", file_name)) {
            Ok(data) => data,
            Err(err) => return Err(NotFound(err.to_string())),
        };
    let data: lsv::Data = serde_json::from_str(&data_json).expect("JSON format error.");
    println!("{:?}", &data);
    Ok(Json(data))
}

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        // 4.
        "http://localhost:8080",
        "http://127.0.0.1:8080",
        "http://localhost:8000",
        "http://0.0.0.0:8000",
    ]);

    CorsOptions {
        // 5.
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(), // 1.
        allowed_headers: AllowedHeaders::some(&[
            "Authorization",
            "Accept",
            "Access-Control-Allow-Origin", // 6.
        ]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("error while building CORS")
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![index])
        .attach(make_cors()) // 7.
}

fn main() -> Result<(), Error> {
    // 2.
    rocket().launch();
    Ok(())
}
