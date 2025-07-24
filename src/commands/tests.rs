#[cfg(test)]
mod commands_tests {
    use crate::commands::go::navigate;
    use crate::utils::test_utils::setup_test_dir;

    /// Layout of the temporary directory structure used in this test:
    /// temp_dir/
    /// ├── file1.txt
    /// ├── subdir1/
    /// │   ├── file2.txt
    /// │   ├── file3.txt
    /// │   └── nested1/
    /// │       └── file4.txt
    /// └── subdir2/
    ///     ├── file5.txt
    ///     └── nested2/
    ///         ├── file6.txt
    ///         └── file7.txt
    #[test]
    fn test_navigate() {
        let (_temp_dir, root_path) = setup_test_dir(true);
        let mut current_dir = root_path.clone();

        // Navigate to subdir1
        let (new_path, msg) = navigate("subdir1", &current_dir, &root_path);
        println!("Navigating to subdir1: {msg}");
        assert_eq!(new_path, root_path.join("subdir1"));

        // Navigate to nested1
        current_dir = new_path;
        let (new_path, _) = navigate("nested1", &current_dir, &root_path);
        assert_eq!(new_path, root_path.join("subdir1/nested1"));

        // Navigate back to subdir1
        current_dir = new_path;
        let (new_path, _) = navigate("..", &current_dir, &root_path);
        assert_eq!(new_path, root_path.join("subdir1"));

        // Navigate to subdir2 from subdir1
        current_dir = new_path;
        let (new_path, _) = navigate("../subdir2", &current_dir, &root_path);
        assert_eq!(new_path, root_path.join("subdir2"));

        // Navigate to HOME
        current_dir = new_path;
        let (new_path, _) = navigate("HOME", &current_dir, &root_path);
        assert_eq!(new_path, root_path);

        // Try to go back from root
        current_dir = new_path;
        let (new_path, message) = navigate("..", &current_dir, &root_path);
        assert_eq!(new_path, root_path);
        assert!(message.contains("You are at the root"));

        // Navigate to nonexistent directory
        let (new_path, message) = navigate("nonexistent", &current_dir, &root_path);
        assert_eq!(new_path, current_dir);
        assert!(message.contains("No such directory"));

        // Navigate to a file
        let (new_path, message) = navigate("file1.txt", &current_dir, &root_path);
        println!("{new_path:?}, {current_dir:?}, {message}");
        assert_eq!(new_path, current_dir);
        assert!(message.contains("Is a file"));

        // Navigate to a restricted directory
        let (new_path, message) = navigate(".dir_info", &current_dir, &root_path);
        assert_eq!(new_path, current_dir);
        assert!(message.contains("Attempted to go to/refers a restricted directory"));

        // Try patterns on destination
        current_dir = root_path.to_path_buf();
        let (new_path, message) =
            navigate("subdir1/../subdir2/../.dir_info", &current_dir, &root_path);
        assert_eq!(new_path, current_dir);
        assert!(message.contains("Attempted to go to/refers a restricted directory"));
    }
}
