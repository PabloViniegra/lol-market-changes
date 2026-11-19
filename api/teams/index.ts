import type { TeamsResponse } from "@lol/shared";
import { db, handler } from "../_lib/http.js";

export const GET = handler(async (): Promise<TeamsResponse> => {
  const rows = await db()`select name, short, league from teams
                          where league <> 'other'
                          order by name`;
  return { teams: rows.map(({ name, short, league }) => ({ name, short, league })) };
});
