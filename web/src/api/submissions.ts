import type { RumorSubmission, SubmissionResponse } from "@lol/shared";
import { useMutation } from "@tanstack/vue-query";
import { HttpError } from "./client";

async function submitRumor(rumor: RumorSubmission): Promise<SubmissionResponse> {
  const path = "/api/submissions";

  const response = await fetch(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(rumor),
  });

  if (!response.ok) throw new HttpError(response.status, path, "POST");

  return response.json();
}

export function useSubmitRumor() {
  return useMutation({ mutationFn: submitRumor });
}
