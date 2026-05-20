import { createFileRoute } from "@tanstack/react-router";
import { TitleBar } from "../components/app-shell/title-bar";
import { StatusBar } from "../components/app-shell/status-bar";
import { WelcomeHero } from "../features/welcome/welcome-hero";
import { RecentProjectsGrid } from "../features/welcome/recent-projects-grid";

export const Route = createFileRoute("/")({
  component: WelcomeRoute,
});

function WelcomeRoute() {
  return (
    <>
      <TitleBar mode="welcome" />

      <main className="flex-1 hero-glow relative" style={{ overflow: "hidden" }}>
        {/* Subtle grid tick overlay */}
        <div className="absolute inset-0 grid-tick opacity-40 pointer-events-none" />

        {/* 60/40 split */}
        <div
          className="relative h-full flex"
          style={{ padding: "56px 64px 40px 64px" }}
        >
          <WelcomeHero />
          <RecentProjectsGrid />
        </div>
      </main>

      <StatusBar mode="welcome" />
    </>
  );
}
