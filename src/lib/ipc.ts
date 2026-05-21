import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AssetTree,
  DecompilePayload,
  DetectionResult,
  LocaleEntry,
  PreviewPayload,
  ProgressPayload,
  Project,
  ProjectHandle,
  RecentEntry,
  SearchResult,
  TranslatableRowWithDraft,
} from "./types";

/**
 * Typed wrappers around Tauri's `invoke()`.
 * Each function maps 1-to-1 with a Rust command in src-tauri/src/commands/.
 *
 * Error shape (thrown on rejection): `{ code: string; message: string }`
 * matching the Rust `AppError` serialization.
 */
export const ipc = {
  /** Detect which format handler can open the project at `path`. */
  detectProject: (path: string): Promise<DetectionResult> =>
    invoke<DetectionResult>("detect_project", { path }),

  /** Open a project folder, returning a lightweight handle. */
  openProject: (path: string): Promise<ProjectHandle> =>
    invoke<ProjectHandle>("open_project", { path }),

  /** Fetch a cached project record by id. */
  getProject: (id: string): Promise<Project> =>
    invoke<Project>("get_project", { id }),

  /** List all recent projects from the local cache. */
  listRecents: (): Promise<RecentEntry[]> =>
    invoke<RecentEntry[]>("list_recents"),

  /** Get the asset tree for a previously opened project handle. */
  tree: (handle: string): Promise<AssetTree> =>
    invoke<AssetTree>("tree", { handle }),

  /** Get a preview payload for a single asset node. */
  preview: (handle: string, nodeId: string): Promise<PreviewPayload> =>
    invoke<PreviewPayload>("preview", { handle, node_id: nodeId }),

  /** Export a single asset node to `dest` on disk. */
  export: (handle: string, nodeId: string, dest: string): Promise<void> =>
    invoke<void>("export", { handle, node_id: nodeId, dest }),

  /** Cancel a running operation by its operation id. */
  cancel: (operationId: string): Promise<void> =>
    invoke<void>("cancel", { operation_id: operationId }),

  /**
   * Read a raw byte slice from a file on disk.
   * Used by HexPreview for chunked / lazy loading of large files.
   * Returns an array of byte values (u8[]).
   * Max len per call capped at 1MB server-side.
   */
  readFileChunk: (path: string, offset: number, len: number): Promise<number[]> =>
    invoke<number[]>("read_file_chunk", { path, offset, len }),

  /**
   * Decompile a script asset node on demand.
   * Returns DecompilePayload with C# source, backend, confidence, and references.
   * Results are cached on disk; subsequent calls for the same node are <50ms.
   */
  decompile: (handle: ProjectHandle, nodeId: string): Promise<DecompilePayload> =>
    invoke<DecompilePayload>("decompile", { handle, node_id: nodeId }),

  /** List translatable rows for a project (triggers lazy scan on first call). */
  listTranslatable: (
    handleId: string,
    sourceLocale: string,
    targetLocale: string,
    filter?: string,
  ): Promise<TranslatableRowWithDraft[]> =>
    invoke<TranslatableRowWithDraft[]>("list_translatable", {
      handle_id: handleId,
      source_locale: sourceLocale,
      target_locale: targetLocale,
      filter: filter ?? null,
    }),

  /** Upsert a translation entry. */
  saveTranslation: (params: {
    rowId: string;
    targetLocale: string;
    text: string;
    status: string;
  }): Promise<void> =>
    invoke<void>("save_translation", {
      row_id: params.rowId,
      target_locale: params.targetLocale,
      text: params.text,
      status: params.status,
    }),

  /** List available source + target locales for a project. */
  listLocales: (handleId: string): Promise<LocaleEntry[]> =>
    invoke<LocaleEntry[]>("list_locales", { handle_id: handleId }),

  /** Export translations to a file on disk. */
  exportTranslations: (params: {
    handleId: string;
    targetLocale: string;
    format: "csv" | "json";
    destPath: string;
  }): Promise<void> =>
    invoke<void>("export_translations", {
      handle_id: params.handleId,
      target_locale: params.targetLocale,
      format: params.format,
      dest_path: params.destPath,
    }),

  /**
   * Full-text search across indexed assets, code, and strings.
   * Returns results ranked by weighted BM25 score.
   */
  search: (query: string, projectId?: string, limit?: number): Promise<SearchResult[]> =>
    invoke<SearchResult[]>("search", {
      query,
      project_id: projectId ?? null,
      limit: limit ?? null,
    }),

  /**
   * Subscribe to progress events for a specific operation.
   * Returns an unlisten function — call it to stop listening.
   *
   * @example
   * const unlisten = await ipc.subscribeProgress(opId, (p) => setProgress(p));
   * // later:
   * unlisten();
   */
  subscribeProgress: (
    operationId: string,
    cb: (p: ProgressPayload) => void,
  ): Promise<UnlistenFn> =>
    listen<ProgressPayload>("progress", (e) => {
      if (e.payload.operation_id === operationId) cb(e.payload);
    }),
} as const;

/**
 * Shape of an `AppError` as serialized by the Rust backend.
 * See src-tauri/src/domain/error.rs::AppError::serialize.
 */
export interface AppErrorShape {
  code: string;
  message: string;
}

/**
 * Normalize a Tauri IPC rejection into a stable {code, message} shape.
 * Tauri may throw either a serialized AppError object or a bare string
 * depending on where the failure occurred — this helper handles both.
 */
export function parseAppError(err: unknown): AppErrorShape {
  if (err && typeof err === "object") {
    const obj = err as Record<string, unknown>;
    if (typeof obj.code === "string" && typeof obj.message === "string") {
      return { code: obj.code, message: obj.message };
    }
  }
  return { code: "UNKNOWN", message: String(err) };
}
