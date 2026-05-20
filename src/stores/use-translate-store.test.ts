// use-translate-store.test.ts — unit tests for useTranslateStore Zustand actions.

import { beforeEach, describe, expect, test } from "vitest";
import { useTranslateStore } from "./use-translate-store";

beforeEach(() => {
  useTranslateStore.setState({
    sourceLocale: "en",
    targetLocale: "vi",
    filter: "all",
    dirty: {},
  });
});

describe("useTranslateStore — locale mutators", () => {
  test("setSourceLocale updates sourceLocale", () => {
    useTranslateStore.getState().setSourceLocale("ja");
    expect(useTranslateStore.getState().sourceLocale).toBe("ja");
  });

  test("setTargetLocale updates targetLocale", () => {
    useTranslateStore.getState().setTargetLocale("fr");
    expect(useTranslateStore.getState().targetLocale).toBe("fr");
  });

  test("setSourceLocale and setTargetLocale are independent", () => {
    useTranslateStore.getState().setSourceLocale("ko");
    useTranslateStore.getState().setTargetLocale("de");
    expect(useTranslateStore.getState().sourceLocale).toBe("ko");
    expect(useTranslateStore.getState().targetLocale).toBe("de");
  });
});

describe("useTranslateStore — setFilter", () => {
  test("setFilter updates filter", () => {
    useTranslateStore.getState().setFilter("translated");
    expect(useTranslateStore.getState().filter).toBe("translated");
  });

  test("setFilter accepts all valid values", () => {
    for (const f of ["all", "translated", "pending", "review"] as const) {
      useTranslateStore.getState().setFilter(f);
      expect(useTranslateStore.getState().filter).toBe(f);
    }
  });
});

describe("useTranslateStore — updateDraft", () => {
  test("updateDraft stores draft text per rowId", () => {
    useTranslateStore.getState().updateDraft("row-1", "translated text");
    expect(useTranslateStore.getState().dirty["row-1"]).toBe("translated text");
  });

  test("updateDraft stores drafts independently per rowId", () => {
    useTranslateStore.getState().updateDraft("row-1", "text A");
    useTranslateStore.getState().updateDraft("row-2", "text B");
    expect(useTranslateStore.getState().dirty["row-1"]).toBe("text A");
    expect(useTranslateStore.getState().dirty["row-2"]).toBe("text B");
  });

  test("updateDraft overwrites previous draft for same rowId", () => {
    useTranslateStore.getState().updateDraft("row-1", "first");
    useTranslateStore.getState().updateDraft("row-1", "second");
    expect(useTranslateStore.getState().dirty["row-1"]).toBe("second");
  });
});

describe("useTranslateStore — clearDraft (flush)", () => {
  test("clearDraft removes the entry for a rowId", () => {
    useTranslateStore.getState().updateDraft("row-1", "some text");
    useTranslateStore.getState().clearDraft("row-1");
    expect(useTranslateStore.getState().dirty["row-1"]).toBeUndefined();
  });

  test("clearDraft leaves other drafts intact", () => {
    useTranslateStore.getState().updateDraft("row-1", "text A");
    useTranslateStore.getState().updateDraft("row-2", "text B");
    useTranslateStore.getState().clearDraft("row-1");
    expect(useTranslateStore.getState().dirty["row-2"]).toBe("text B");
  });

  test("clearDraft on non-existent key is a no-op", () => {
    useTranslateStore.getState().clearDraft("no-such-row");
    expect(useTranslateStore.getState().dirty).toEqual({});
  });
});
