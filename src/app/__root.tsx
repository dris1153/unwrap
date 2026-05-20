import { createRootRoute, Outlet } from "@tanstack/react-router";
import { AppShell } from "../components/app-shell/app-shell";
import { CommandPalette } from "../features/command-palette/command-palette";
import { GlobalShortcuts } from "../features/command-palette/shortcuts";

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  return (
    <AppShell>
      <Outlet />
      <GlobalShortcuts />
      <CommandPalette />
    </AppShell>
  );
}
