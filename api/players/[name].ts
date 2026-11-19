import { handler, pathName } from "../_lib/http.js";
import { loadPlayer } from "../_lib/player.js";

export const GET = handler((url) => loadPlayer(pathName(url)));
