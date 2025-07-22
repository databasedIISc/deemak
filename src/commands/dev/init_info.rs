
use super::super::argparser::ArgParser;
use super::super::cmds::normalize_path;
use crate::metainfo::info_reader::{read_get_obj_info, update_obj_status, read_about, read_location,write_about};
use crate::metainfo::lock_perm::operation_locked_perm;
use crate::metainfo::read_lock_perm;
use crate::rns::security::{argonhash, characterise_enc_key, decrypt, encrypt};
use crate::utils::{
    //globals::{USER_NAME, USER_SALT},
    log,
    prompt::UserPrompter,
};
use argon2::password_hash::SaltString;
use rocket::http::tls::rustls::internal::msgs::message;
use serde_json::Value;
use std::path::Path;
use std::result;
pub const HELP_TEXT: &str = r#"
Usage: dev info [OPTIONS_1] <PROPERTY_NAME> <PROPERTY_VALUE(if writing)> 

Option_1:
    -w, --write      write about an object
    -r, --read       read about an object

<property-name>:
    -a, --about      about of the object
    -l, --location   location of the object(does not allow writing)

Examples:
"#;
pub fn dev_info(
    args: &[&str],
    current_dir: &Path,
    root_dir: &Path,
) -> Result<String, String> {
    // Check if the path is valid and get info path
    let mut write_mode = false;
    let mut read_mode = false;
    
    let mut property_value = None;

    let path = current_dir;
    let info_path = path.join("/.dir_info/info.json");//where the locationa and about are stored
    
    //store positional args in order and create one string out of them
    let mut pos_args_str = String::new();
    let mut parser = ArgParser::new(&["-w", "--write", "-r", "--read", "-a", "--about", "-l", "--location"]);
    let pos_args = parser.get_positional_args();
    
    match args[0]{
        "-w" | "--write" => {
            write_mode = true;
        }
        "-r" | "--read" => {
            read_mode = true;
        }
        _ => {
            return Err("Invalid option. Use -w/--write or -r/--read.".to_string());
        }
    }
    if read_mode{
        match args[1] {
            "-a" | "--about" => {
                return read_about(&info_path);
            }
            "-l" | "--location" => {
                return read_location(&info_path);
            }
            _ => {
                return Err("Invalid field. Use -a/--about or -l/--location.".to_string());
            }
        }
    }
    if write_mode{
        match args[1]{
            "-a" | "--about" => {
                if args.len() < 3 {
                    return Err("No value provided for about.".to_string());
                }
                //join args[2..] to a string
                property_value = Some(args[2..].join(" "));
                return write_about(&info_path, property_value.unwrap());
            }
            "-l" | "--location" => {
                return Err("Cannot write to location field.".to_string());
            }
            _ => {
                return Err("Invalid field. Use -a/--about or -l/--location.".to_string());
            }
        }
    }
    else{
        return Err("Invalid mode. Use -w/--write or -r/--read.".to_string());
    }
}



