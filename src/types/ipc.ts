import { SearchResponse, LaunchResponse, SearchResult } from './results';
import { AppSettings } from './settings';

export interface IndexStatus {
  is_indexing: boolean;
  total_applications: number;
  total_files: number;
  total_folders: number;
  last_indexed_at: number | null;
}

export interface AppInfo {
  name: string;
  version: string;
  tauri_version: string;
}

export interface IpcCommands {
  search: { args: { query: string }; response: SearchResponse };
  launch: { args: { id: string; result_type: string }; response: LaunchResponse };
  get_icon: { args: { id: string }; response: { data: string | null } };
  hide_launcher: { args: Record<string, never>; response: undefined };
  get_settings: { args: Record<string, never>; response: AppSettings };
  update_settings: { args: { settings: AppSettings }; response: boolean };
  get_recent_results: { args: Record<string, never>; response: SearchResult[] };
  get_index_status: { args: Record<string, never>; response: IndexStatus };
  rebuild_index: { args: Record<string, never>; response: undefined };
  get_app_info: { args: Record<string, never>; response: AppInfo };
}
