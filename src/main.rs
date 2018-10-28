//! Server pro symposion

#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate ring;
extern crate rand;
extern crate rocket;
extern crate reqwest;
extern crate rpassword;
extern crate rocket_contrib;
#[macro_use] extern crate lazy_static;

extern crate serde;
#[macro_use] extern crate serde_derive;

use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

use rocket_contrib::{Template, Json};
use rocket::http::{ContentType, Status};
use rocket::response::{NamedFile, Response};

use std::iter;
use std::sync::RwLock;
use std::fs::{File, rename};
use std::io::{Cursor, Write};
use std::path::{PathBuf, Path};
use std::collections::HashMap;

pub mod util;

lazy_static! {
	/// SHA-1 hash hesla pro administraci
	pub static ref HASH: RwLock<String> = RwLock::new(String::new());
	/// Momentálně validní autentifikační tokeny
	pub static ref TOKENS: RwLock<Vec<String>> = RwLock::new(Vec::new());
}


/// Staticky podává soubory ve složce web/static/
#[get("/static/<path..>")]
pub fn static_server(path: PathBuf) -> Option<NamedFile> {
	NamedFile::open(Path::new("web/static/").join(path)).ok()
}

/// Přihlášení, vrací 200 + token pokud se autentifikace podařila,
/// 403 + chybu, pokud ne
#[post("/login", format = "application/json", data = "<input>")]
pub fn login<'a>(input: Json<String>) -> Response<'a> {
	let hash = HASH.read().unwrap();
	let new_hash = util::make_hash(input.into_inner().as_str());

	if *hash == new_hash {
		let mut rng = thread_rng();
		let chars: String = iter::repeat(())
			.map(|()| rng.sample(Alphanumeric))
			.take(16)
			.collect();
		println!("   => new token: {}", chars);

		let mut tokens = TOKENS.write().unwrap();
		tokens.push(chars.clone());

		Response::build()
			.status(Status::Ok)
			.sized_body(Cursor::new(format!("\"{}\"", chars)))
			.header(ContentType::JSON)
			.finalize()
	} else {
		Response::build()
			.status(Status::Forbidden)
			.sized_body(Cursor::new("incorrect password"))
			.finalize()
	}
}

/// Vymaže token z pole
#[post("/logout", format = "application/json", data = "<input>")]
pub fn logout<'a>(input: Json<String>) {
	let mut tokens = TOKENS.write().unwrap();
	tokens.retain(|x| *x != *input);
}

/// Posílá informace z harmonogramové tabulky
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

/// Vrací daný obsah
#[get("/<name>", rank = 2)]
pub fn get_content(name: String) -> Option<NamedFile> {
	NamedFile::open(Path::new("web/").join(Path::new(&name))).ok()
}

/// Přidá obsah nebo vytvoří novou verzi, pokud obsah již existoval,
/// je nová verze uložena do souboru `web/<name>.new` a musí být následně
/// potvrzena
#[put("/<name>", format = "application/json", data = "<input>")]
pub fn add_content<'a>(name: String, input: Json<util::NewContent>) -> Response<'a> {
	let tokens = TOKENS.read().unwrap();

	if !tokens.contains(&input.token) {
		return Response::build()
			.status(Status::Forbidden)
			.sized_body(Cursor::new("unknown auth token"))
			.finalize();
	}

	drop(tokens);

	if Path::new(&format!("web/{}", name)).exists() {
		match File::create(format!("web/{}.new", name)) {
			Ok(mut f) => write!(f, "{}", input.content)
				.map(|_| Response::build().status(Status::Ok).finalize())
				.map_err(|e| Response::build().status(Status::InternalServerError).sized_body(Cursor::new(e.to_string())).finalize())
				.unwrap_or_else(|e| e),
			Err(_) => Response::build()
				.status(Status::InternalServerError)
				.sized_body(Cursor::new("can't open file for writing"))
				.finalize()
		}
	} else {
		match File::create(format!("web/{}", name)) {
			Ok(mut f) => write!(f, "{}", input.content)
				.map(|_| Response::build().status(Status::Ok).finalize())
				.map_err(|e| Response::build().status(Status::InternalServerError).sized_body(Cursor::new(e.to_string())).finalize())
				.unwrap_or_else(|e| e),
			Err(_) => Response::build()
				.status(Status::InternalServerError)
				.sized_body(Cursor::new("can't open file for writing"))
				.finalize()
		}
	}
}

/// Potvrdí daný obsah, tzn. přesune `web/soubor.new` -> `web/soubor`
#[post("/<name>", format = "application/json", data = "<token>", rank = 2)]
pub fn approve<'a>(name: String, token: Json<String>) -> Response<'a> {
	let tokens = TOKENS.read().unwrap();

	if !tokens.contains(&token) {
		return Response::build()
			.status(Status::Forbidden)
			.sized_body(Cursor::new("unknown auth token"))
			.finalize();
	}
	drop(tokens);

	if Path::new(&format!("web/{}.new", name)).exists() {
		match rename(format!("web/{}.new", name), format!("web/{}", name)) {
			Ok(_) => Response::build()
				.status(Status::Ok)
				.finalize(),
			Err(_) => Response::build()
				.status(Status::InternalServerError)
				.sized_body(Cursor::new("couldn't move file"))
				.finalize(),
		}
	} else {
		Response::build()
			.status(Status::NoContent)
			.sized_body(Cursor::new("nothing to approve"))
			.finalize()
	}
}

/* |
 * |
 * |  Takhle se přidávaj templaty, viz Tera templating engine
 * |
 * |
 * v
 */

/// Posílá template indexu
#[get("/")]
pub fn index() -> Template {
	let context: HashMap<String, String> = HashMap::new();
	Template::render("index", &context)
}


fn main() {
	let mut hash = HASH.write().unwrap();
	*hash = rpassword::prompt_password_stdout("Hash pro autentifikaci: ").unwrap();
	drop(hash);

	rocket::ignite().mount("/", routes![
			static_server,
			harmonogram,
			get_content,
			add_content,
			approve,
			logout,
			login,
			index])
		.attach(Template::fairing())
		.launch();
}

