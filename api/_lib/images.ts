import { createHash } from "node:crypto";

const FILE_PATH = "https://lol.fandom.com/wiki/Special:FilePath/";
const CDN = "https://static.wikia.nocookie.net/lolesports_gamepedia_en/images";

// Special:FilePath answers hotlinked <img> requests with 403, so images point at the CDN path
// MediaWiki derives from the MD5 of the file name.
export function thumbnail(file: string, width: number): string {
  const name = file.trim().replaceAll(" ", "_");
  const title = name.charAt(0).toUpperCase() + name.slice(1);
  const hash = createHash("md5").update(title).digest("hex");
  return `${CDN}/${hash[0]}/${hash.slice(0, 2)}/${encodeURIComponent(title)}/revision/latest/scale-to-width-down/${width}`;
}

export function storedImage(url: string | null, width: number): string | null {
  if (!url?.startsWith(FILE_PATH)) return null;
  return thumbnail(url.slice(FILE_PATH.length), width);
}

export function teamLogo(team: string | null, width = 64): string | null {
  return team ? thumbnail(`${team}logo square.png`, width) : null;
}
