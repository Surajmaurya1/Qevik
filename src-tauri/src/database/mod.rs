pub mod apps;
pub mod connection;
pub mod files;
pub mod folders;
pub mod history;
pub mod migrations;
pub mod settings;
pub mod usage;

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        migrations::run_migrations(&mut conn).unwrap();
        conn
    }

    #[test]
    fn test_application_search_and_lookup() {
        let mut conn = setup_test_db();
        let apps = vec![
            apps::ApplicationRecord {
                id: "app_np".into(),
                display_name: "Notepad".into(),
                exe_path: "C:\\Windows\\System32\\notepad.exe".into(),
                shortcut_path: Some("C:\\Start\\Notepad.lnk".into()),
                arguments: None,
                icon_path: None,
                icon_index: 0,
                source: "System".into(),
                indexed_at: 100,
                updated_at: 100,
            },
            apps::ApplicationRecord {
                id: "app_ag".into(),
                display_name: "Antigravity IDE".into(),
                exe_path: "C:\\Programs\\Antigravity IDE.exe".into(),
                shortcut_path: Some("C:\\Start\\Antigravity IDE.lnk".into()),
                arguments: None,
                icon_path: None,
                icon_index: 0,
                source: "Programs".into(),
                indexed_at: 100,
                updated_at: 100,
            },
            apps::ApplicationRecord {
                id: "app_node".into(),
                display_name: "Node.js command prompt".into(),
                exe_path: "C:\\Tools\\node.exe".into(),
                shortcut_path: None,
                arguments: None,
                icon_path: None,
                icon_index: 0,
                source: "Node".into(),
                indexed_at: 100,
                updated_at: 100,
            },
        ];

        apps::upsert_applications(&mut conn, &apps).unwrap();

        // Exact match
        let res = apps::search_applications_fts(&conn, "notepad", 10).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].display_name, "Notepad");

        // Partial substring match
        let res = apps::search_applications_fts(&conn, "pad", 10).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].display_name, "Notepad");

        // Substring and multi-word
        let res = apps::search_applications_fts(&conn, "anti", 10).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].display_name, "Antigravity IDE");

        let res = apps::search_applications_fts(&conn, "grav", 10).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].display_name, "Antigravity IDE");

        // Special characters (dot in query)
        let res = apps::search_applications_fts(&conn, "node.js", 10).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].display_name, "Node.js command prompt");

        // Lookup by ID
        let found = apps::get_application_by_id_or_path(&conn, "app_ag").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().display_name, "Antigravity IDE");
    }

    #[test]
    fn test_file_and_folder_search_and_lookup() {
        let mut conn = setup_test_db();
        let files = vec![
            files::FileRecord {
                id: "file_txt".into(),
                name: "my_notes.txt".into(),
                display_name: "my_notes".into(),
                extension: Some("txt".into()),
                path: "C:\\Users\\Test\\Documents\\my_notes.txt".into(),
                parent_dir: "C:\\Users\\Test\\Documents".into(),
                size_bytes: 1024,
                modified_at: 100,
                indexed_at: 100,
                is_hidden: false,
                is_system: false,
            },
            files::FileRecord {
                id: "file_pdf".into(),
                name: "invoice_2026.pdf".into(),
                display_name: "invoice_2026".into(),
                extension: Some("pdf".into()),
                path: "C:\\Users\\Test\\Downloads\\invoice_2026.pdf".into(),
                parent_dir: "C:\\Users\\Test\\Downloads".into(),
                size_bytes: 2048,
                modified_at: 100,
                indexed_at: 100,
                is_hidden: false,
                is_system: false,
            },
        ];

        let folders = vec![folders::FolderRecord {
            id: "folder_doc".into(),
            name: "Documents".into(),
            path: "C:\\Users\\Test\\Documents".into(),
            parent_dir: "C:\\Users\\Test".into(),
            indexed_at: 100,
        }];

        files::upsert_files(&mut conn, &files).unwrap();
        folders::upsert_folders(&mut conn, &folders).unwrap();

        // Search file by extension with dot
        let res = files::search_files_fts(&conn, "my_notes.txt", 10).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "my_notes.txt");

        // Search file by extension
        let res = files::search_files_fts(&conn, "pdf", 10).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "invoice_2026.pdf");

        // Search folder
        let res = folders::search_folders_fts(&conn, "Documents", 10).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "Documents");

        // Lookup file by ID
        let found = files::get_file_by_id_or_path(&conn, "file_txt").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().path, "C:\\Users\\Test\\Documents\\my_notes.txt");

        // Lookup folder by ID
        let found_folder = folders::get_folder_by_id_or_path(&conn, "folder_doc").unwrap();
        assert!(found_folder.is_some());
        assert_eq!(found_folder.unwrap().path, "C:\\Users\\Test\\Documents");
    }
}

