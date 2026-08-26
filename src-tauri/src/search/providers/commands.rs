use crate::search::query::{ResultType, SearchCandidate};

pub struct CommandItem {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub command: &'static str,
}

const BUILTIN_COMMANDS: &[CommandItem] = &[
    CommandItem {
        id: "cmd_lock",
        name: "Lock Screen",
        description: "Lock your Windows workstation",
        command: "rundll32.exe user32.dll,LockWorkStation",
    },
    CommandItem {
        id: "cmd_taskmgr",
        name: "Open Task Manager",
        description: "Launch Windows Task Manager",
        command: "taskmgr.exe",
    },
    CommandItem {
        id: "cmd_recycle",
        name: "Open Recycle Bin",
        description: "View and manage deleted files",
        command: "explorer.exe shell:RecycleBinFolder",
    },
    CommandItem {
        id: "cmd_settings",
        name: "Windows Settings",
        description: "Open Windows System Settings",
        command: "ms-settings:",
    },
];

pub struct CommandsProvider;

impl CommandsProvider {
    pub fn search(query: &str) -> Vec<SearchCandidate> {
        let q_lower = query.to_lowercase();
        let mut results = Vec::new();

        for cmd in BUILTIN_COMMANDS {
            if cmd.name.to_lowercase().contains(&q_lower)
                || cmd.description.to_lowercase().contains(&q_lower)
            {
                results.push(SearchCandidate {
                    id: cmd.id.to_string(),
                    result_type: ResultType::Command,
                    display_name: cmd.name.to_string(),
                    subtitle: cmd.description.to_string(),
                    target_path: cmd.command.to_string(),
                    icon_id: None,
                    base_score: 0.75,
                });
            }
        }

        results
    }
}
