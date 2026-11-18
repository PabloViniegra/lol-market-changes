export type SyncStatus = "ok" | "error" | "never";

export interface MetaResponse {
  lastRunAt: string | null;
  lastRunStatus: SyncStatus;
  attribution: {
    text: string;
    url: string;
  };
}
