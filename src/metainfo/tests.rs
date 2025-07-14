// #[cfg(test)]
// mod metainfo_test {
//     use crate::metainfo::info_reader::*;
//     use crate::utils::test_utils::{create_file, setup_test_dir};
//     use std::collections::HashMap;

//     /// Test to ensure that the info.json lifecycle works correctly
//     #[test]
//     fn test_info_json_lifecycle() {
//         // Setup test environment
//         let (_temp_dir, root_path) = setup_test_dir(false);
//         let info_path = root_path.join(".dir_info/info.json");
//         let file1_path = root_path.join("file1.txt");
//         let file2_path = root_path.join("file2.txt");

//         // Phase 1: Test adding objects
//         create_dummy_dir_info(&info_path, None);
//         create_file(&file1_path, "content");
//         create_file(&file2_path, "content");

//         // Add first object with default properties
//         add_obj_to_info(&file1_path, "file1.txt", None).unwrap();
//         let info = read_validate_info(&info_path).unwrap();
//         assert!(info.objects.contains_key("file1.txt"));
//         assert_eq!(
//             info.objects["file1.txt"].properties["locked"],
//             serde_json::Value::String("00".to_string())
//         );

//         // Add second object with custom properties
//         let mut custom_props = HashMap::new();
//         custom_props.insert("special".to_string(), serde_json::Value::Bool(true));
//         add_obj_to_info(&file2_path, "file2.txt", Some(custom_props)).unwrap();
//         let info = read_validate_info(&info_path).unwrap();
//         assert!(info.objects.contains_key("file2.txt"));
//         assert!(
//             info.objects["file2.txt"].properties["special"]
//                 .as_bool()
//                 .unwrap()
//         );

//         // Phase 2: Test updating objects
//         update_obj_status(
//             &file1_path,
//             "file1.txt",
//             "locked",
//             serde_json::Value::String("11".to_string()),
//         )
//         .unwrap();
//         let info = read_validate_info(&info_path).unwrap();
//         assert_eq!(
//             info.objects["file1.txt"].properties["locked"],
//             serde_json::Value::String("11".to_string())
//         );

//         // Phase 3: Test deleting objects
//         del_obj_from_info(&file1_path, "file1.txt").unwrap();
//         let info = read_validate_info(&info_path).unwrap();
//         assert!(!info.objects.contains_key("file1.txt"));
//         assert!(info.objects.contains_key("file2.txt")); // Other object remains
//     }

//     /// Test to ensure that reading and validating info.json works correctly
//     #[test]
//     fn test_read_validate_info() {
//         let (_temp_dir, root_path) = setup_test_dir();
//         let info_path = root_path.join(".dir_info/info.json");

//         // Test valid info.json
//         let valid_json = r#"{
//             "location": "valid_loc",
//             "about": "valid_about",
//             "objects": {
//                 "valid.txt": {"locked": "01"}
//             }
//         }"#;
//         create_file(&info_path, valid_json);
//         let info = read_validate_info(&info_path).unwrap();
//         assert_eq!(info.location, "valid_loc");
//         assert_eq!(info.about, "valid_about");
//         assert_eq!(
//             info.objects["valid.txt"].properties["locked"],
//             serde_json::Value::String("01".to_string())
//         );

//         // Test invalid locked format
//         let invalid_json = r#"{
//             "location": "invalid",
//             "about": "invalid",
//             "objects": {
//                 "invalid.txt": {"locked": "012"}
//             }
//         }"#;
//         create_file(&info_path, invalid_json);
//         let result = read_validate_info(&info_path);
//         assert!(matches!(result, Err(InfoError::ValidationError(_))));
//     }

//     #[test]
//     fn test_default_info_creation() {
//         let (_temp_dir, root_path) = setup_test_dir();
//         let dir = root_path.join("test_dir");
//         create_file(dir.join("existing.txt"), "content");

//         // Test home directory case
//         let home_info = Info::default_for_path(&root_path, true);
//         assert_eq!(home_info.location.to_lowercase(), "home");

//         // Test regular directory case
//         let dir_info = Info::default_for_path(&dir, false);
//         println!("{:?}", dir_info);
//         assert!(dir_info.location.to_lowercase().ends_with("test_dir"));
//         assert!(dir_info.objects.contains_key("existing.txt"));
//         assert_eq!(
//             dir_info.objects["existing.txt"].properties["locked"],
//             serde_json::Value::String("00".to_string())
//         );
//     }
// }
