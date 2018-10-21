#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate reqwest;
extern crate rocket_contrib;

use rocket_contrib::Template;
use rocket::http::{ContentType, Status};
use rocket::response::{NamedFile, Response};

use std::io::Cursor;
use std::path::{PathBuf, Path};

#[get("/static/<path..>")]
pub fn static_server(path: PathBuf) -> Option<NamedFile> {
	NamedFile::open(Path::new("web/static/").join(path)).ok()
}

#[get("/harmonogram", format = "application/json")]
pub fn harmonogram<'a>() -> Response<'a> {
	if let Ok(res) = reqwest::get("http://gsx2json.com/api?id=12Q1jmsBpZh1LHSAcMwXIwWTZwKMzFoypw_fUrDbWJEQ").expect("failed to send request").text() {
		Response::build()
			.header(ContentType::JSON)
			.sized_body(Cursor::new(res))
			.finalize()
	} else {
		Response::build()
			.status(Status::InternalServerError)
			.sized_body(Cursor::new("Internal Server Error"))
			.finalize()
	}
}

#[get("/")]
pub fn index() -> Option<NamedFile> {
	NamedFile::open(Path::new("web/index.html")).ok()
}

fn main() {
    rocket::ignite().mount("/", routes![
    		static_server,
    		harmonogram,
    		index])
    	.attach(Template::fairing())
    	.launch();
}

