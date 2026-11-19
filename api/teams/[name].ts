import { handler, pathName } from "../_lib/http.js";
import { loadTeam } from "../_lib/team.js";

export const GET = handler((url) => loadTeam(pathName(url)));
