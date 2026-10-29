import type { RouteName } from "./types";

export interface NavItem {
  id: RouteName;
  icon: string;
  label: string;
}

/** Primary browse destinations shared by Sidebar and PhoneTabs. */
export const browseTabs: NavItem[] = [
  { id: "home", icon: "home", label: "nav.home" },
  { id: "videos", icon: "video", label: "nav.videos" },
  { id: "music", icon: "music", label: "nav.music" },
  { id: "playlists", icon: "playlist", label: "nav.playlists" },
  { id: "recent", icon: "recent", label: "nav.recent" },
];
