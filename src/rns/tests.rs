#[cfg(test)]
mod rns_test {
    use crate::rns::restore_comp::{backup_sekai, can_restore, can_save, restore_sekai};
    use crate::utils::test_utils::{create_file, get_dir_contents, remove_file, setup_test_dir};
    use std::fs;
    use std::fs::File;

    /// Test to ensure that the can_restore and can_save functionality works correctly
    #[test]
    fn test_can_restore_and_can_save_functionality() {
        let (_temp_dir, root_path) = setup_test_dir();

        assert!(!can_restore(&root_path));
        assert!(!can_save(&root_path));

        fs::create_dir_all(root_path.join(".dir_info")).unwrap();
        File::create(root_path.join(".dir_info").join("restore_me")).unwrap();
        assert!(can_restore(&root_path));
        assert!(!can_save(&root_path));

        File::create(root_path.join(".dir_info").join("save_me")).unwrap();
        assert!(can_restore(&root_path));
        assert!(can_save(&root_path));
    }

    /// Test to ensure that backup and restore functionality handles errors correctly
    #[test]
    fn test_invalid_usage_errors() {
        let (_temp_dir, root_path) = setup_test_dir();

        let backup_result = backup_sekai("invalid_usage", &root_path);
        assert!(backup_result.is_err());
        assert_eq!(
            backup_result.err().unwrap().kind(),
            std::io::ErrorKind::InvalidInput
        );

        let restore_result = restore_sekai("invalid_usage", &root_path);
        assert!(restore_result.is_err());
        assert_eq!(
            restore_result.err().unwrap().kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    /// Test to ensure that backup and restore functionality is idempotent, i.e., calling backup
    /// multiple times does not create duplicate restore points
    #[test]
    fn test_backup_restore_idempotency() {
        let (_temp_dir, root_path) = setup_test_dir();

        let first_backup_result = backup_sekai("restore", &root_path).unwrap();
        assert!(first_backup_result.contains("created"));

        // Calling backup again for "restore" should skip creation
        let second_backup_result = backup_sekai("restore", &root_path).unwrap();
        assert!(second_backup_result.contains("skipping creation"));
    }

    /// Test to ensure that backup and restore functionality works correctly
    #[test]
    fn test_backup_and_restore() {
        let (_temp_dir, root_path) = setup_test_dir();
        let initial_contents = get_dir_contents(&root_path, false);

        // Create a restore point
        assert!(backup_sekai("restore", &root_path).is_ok());
        assert!(can_restore(&root_path));
        assert!(root_path.join(".dir_info").join("restore_me").exists());

        // Modify the directory
        remove_file(root_path.join("file1.txt"));
        create_file(root_path.join("subdir1/file2.txt"), "new content");

        // Restore from the restore point
        assert!(restore_sekai("restore", &root_path).is_ok());

        // Check if the directory is restored to its original state
        let restored_contents = get_dir_contents(&root_path, false);
        assert_eq!(initial_contents, restored_contents);
    }

    /// Test to ensure that backup and save functionality works correctly
    #[test]
    fn test_backup_and_save_flow() {
        let (_temp_dir, root_path) = setup_test_dir();

        // 1. Create initial state and a restore point
        let initial_contents = get_dir_contents(&root_path, false);
        assert!(backup_sekai("restore", &root_path).is_ok());
        assert!(backup_sekai("save", &root_path).is_ok());
        assert!(can_save(&root_path));

        // 2. Modify the directory and create a save point
        remove_file(root_path.join("file1.txt"));
        create_file(root_path.join("new_file.txt"), "new content");
        create_file(root_path.join("subdir1/new_file.txt"), "new content");
        let modified_contents_1 = get_dir_contents(&root_path, false);
        assert!(backup_sekai("save", &root_path).is_ok());

        // 3. Modify the directory again
        remove_file(root_path.join("subdir1/file2.txt"));
        create_file(
            root_path.join("subdir2/another_file.txt"),
            "another content",
        );
        let modified_contents_2 = get_dir_contents(&root_path, false);
        // restore to the first save point and check
        assert!(restore_sekai("save", &root_path).is_ok());
        assert_eq!(get_dir_contents(&root_path, false), modified_contents_1);

        // 4. Restore to the initial state
        assert!(restore_sekai("restore", &root_path).is_ok());
        assert_eq!(get_dir_contents(&root_path, false), initial_contents);
    }
}