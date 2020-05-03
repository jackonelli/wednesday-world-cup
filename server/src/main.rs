#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::response::status::NotFound;
use rocket_contrib::json::Json;
use wwc_data::lsv;

#[get("/<file_name>")]
fn index(file_name: String) -> Result<Json<lsv::Data>, NotFound<String>> {
    let data_json =
        match wwc_data::file_io::read_json_file_to_str(&format!("server/data/{}", file_name)) {
            Ok(data) => data,
            Err(err) => return Err(NotFound(err.to_string())),
        };
    let data: lsv::Data = serde_json::from_str(&data_json).expect("JSON format error.");
    Ok(Json(data))
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
