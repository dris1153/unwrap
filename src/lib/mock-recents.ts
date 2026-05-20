/** Static mock data for the Welcome screen recent-projects grid. */
export type RecentProject = {
  id: string;
  name: string;
  path: string;
  unityVersion: string;
  assetCount: number;
  openedLabel: string;
  /** Chip variant shown top-right of tile */
  chip?: { label: string; variant: "pinned" | "il2cpp" | "mono" };
  /** Icon/color theme for the tile icon */
  iconTheme: "tex" | "mesh" | "script" | "txt";
};

export const MOCK_RECENTS: RecentProject[] = [
  {
    id: "hollow-vale",
    name: "Hollow Vale",
    path: "C:\\Games\\HollowVale\\HollowVale_Data",
    unityVersion: "Unity 2022.3.18",
    assetCount: 14238,
    openedLabel: "opened 3h ago",
    chip: { label: "pinned", variant: "pinned" },
    iconTheme: "tex",
  },
  {
    id: "ferrocity-demo",
    name: "Ferrocity Demo",
    path: "D:\\Builds\\Ferrocity\\demo_v0.4.2",
    unityVersion: "Unity 2021.3.9",
    assetCount: 8921,
    openedLabel: "yesterday",
    chip: { label: "il2cpp", variant: "il2cpp" },
    iconTheme: "mesh",
  },
  {
    id: "knight-of-the-tin-drum",
    name: "Knight of the Tin Drum",
    path: "E:\\Steam\\steamapps\\common\\KOTD",
    unityVersion: "Unity 2023.1.4",
    assetCount: 22447,
    openedLabel: "4 days ago",
    chip: { label: "mono", variant: "mono" },
    iconTheme: "script",
  },
  {
    id: "wandersmith",
    name: "Wandersmith",
    path: "C:\\itch\\wandersmith-win64",
    unityVersion: "Unity 6000.0.18",
    assetCount: 5612,
    openedLabel: "last week",
    chip: { label: "mono", variant: "mono" },
    iconTheme: "txt",
  },
];
