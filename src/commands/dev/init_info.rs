use crate::utils::{
    //globals::{USER_NAME, USER_SALT},
    log,
    prompt::{UserPrompter, },
};
pub const HELP_TEXT: &str = r#"
Usage: dev [OPTIONS_1] [OPTIONS_2] <PATH TO OBJECT> 

use dev mode wth the below options to change set levels and chest as locked or unlocked and manage the solutions and flags to them
Option_1:
    -l, --lock       Lock a object or reset its key(s):{solution, flag}
    -u, --unlock     Unlock an object

Option_2:
    -c, --chest      Lock/unlock a chest
    -l, --level      Lock/unlock a level

Examples:
"#;
pub static level_order:Vec<String>=vec![room_2,room_3,room_4,room_5,room_6,room_7,room_8,room_9,room_10];
pub fn dev_unlock_level(path_to_obj:&Path,path_to_next_level:Option<&Path>)->{
    //check if the object is locked at all
    //if chest then simpel -change lock info and remove compare_me and obj_salt (compare_me just in case)
    //if level then shift the 


}
pub fn dev_lock_level(path_to_obj:&Path,lock_perm:String,common_answer:String,path_to_next_level:&Path)->{
        
}
pub fn dev_unlock_chest(args: &[&str],
    current_dir: &Path,
    root_dir: &Path)->Result<(),String>{
        //check if path valid and get info path
        //check if is chest
        //check if is locked
        //set lock perm to "00"
        //delete decrypt_me


}
pub fn dev_lock_chest(args: &[&str],
    current_dir: &Path,
    root_dir: &Path,
    prompter_for_answer: &mut dyn UserPrompter,prompter_confirm:&mut dyn Prompter)->Result<(),String>{
        //check if path valid and get info path
        //check if chest
        //check if unlocked
        //if locked still ask to lock with new answer

    }