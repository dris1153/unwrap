import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import { ErrorToast } from "./error-toast";
import { useErrorStore } from "../../stores/use-error-store";

describe("ErrorToast", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    useErrorStore.getState().dismiss();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders nothing when no error is present", () => {
    const { container } = render(<ErrorToast />);
    expect(container.firstChild).toBeNull();
  });

  it("renders title, message, and log hint when shown", () => {
    render(<ErrorToast />);

    act(() => {
      useErrorStore.getState().show({
        title: "Couldn't open project",
        message: "Sidecar download failed",
        logHint: "%LOCALAPPDATA%\\Unwrap\\Unwrap\\data\\logs\\",
      });
    });

    expect(screen.getByText("Couldn't open project")).toBeInTheDocument();
    expect(screen.getByText("Sidecar download failed")).toBeInTheDocument();
    expect(
      screen.getByText("%LOCALAPPDATA%\\Unwrap\\Unwrap\\data\\logs\\"),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Copy log path")).toBeInTheDocument();
  });

  it("does NOT render the log hint when not provided", () => {
    render(<ErrorToast />);

    act(() => {
      useErrorStore.getState().show({
        title: "Generic error",
        message: "Something went wrong",
      });
    });

    expect(screen.queryByLabelText("Copy log path")).not.toBeInTheDocument();
  });

  it("auto-dismisses after 10 seconds", () => {
    render(<ErrorToast />);

    act(() => {
      useErrorStore.getState().show({
        title: "Auto-dismiss test",
        message: "Will disappear",
      });
    });

    expect(screen.getByText("Auto-dismiss test")).toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(10_000);
    });

    expect(screen.queryByText("Auto-dismiss test")).not.toBeInTheDocument();
    expect(useErrorStore.getState().current).toBeNull();
  });

  it("can be manually dismissed via the close button", () => {
    render(<ErrorToast />);

    act(() => {
      useErrorStore.getState().show({
        title: "Manual dismiss",
        message: "Click X",
      });
    });

    expect(screen.getByText("Manual dismiss")).toBeInTheDocument();

    act(() => {
      screen.getByLabelText("Dismiss").click();
    });

    expect(screen.queryByText("Manual dismiss")).not.toBeInTheDocument();
  });
});
