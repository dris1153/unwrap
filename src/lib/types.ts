/**
 * TypeScript mirrors of the Rust domain types in src-tauri/src/domain/.
 * Kept in sync manually for now; codegen via ts-rs deferred to phase-10.
 */

// ---------------------------------------------------------------------------
// Project
// ---------------------------------------------------------------------------

export type ScriptingBackend = "mono" | "il2cpp" | "unknown";

export interface Project {
  id: { "0": string };
  root_path: string;
  format_id: string;
  engine_version: string | null;
  scripting_backend: ScriptingBackend;
  /** Unix timestamp (seconds). */
  created_at: number;
  /** Unix timestamp (seconds). */
  last_opened: number;
}

export interface RecentEntry {
  project: Project;
  pinned: boolean;
}

export interface ProjectHandle {
  id: { "0": string };
  root_path: string;
  format_id: string;
}

// ---------------------------------------------------------------------------
// Asset tree
// ---------------------------------------------------------------------------

export type AssetKind =
  | "folder"
  | "texture"
  | "audio"
  | "mesh"
  | "text"
  | "script"
  | "scene"
  | "material"
  | "shader"
  | "animation"
  | "prefab"
  | "binary";

export interface AssetNode {
  id: { "0": string };
  parent: { "0": string } | null;
  name: string;
  kind: AssetKind;
  size: number;
  source_path: string;
  metadata: Record<string, unknown>;
}

export interface AssetTree {
  root: { "0": string };
  /** Map from NodeId string to AssetNode. */
  nodes: Record<string, AssetNode>;
}

// ---------------------------------------------------------------------------
// Preview
// ---------------------------------------------------------------------------

export type ModelFormat = "glb" | "fbx" | "obj";

export interface CodeRef {
  symbol: string;
  file: string | null;
  line: number | null;
}

export type PreviewPayload =
  | { type: "image"; path: string; width: number; height: number }
  | { type: "audio"; path: string; duration_ms: number }
  | { type: "model3_d"; path: string; format: ModelFormat }
  | { type: "text"; content: string; language: string | null }
  | {
      type: "code";
      content: string;
      language: string;
      references: CodeRef[];
      // Phase-07 decompile fields (null when raw .cs preview from preview command).
      source: string | null;
      backend: ScriptingBackend | null;
      confidence: number | null;
      class_fullname: string | null;
      assembly: string | null;
      elapsed_ms: number | null;
    }
  | { type: "hex"; path: string; size: number }
  | { type: "empty" };

// ---------------------------------------------------------------------------
// Decompile
// ---------------------------------------------------------------------------

/** Returned by the `decompile` IPC command (phase-07). */
export interface DecompilePayload {
  source: string; // "raw" | "ilspy" | "il2cpp-dump"
  backend: ScriptingBackend;
  confidence: number;
  language: string;
  content: string;
  class_fullname: string | null;
  assembly: string | null;
  references: CodeRef[];
  elapsed_ms: number;
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

export interface DetectionCandidate {
  handler_id: string;
  display_name: string;
  confidence: number;
  reason: string | null;
}

export type DetectionResult =
  | { type: "auto"; candidate: DetectionCandidate }
  | { type: "chooser"; candidates: DetectionCandidate[] };

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

export interface ProgressPayload {
  operation_id: string;
  /** e.g. "extracting" | "indexing" | "decompiling" */
  phase: string;
  /** 0–100, or null if indeterminate. */
  percent: number | null;
  message: string;
}

// ---------------------------------------------------------------------------
// Translate
// ---------------------------------------------------------------------------

export type TranslationStatus = "Pending" | "Translated" | "Review";

export interface TranslatableRow {
  id: string;
  project_id: string;
  source_path: string;
  key: string;
  comment: string | null;
  source_locale: string;
  source_text: string;
  detected_at: number;
}

export interface TranslationEntry {
  row_id: string;
  target_locale: string;
  text: string;
  status: TranslationStatus;
  updated_at: number;
}

export interface TranslatableRowWithDraft {
  id: string;
  project_id: string;
  source_path: string;
  key: string;
  comment: string | null;
  source_locale: string;
  source_text: string;
  detected_at: number;
  translation: TranslationEntry | null;
}

export interface LocaleEntry {
  code: string;
  count: number;
  kind: "source" | "target";
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

export type SearchResultKind = "Asset" | "Code" | "String" | "Command";

export type SearchAction =
  | { type: "OpenAsset"; project_id: string; node_id: string }
  | { type: "OpenCode"; project_id: string; node_id: string; line: number | null }
  | { type: "OpenString"; project_id: string; row_id: string }
  | { type: "Command"; command_id: string };

export interface SearchResult {
  kind: SearchResultKind;
  /** ref_id from the FTS index, or built-in command id */
  id: string;
  label: string;
  sublabel: string;
  /** Phosphor icon name */
  icon: string;
  project_id: string | null;
  action: SearchAction;
  /** Weighted BM25 score (higher = more relevant) */
  score: number;
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

export interface AppError {
  code:
    | "NOT_IMPLEMENTED"
    | "IO_ERROR"
    | "DB_ERROR"
    | "DETECT_FAILED"
    | "SIDECAR_FAILED"
    | "INVALID_PATH"
    | "CANCELLED"
    | "TIMEOUT";
  message: string;
}
