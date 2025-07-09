use deemak::commands::cmds;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

// === External Crates ===
use rocket::{get, post, options, routes, Request, Response, Config, State};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::fs::{FileServer, relative};
use rocket::http::Header;
use rocket::serde::{Deserialize, Serialize, json::Json};
use dotenvy::dotenv;

// === Local Modules ===
use crate::auth;
use deemak::commands::{self, CommandResult, cmd_manager};
use deemak::utils::find_root;
use deemak::utils::prompt::DummyPrompter;
use crate::globals::WORLD_DIR;

// === Data Structures ===
#[derive(Serialize)]
struct CommandResponse {
    output: String,
    new_current_dir: Option<String>,
}

// GET endpoint for command execution
#[get("/run?<command>&<current_dir>")]
fn response(command: &str, current_dir: &str) -> Json<CommandResponse> {
    let world_dir = WORLD_DIR.get().expect("WORLD_DIR not initialized");
    let root_dir = find_root::find_home(&world_dir).expect("Could not find sekai home directory");

    let mut current_dir = if current_dir.is_empty() {
        root_dir.clone()
    } else {
        PathBuf::from(current_dir)
    };

    let mut prompter = DummyPrompter;
    match cmd_manager(&command.split_whitespace().collect::<Vec<_>>(), &mut current_dir, &root_dir, &mut prompter) {
        CommandResult::Output(output) => Json(CommandResponse { output, new_current_dir: None }),
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

// CORS Preflight
#[options("/<_..>")]
fn cors_preflight() -> &'static str {
    ""
}
// === CORS Fairing ===
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
        res.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, OPTIONS"));
        res.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        res.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

fn generate_config_js(port: u16) {
    let js_content = format!(
        r#"export const BACKEND_URL = "http://localhost:{}";"#,
        port
    );

    let path = "static/config.js";
    let mut file = File::create(path).expect("Failed to create config.js");
    file.write_all(js_content.as_bytes())
        .expect("Failed to write config.js");

    println!("Generated static/config.js with port {}", port);
}

// === Rocket Entry Point ===
#[rocket::main]
pub async fn server() -> Result<(), rocket::Error> {
    dotenv().ok();

    let port: u16 = env::var("BACKEND_PORT")
        .unwrap_or_else(|_| "8001".to_string())
        .parse()
        .expect("Invalid port number");

    generate_config_js(port);

    let config = Config {
        port,
        ..Config::default()
    };

    let _rocket = rocket::custom(config)
        .attach(CORS)
        .mount("/", FileServer::from(relative!("static")))
        .mount("/backend", routes![response, cors_preflight,auth::register,auth::login])
        .launch()
        .await?;
    Ok(())
}