import * as React from "react";
import { TooltipProvider } from "../ui/tooltip";

type AppShellProps = {
  children: React.ReactNode;
};

/**
 * Root shell wrapper — sets viewport, provides tooltip context.
 * Each route composes TitleBar + body + StatusBar inside this shell.
 */
export function AppShell({ children }: AppShellProps) {
  return (
    <TooltipProvider>
      <div
        className="flex flex-col bg-base text-primary font-sans antialiased"
        style={{ width: "100vw", height: "100vh", overflow: "hidden" }}
      >
        {children}
      </div>
    </TooltipProvider>
  );
}
