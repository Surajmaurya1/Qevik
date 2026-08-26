# Search & Ranking Architecture

## Scoring Formula

Each search candidate is evaluated by the scoring function in `src-tauri/src/search/ranking.rs`:

```
score = text_relevance_score
      + match_type_bonus
      + type_priority_bonus
      + usage_frequency_score
      + recency_score
```

### Components:

- **`text_relevance_score`:**
  - Exact match: `1.0`
  - Prefix match: `0.85`
  - Token prefix match: `0.70`
  - Contains match: `0.50`
  - Fallback: `0.20`
- **`match_type_bonus`:**
  - Exact: `+0.30`
  - Prefix: `+0.20`
  - Token: `+0.10`
- **`type_priority_bonus`:**
  - Calculator: `+0.25`
  - Application: `+0.20`
  - Command: `+0.15`
  - Folder: `+0.10`
  - File: `+0.05`
  - Web: `+0.00`
- **`usage_frequency_score`:**
  `usage_score = min(launch_count / 50.0, 1.0) * 0.5`
- **`recency_score`:**
  `recency_score = max(0.0, 0.3 - (hours_since_last_launch / 168.0) * 0.3)`
