// test-setup.ts — Vitest global setup for jsdom environment.
// Stubs out Tauri runtime APIs so tests run without a real Tauri host.

import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// Stub @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  convertFileSrc: (path: string) => `asset://${path}`,
}));

// Stub @tauri-apps/api/event
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

// Stub @tauri-apps/api/window
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    close: vi.fn(),
    onDragDropEvent: vi.fn(() => Promise.resolve(() => {})),
  }),
}));
