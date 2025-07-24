#[cfg(test)]
mod utils_tests {
    use crate::utils::find_root::relative_deemak_path;
    use std::path::{Path, PathBuf};
    pub const SEKAI_TEST_DIR: &str = "/home/deemak";

    #[test]
    fn test_relative_world_dir() {
        let sekai_dir = Path::new(SEKAI_TEST_DIR);
        assert_eq!(
            relative_deemak_path(sekai_dir, Some(sekai_dir)),
            PathBuf::from("HOME")
        );
        assert_eq!(
            relative_deemak_path(&sekai_dir.join("some/file.txt"), Some(sekai_dir)),
            PathBuf::from("HOME").join("some/file.txt")
        );

        // Temporary path testing
        let tmp_path = PathBuf::from("/tmp");
        assert_eq!(
            relative_deemak_path(&tmp_path, Some(sekai_dir)),
            PathBuf::from("DEEMAK_TEMP")
        );
        assert_eq!(
            relative_deemak_path(&tmp_path.join("temp_file.log"), Some(sekai_dir)),
            PathBuf::from("DEEMAK_TEMP").join("temp_file.log")
        );

        let other_path = PathBuf::from("/var/log/system.log");
        assert_eq!(
            relative_deemak_path(&other_path, Some(sekai_dir)),
            other_path
        );

        let relative_path = PathBuf::from("my_doc.txt");
        assert_eq!(
            relative_deemak_path(&relative_path, Some(sekai_dir)),
            relative_path
        );
    }

    #[test]
    fn test_filter_text() {
        use crate::utils::log::filter_msg;
        let sekai_dir = Some(Path::new(SEKAI_TEST_DIR));
        let text0 = "This is a test message with no paths and no sekai_dir specified.";
        assert_eq!(filter_msg(text0, None), text0);
        let text1 = "Test /home/deemak/some/file.txt, this is a test message.";
        assert_eq!(
            filter_msg(text1, sekai_dir),
            "Test HOME/some/file.txt, this is a test message."
        );
        let text2 = "Another test with /tmp/temp_file.log and some other text.";
        assert_eq!(
            filter_msg(text2, sekai_dir),
            "Another test with DEEMAK_TEMP/temp_file.log and some other text."
        );
        let text3 = "No paths here, just a simple message. my_doc.txt should remain unchanged.";
        assert_eq!(filter_msg(text3, sekai_dir), text3);
        let text4 = "Path within only apostrophe: '/home/deemak/some/file.txt', \"/home/deemak/another_file\" should be replaced.";
        assert_eq!(
            filter_msg(text4, sekai_dir),
            "Path within only apostrophe: 'HOME/some/file.txt', \"HOME/another_file\" should be replaced."
        );
        let text5 = "Multiple paths: /home/deemak/some/file.txt, /home/deemak/other_some, /tmp/temp_file.log, and /var/log/system.log.";
        assert_eq!(
            filter_msg(text5, sekai_dir),
            "Multiple paths: HOME/some/file.txt, HOME/other_some, DEEMAK_TEMP/temp_file.log, and /var/log/system.log."
        );
    }
}
