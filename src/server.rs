use rocket::{get, routes};
use rocket::fs::{FileServer, relative};
use std::path::PathBuf;
use deemak::commands;
use rocket::serde::json::Json;
use rocket::serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct CommandResponse {
    output: String,
    new_current_dir: Option<String>,
}

#[get("/run?<command>&<current_dir>")]
fn response(command: &str, current_dir: &str) -> Json<CommandResponse> {
    use commands::{cmd_manager, CommandResult};

    let parts: Vec<&str> = command.split_whitespace().collect();
    // let mut current_dir = PathBuf::from(current_dir);
    let root_dir = find_sekai_root().expect("Sekai root directory not found");
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

fn find_sekai_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;

    loop {
        let info_path = current.join("sekai/info.json");
        if info_path.exists() {
            return Some(current.join("sekai"));
        }

        if !current.pop() {
            break;
        }
    }

    None
}

#[rocket::main]
pub async fn launch_web() {
    let _ = rocket::build()
        .mount("/", FileServer::from(relative!("static")))
        .mount("/backend", routes![response]) 
        .launch()
        .await
        .expect("failed to launch Rocket server");
}
