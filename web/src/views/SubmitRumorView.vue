<script setup lang="ts">
import type { RumorSubmission } from "@lol/shared";
import { useForm } from "vee-validate";
import { computed, ref } from "vue";
import { HttpError } from "../api/client";
import { z } from "zod";
import { useSubmitRumor } from "../api/submissions";
import { useTeams } from "../api/teams";
import FormField from "../components/FormField.vue";
import RumorPreview from "../components/RumorPreview.vue";
import WarningIcon from "../components/WarningIcon.vue";
import { zodSchema } from "../forms";
import { KIND_LABELS, KINDS, ROLES } from "../format";

const today = () => {
  const now = new Date();

  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
};

const UNSAFE = /[\p{Cc}\u2028\u2029\uFEFF]/u;
const HIDDEN = "Remove hidden characters, they often come from copy and paste.";

const clean = (value: string) => value.isWellFormed() && !UNSAFE.test(value);

const schema = z
  .object({
    player: z
      .string()
      .trim()
      .min(1, "Enter the player’s name.")
      .max(60, "Keep the name under 60 characters.")
      .refine(clean, HIDDEN),
    fromTeam: z.string().trim().max(80, "Keep the team under 80 characters.").refine(clean, HIDDEN),
    toTeam: z.string().trim().max(80, "Keep the team under 80 characters.").refine(clean, HIDDEN),
    role: z.enum([...ROLES, ""]),
    kind: z.enum([...KINDS, ""]),
    reportedAt: z
      .string()
      .regex(/^\d{4}-\d{2}-\d{2}$/, "Pick the date it was reported.")
      .refine((date) => date <= today(), "The date can’t be in the future."),
    sourceUrl: z
      .url({ protocol: /^https?$/, error: "Paste a link that starts with https://." })
      .max(500, "Use a shorter link.")
      .refine(clean, HIDDEN),
    outlet: z.string().trim().max(80, "Keep the outlet under 80 characters.").refine(clean, HIDDEN),
    reliability: z.number().int().min(1).max(5),
    notes: z
      .string()
      .trim()
      .max(500, "Keep notes under 500 characters.")
      .refine((value) => clean(value.replaceAll("\n", "")), HIDDEN),
  })
  .refine((values) => values.fromTeam || values.toTeam, {
    path: ["toTeam"],
    error: "Enter at least one team.",
  });

const initialValues = {
  player: "",
  fromTeam: "",
  toTeam: "",
  role: "",
  kind: "",
  reportedAt: today(),
  sourceUrl: "",
  outlet: "",
  reliability: 3,
  notes: "",
} as const;

const { defineField, errors, handleSubmit, resetForm, submitCount } = useForm({
  validationSchema: zodSchema(schema),
  initialValues,
});

const lazy = { validateOnModelUpdate: false } as const;
const [player, playerAttrs] = defineField("player", lazy);
const [fromTeam, fromTeamAttrs] = defineField("fromTeam", lazy);
const [toTeam, toTeamAttrs] = defineField("toTeam", lazy);
const [role, roleAttrs] = defineField("role", lazy);
const [kind, kindAttrs] = defineField("kind", lazy);
const [reportedAt, reportedAtAttrs] = defineField("reportedAt", lazy);
const [sourceUrl, sourceUrlAttrs] = defineField("sourceUrl", lazy);
const [outlet, outletAttrs] = defineField("outlet", lazy);
const [reliability, reliabilityAttrs] = defineField("reliability", lazy);
const [notes, notesAttrs] = defineField("notes", lazy);

const website = ref("");
const teams = useTeams();
const submit = useSubmitRumor();

const onSubmit = handleSubmit(
  (values) => {
    const rumor: RumorSubmission = {
      player: values.player,
      fromTeam: values.fromTeam || null,
      toTeam: values.toTeam || null,
      role: values.role || null,
      kind: values.kind || null,
      reportedAt: values.reportedAt,
      sourceUrl: values.sourceUrl,
      outlet: values.outlet || null,
      reliability: values.reliability,
      notes: values.notes || null,
      website: website.value,
    };

    submit.mutate(rumor);
  },
  ({ errors: failed }) => {
    const first = Object.keys(failed)[0];

    if (first) document.getElementById(`rumor-${first}`)?.focus();
  },
);

const failure = computed(() => {
  const error = submit.error.value;
  const status = error instanceof HttpError ? error.status : 0;

  if (status === 429) return "You’ve sent several rumors in a short time. Try again in 10 minutes.";

  if (status === 400) return "The rumor wasn’t accepted. Check the fields and try again.";

  return "Couldn’t send the rumor. Try again in a moment.";
});

