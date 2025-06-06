use deemak::commands;
use deemak::utils::find_root;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::fs::{FileServer, relative};
use rocket::http::Header;
use rocket::serde::Serialize;
use rocket::serde::json::Json;
use rocket::{Request, Response, get, options, routes};
use std::path::PathBuf;
use crate::globals::WORLD_DIR;


#[derive(Serialize)]
struct CommandResponse {
    output: String,
    new_current_dir: Option<String>,
}

// === Main GET endpoint ===
#[get("/run?<command>&<current_dir>")]
fn response(command: &str, current_dir: &str ) -> Json<CommandResponse> {
    use commands::{CommandResult, cmd_manager};

    let world_dir = WORLD_DIR.get().expect("WORLD_DIR not initialized");
    let parts: Vec<&str> = command.split_whitespace().collect();
    let root_dir = find_root::find_home(&world_dir).expect("Could not find sekai home directory");
    let mut current_dir = if current_dir.is_empty() {
        root_dir.clone()
    } else {
        PathBuf::from(current_dir)
    };

    match cmd_manager(&parts, &mut current_dir, &root_dir) {
        CommandResult::Output(output) => Json(CommandResponse {
            output,
            new_current_dir: None,
        }),
        CommandResult::ChangeDirectory(new_dir, message) => Json(CommandResponse {
            output: message,
            new_current_dir: Some(new_dir.display().to_string()),
        }),
        CommandResult::Clear => Json(CommandResponse {
            output: "__CLEAR__".to_string(),
            new_current_dir: None,
        }),
        CommandResult::Exit => Json(CommandResponse {
            output: "__EXIT__".to_string(),
            new_current_dir: None,
        }),
        CommandResult::NotFound => Json(CommandResponse {
            output: "Command not found. Try `help`.".to_string(),
            new_current_dir: None,
        }),
    }
}

// === CORS preflight handler ===
#[options("/<_..>")]
fn cors_preflight() -> &'static str {
    ""
}

// === Add CORS headers ===
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        res.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, OPTIONS",
        ));
        res.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        res.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

// // === Root directory finder ===
// fn set_world_dir(world_dir: PathBuf) -> Option<PathBuf> {
//     WORLD_DIR.set(world_dir).expect("WORLD_DIR already set");
// }

#[rocket::main]
pub async fn server() {
        let _ = rocket::build()
        .attach(CORS)
        .mount("/", FileServer::from(relative!("static")))
        .mount("/backend", routes![response, cors_preflight])
        .launch()
        .await
        .expect("failed to launch Rocket server");
}