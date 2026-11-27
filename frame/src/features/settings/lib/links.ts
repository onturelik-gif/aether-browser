import * as m from "$shared/i18n/messages";
import { IS_MAC } from "$shared/platform";

export const WEBSITE = "https://github.com/onturelik-gif/aether-browser";
export const RELEASES = "https://github.com/onturelik-gif/aether-browser/releases/latest";

export const links = [
  { id: "website", url: WEBSITE, label: () => m.about_link_website() },
  {
    id: "docs",
    url: `${WEBSITE}#readme`,
    label: () => (IS_MAC ? m.about_link_docs() : m.native_help()),
  },
  { id: "github", url: WEBSITE, label: () => "GitHub" },
] as const;
