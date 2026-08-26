export type ResultType = 'app' | 'file' | 'folder' | 'command' | 'calculator' | 'web';

export interface SearchResult {
  id: string;
  result_type: ResultType;
  display_name: string;
  subtitle: string;
  score: number;
  icon_id: string | null;
}

export interface SearchResponse {
  results: SearchResult[];
  duration_ms: number;
}

export interface LaunchPayload {
  id: string;
  result_type: ResultType;
}

export interface LaunchResponse {
  success: boolean;
  error?: string;
}
