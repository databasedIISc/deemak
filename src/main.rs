#![allow(unused_variables, unused_mut, dead_code)]
use crate::gui_main::{run_gui_loop, sekai_initialize};
use crate::gui_shell::DEEMAK_BANNER;
use crate::rns::create_dmk_sekai::{self, original_from_encrypted_sekai};
use crate::utils::file_mgr::DeemakSekaiMgr;
use crate::utils::{cleanup::exit_deemak, debug_mode, log};
use clap::{Parser, Subcommand};
use deemak::utils::file_mgr::SekaiOperation;
use deemak::*;
use raylib::ffi::{SetConfigFlags, SetTargetFPS};
use raylib::prelude::get_monitor_width;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Deemak - A Text Adventure Game Engine for your Sekai (World), developed by Databased Club, IISc Bangalore.
struct DeemakArgs {
    /// Path to the Deemak Encrypted Sekai (World) file or directory
    #[arg(required = true, value_name = "SEKAI_PATH")]
    sekai_directory: PathBuf,

    /// Run the application in GUI mode (default)
    #[arg(long, default_value_t = true)]
    gui: bool,

    /// Run the application in web mode (requires a web server)
    #[arg(long, default_value_t = false)]
    web: bool,

    /// Development subcommands
    #[command(subcommand)]
    command: Option<DeemakCommands>,
}

#[derive(Subcommand, Debug)]
enum DeemakCommands {
    /// Sekai commands for creating and restoring Deemak Encrypted Sekai files.
    Dev {
        #[command(subcommand)]
        subcommand: DeemakDev,
    },

    /// Authorization of user login and registration
    Auth,
}

#[derive(Subcommand, Debug)]
enum DeemakDev {
    /// Developer can play the Sekai in Developer Mode
    Play,

    /// Create new Deemak Encrypted file from your existing Sekai directory
    Create {
        /// Password for encryption
        #[arg(short, long)]
        password: Option<String>,

        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Force overwrite existing Deemak file(if it exists)
        #[arg(short, long, default_value_t = false)]
        force: bool,
    },

    /// Restore Sekai from a Deemak Encrypted file
    Restore {
        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,

        /// Password for encryption
        #[arg(short, long)]
        password: Option<String>,
    },
}