function sendAnother() {
  submit.reset();
  resetForm({ values: { ...initialValues, reportedAt: today() } });
}

const input =
  "h-10 w-full rounded-control border-2 border-transparent bg-surface-raised px-3 type-body-md text-text placeholder:text-text-muted aria-invalid:border-text";
</script>

<template>
  <div class="grid gap-x-8 gap-y-6 lg:grid-cols-12">
    <div class="lg:col-span-8">
      <h1 class="type-heading-lg">Send a rumor</h1>
      <p class="mt-1 type-body-md text-text-muted">
        Heard about a move before it’s official? Send it with a source. Every rumor is reviewed
        before it goes on the board.
      </p>
    </div>

    <section
      v-if="submit.isSuccess.value"
      class="rounded-panel bg-surface p-6 lg:col-span-8"
      role="status"
      aria-labelledby="sent-heading"
    >
      <h2 id="sent-heading" class="type-heading-md">Thanks, your rumor is in review</h2>
      <p class="mt-2 type-body-md text-text-muted">
        If it’s accepted, it shows in the rumor tracker after the next sync.
      </p>
      <div class="mt-6 flex flex-wrap gap-3">
        <RouterLink
          to="/rumors"
          class="inline-flex h-10 items-center rounded-control bg-primary px-4 type-label-md text-on-primary hover:bg-primary-hover"
        >
          Open the rumor tracker
        </RouterLink>
        <button
          type="button"
          class="h-10 rounded-control bg-surface-raised px-4 type-label-md hover:bg-line"
          @click="sendAnother"
        >
          Send another rumor
        </button>
      </div>
    </section>

    <template v-else>
      <form
        class="flex flex-col gap-8 rounded-panel bg-surface p-6 lg:col-span-8"
        novalidate
        @submit.prevent="onSubmit"
      >
        <fieldset class="flex flex-col gap-6">
          <legend class="mb-4 type-heading-md">The move</legend>
          <FormField
            id="rumor-player"
            v-slot="{ describedBy, invalid }"
            label="Player"
            hint="Use the name on Leaguepedia, for example Caps."
            :error="errors.player"
          >
            <input
              id="rumor-player"
              v-model="player"
              v-bind="playerAttrs"
              :aria-describedby="describedBy"
              :aria-invalid="invalid || undefined"
              autocomplete="off"
              :class="input"
            />
          </FormField>

          <div class="grid gap-6 md:grid-cols-2">
            <FormField
              id="rumor-fromTeam"
              v-slot="{ describedBy, invalid }"
              label="From team"
              hint="Leave empty for a free agent."
              :error="errors.fromTeam"
            >
              <input
                id="rumor-fromTeam"
                v-model="fromTeam"
                v-bind="fromTeamAttrs"
                list="rumor-teams"
                :aria-describedby="describedBy"
                :aria-invalid="invalid || undefined"
                autocomplete="off"
                :class="input"
              />
            </FormField>
            <FormField
              id="rumor-toTeam"
              v-slot="{ describedBy, invalid }"
              label="To team"
              hint="Leave empty if they leave the scene."
              :error="errors.toTeam"
            >
              <input
                id="rumor-toTeam"
                v-model="toTeam"
                v-bind="toTeamAttrs"
                list="rumor-teams"
                :aria-describedby="describedBy"
                :aria-invalid="invalid || undefined"
                autocomplete="off"
                :class="input"
              />
            </FormField>
            <datalist id="rumor-teams">
              <option v-for="team in teams.data.value?.teams" :key="team.name" :value="team.name" />
            </datalist>

            <FormField
              id="rumor-role"
              v-slot="{ describedBy }"
              label="Role"
              optional
              :error="errors.role"
            >
              <select
                id="rumor-role"
                v-model="role"
                v-bind="roleAttrs"
                :aria-describedby="describedBy"
                :class="input"
              >
                <option value="">Not sure</option>
                <option v-for="item in ROLES" :key="item" :value="item">{{ item }}</option>
              </select>
            </FormField>
            <FormField
              id="rumor-kind"
              v-slot="{ describedBy }"
              label="Move"
              optional
              :error="errors.kind"
            >
              <select
                id="rumor-kind"
                v-model="kind"
                v-bind="kindAttrs"
                :aria-describedby="describedBy"
                :class="input"
              >
                <option value="">Not sure</option>
                <option v-for="item in KINDS" :key="item" :value="item">
                  {{ KIND_LABELS[item] }}
                </option>
              </select>
            </FormField>
          </div>
        </fieldset>

        <fieldset class="flex flex-col gap-6">
          <legend class="mb-4 type-heading-md">The source</legend>
          <div class="grid gap-6 md:grid-cols-2">
            <FormField
              id="rumor-sourceUrl"
              v-slot="{ describedBy, invalid }"
              label="Source link"
              :error="errors.sourceUrl"
            >
              <input
                id="rumor-sourceUrl"
                v-model="sourceUrl"
                v-bind="sourceUrlAttrs"
                type="url"
                inputmode="url"
                placeholder="https://"
                :aria-describedby="describedBy"
                :aria-invalid="invalid || undefined"
                :class="input"
              />
            </FormField>
            <FormField
              id="rumor-outlet"
              v-slot="{ describedBy, invalid }"
              label="Outlet"
              optional
              hint="For example, Sheep Esports."
              :error="errors.outlet"
            >
              <input
                id="rumor-outlet"
                v-model="outlet"
                v-bind="outletAttrs"
                :aria-describedby="describedBy"
                :aria-invalid="invalid || undefined"
                :class="input"
              />
            </FormField>

            <FormField
              id="rumor-reportedAt"
              v-slot="{ describedBy, invalid }"
              label="Reported on"
              :error="errors.reportedAt"
            >
              <input
                id="rumor-reportedAt"
                v-model="reportedAt"
                v-bind="reportedAtAttrs"
                type="date"
                :max="today()"
                :aria-describedby="describedBy"
                :aria-invalid="invalid || undefined"
                :class="input"
              />
            </FormField>
            <FormField
              id="rumor-reliability"
              v-slot="{ describedBy }"
              label="Reliability"
              hint="How trustworthy is the source?"
              :error="errors.reliability"
            >
              <select
                id="rumor-reliability"
                v-model.number="reliability"
                v-bind="reliabilityAttrs"
                :aria-describedby="describedBy"
                :class="input"
              >
                <option :value="1">1 · Weak</option>
                <option :value="2">2 · Shaky</option>
                <option :value="3">3 · Plausible</option>
                <option :value="4">4 · Reliable</option>
                <option :value="5">5 · Very reliable</option>
              </select>
            </FormField>
          </div>

          <FormField
            id="rumor-notes"
            v-slot="{ describedBy, invalid }"
            label="Notes"
            optional
            :error="errors.notes"
          >
            <textarea
              id="rumor-notes"
              v-model="notes"
              v-bind="notesAttrs"
              rows="3"
              :aria-describedby="describedBy"
              :aria-invalid="invalid || undefined"
              :class="input"
              class="h-auto py-2"
            />
          </FormField>
        </fieldset>

        <div class="sr-only" aria-hidden="true">
          <label for="rumor-website">Website</label>
          <input id="rumor-website" v-model="website" tabindex="-1" autocomplete="off" />
        </div>

        <div class="flex flex-col gap-4">
          <p
            v-if="submit.isError.value"
            class="flex items-center gap-2 rounded-control bg-surface-raised px-4 py-3 type-body-sm"
            role="alert"
          >
            <WarningIcon />
            {{ failure }}
          </p>
          <p v-else-if="submitCount > 0 && Object.keys(errors).length" class="sr-only" role="alert">
            Some fields need fixing.
          </p>

          <div class="flex flex-wrap gap-3">
            <button
              type="submit"
              class="h-10 rounded-control bg-primary px-4 type-label-md text-on-primary hover:bg-primary-hover disabled:opacity-60"
              :disabled="submit.isPending.value"
            >
              {{ submit.isPending.value ? "Sending…" : "Send for review" }}
            </button>
            <RouterLink
              to="/rumors"
              class="inline-flex h-10 items-center rounded-control bg-surface-raised px-4 type-label-md hover:bg-line"
            >
              Cancel
            </RouterLink>
          </div>
        </div>
      </form>

      <aside class="max-lg:hidden lg:col-span-4">
        <RumorPreview
          class="sticky top-6"
          :player="player ?? ''"
          :from-team="fromTeam ?? ''"
          :to-team="toTeam ?? ''"
          :role="role ?? ''"
          :kind="kind ?? ''"
          :reliability="reliability ?? 3"
          :teams="teams.data.value?.teams ?? []"
        />
      </aside>
    </template>
  </div>
</template>