fn main() {
    println!("{DEEMAK_BANNER}");
    log::log_info("Application", "Starting DEEMAK Shell");

    let args = DeemakArgs::parse();
    // Check existance.
    if !args.sekai_directory.exists() {
        fatal_error!(
            "SEKAI",
            "Sekai path provided does not exist: {}. Please provide a valid Sekai file or directory.",
            args.sekai_directory.clone().display()
        );
    }

    // This object will manage the whole Sekai operations for us.
    let mut sekai_obj = DeemakSekaiMgr::new(args.sekai_directory, None);

    // If its a directory, validated sekai with dir_info's will get created, ELSE
    // if its a file, we need a valid deemak file.
    if !sekai_obj.valid_sekai {
        fatal_error!(
            "SEKAI",
            "Invalid Sekai path provided: {}. Please provide a valid Sekai file or directory.",
            sekai_obj.abs_path.display()
        );
    }
    // NOTE: Here on out, SEKAI exist and is VALID! No need to check again!

    // get absolute path to the sekai path
    let sekai_path = sekai_obj.abs_path.clone();
    log::log_info("SEKAI", &format!("Sekai path provided: {sekai_path:?}"));

    let possible_sekai_opers = sekai_obj.oper_allowed();
    if SekaiOperation::Invalid.is_present(possible_sekai_opers.clone()) {
        fatal_error!(
            "SEKAI",
            "Invalid Sekai operation detected. Sekai Criterion Unmet. Please check the Sekai file or directory."
        );
    }

    if let Some(_cmd) = args.command {
        match _cmd {
            DeemakCommands::Dev { subcommand } => {
                DEV_MODE.set(true).expect("DEV_MODE already set");
                match subcommand {
                    DeemakDev::Create {
                        password,
                        output,
                        force,
                    } => {
                        // Make sure Create Criteria is met, as mentioned in file_mgr
                        if !SekaiOperation::Create.is_present(possible_sekai_opers) {
                            SekaiOperation::Create.log_err();
                            exit_deemak(1);
                        }

                        log::log_info("DEEMAK", "Creating Deemak Encrypted Sekai file");
                        // Get input password securely
                        sekai_obj.set_password(true).unwrap_or_else(|e| {
                            fatal_error!(
                                "SEKAI",
                                "Failed to set password for Sekai: {}",
                                e.to_string()
                            );
                        });

                        let mut output_path = if let Some(out) = output {
                            out
                        } else {
                            let filename = sekai_obj
                                .abs_path
                                .file_name()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or("deemak_sekai");
                            PathBuf::from(filename)
                        };
                        let output_obj =
                            DeemakSekaiMgr::new(output_path.clone(), sekai_obj.password.clone());
                        output_path = output_obj.abs_path.clone(); // Set the output path to the absolute path

                        // Handle encryption
                        match create_dmk_sekai::deemak_encrypt_sekai(
                            &sekai_obj.abs_path,
                            &output_path,
                            output_obj.password.clone().unwrap().as_str(),
                            force,
                        ) {
                            Ok(_) => {
                                let absolute_created_path = std::fs::canonicalize(&output_path)
                                    .unwrap_or_else(|_| output_path.clone());
                                pr_info!(
                                    "SEKAI",
                                    "Successfully created Deemak file at: {}",
                                    absolute_created_path.display()
                                );
                            }
                            Err(e) => {
                                fatal_error!(
                                    "SEKAI",
                                    "Failed to create Deemak file: {}",
                                    e.to_string()
                                )
                            }
                        }
                        return;
                    }
                    DeemakDev::Restore { output, password } => {
                        if !SekaiOperation::Restore.is_present(possible_sekai_opers) {
                            SekaiOperation::Restore.log_err();
                            exit_deemak(1);
                        }
                        log::log_info("SEKAI", "Restoring Sekai from Deemak Encrypted file");
                        let output_path = output.unwrap_or_else(|| {
                            std::env::current_dir()
                                .expect("Failed to get current directory")
                                .join("restored_deemak_sekai")
                        });
                        let mut output_obj = DeemakSekaiMgr::new(output_path, None);
                        // Get input password securely
                        output_obj.set_password(true).unwrap_or_else(|e| {
                            fatal_error!(
                                "SEKAI",
                                "Failed to set password for output Sekai: {}",
                                e.to_string()
                            );
                        });
                        match create_dmk_sekai::original_from_encrypted_sekai(
                            &sekai_obj.abs_path,
                            &output_obj.abs_path,
                            output_obj.password.as_deref(),
                        ) {
                            Ok(restored_path) => {
                                let absolute_restored_path = std::fs::canonicalize(&restored_path)
                                    .unwrap_or_else(|_| restored_path.clone());
                                pr_info!(
                                    "SEKAI",
                                    "Successfully restored Sekai to: {}",
                                    absolute_restored_path.display()
                                );
                            }
                            Err(e) => {
                                fatal_error!("SEKAI", "Restoration Failed: {}", e.to_string());
                            }
                        }
                        return;
                    }
                    DeemakDev::Play => {
                        // Make sure Play Criteria is met, as mentioned in file_mgr
                        if !SekaiOperation::Play.is_present(possible_sekai_opers) {
                            SekaiOperation::Play.log_err();
                            exit_deemak(1);
                        }
                        /*
                        Here, DEV_MODE is set, so logging will be verbose.
                        Also the temporary directory is set as the working directory.
                        See [`DeemakFileMgr::oper_allowed`] for more details.
                        */
                        log::log_info("DEEMAK", "Playing Sekai in Developer Mode");
                    }
                }
            }
            DeemakCommands::Auth => {
                // TODO: Implement authentication commands
                log::log_info("DEEMAK", "Authentication commands are not implemented yet.");
                exit_deemak(1);
            }
        }
    }

    /*
    If DEV_MODE is on, sekai_path can either be a file or a directory.
    In either case, _tmp_sekai_wd will be the current working directory since we want to easily see changes.
    If DEV_MODE is off, sekai_path must be a file, and it will be restored to a temporary directory.
    */

    // Deemak working should go inside a temporary directory.
    let _tmp_sekai_wd = sekai_obj.temp_location;
    let _sekai_from_enc_path = original_from_encrypted_sekai(&sekai_path, &_tmp_sekai_wd, None)
        .map_err(|e| {
            fatal_error!(
                "SEKAI",
                "Failed to restore Sekai: {}. Please check the Sekai file or directory.",
                e
            );
        })
        .unwrap();
    let sekai_enc_obj = DeemakSekaiMgr::new(_sekai_from_enc_path, None);

    // Automatically creates the Sekai dirinfo's if they do not exist and set the global SEKAI_DIR
    if !sekai_enc_obj.valid_sekai {
        fatal_error!(
            "SEKAI",
            "Restored Sekai path is not valid: {}. Please provide a valid Sekai file or directory.",
            sekai_enc_obj.abs_path.display()
        );
    }

    // NOTE: All Directory operations and variables settings should be done before this point.
    //
    // We have 2 modes, the web and the raylib gui. The web argument runs it on the web, else
    // raylib gui is set by default.
    //
    // NOTE: #############    WEB USAGE    #############
    //
    // Initialize the server if --web argument is provided
    if args.web {
        // TODO: Remove the extra sekai_no_hajimari call, it will be shifted to the server module
        // later on.
        sekai_initialize(&sekai_path);
        log::log_info("Application", "Running in web mode");
        // server::launch_web(sekai_dir.clone().unwrap());
        let _ = deemak::server::server();
        return;
    }

    // NOTE: #############    RAYLIB GUI USAGE    #############
    //
    // Initialize Raylib window
    if args.gui {
        unsafe {
            SetConfigFlags(4);
            SetTargetFPS(60);
        }
        let loglevel = if !debug_mode() {
            raylib::consts::TraceLogLevel::LOG_ERROR
        } else {
            raylib::consts::TraceLogLevel::LOG_ALL
        };

        let (mut rl, thread) = raylib::init()
            .log_level(loglevel)
            .size(800, 600)
            .title("DEEMAK Shell")
            .build();
        let font_size = get_monitor_width(0) as f32 / 73.5;
        rl.set_trace_log(loglevel);
        // Disable escape key exit to prevent accidental application closure
        unsafe {
            raylib::ffi::SetExitKey(0i32);
        }
        log::log_info("Application", "DEEMAK initialized successfully");

        // Show login screen before menu
        if !deemak::login::show_login(&mut rl, &thread, font_size) {
            log::log_info("Application", "Login aborted by user.");
            return; // Exit if window closed during login
        }

        // Run the GUI loop
        run_gui_loop(&mut rl, &thread, font_size);
    }
}
