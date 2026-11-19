<script setup lang="ts">
import { ref } from "vue";
import InfoRow from "../components/InfoRow.vue";
import { privacyController, privacyEmail } from "../privacy";

const flows = [
  {
    from: "You",
    code: "Site",
    title: "Visiting the site",
    detail: "Vercel may process IP addresses, request details and technical logs.",
    href: "#visiting",
  },
  {
    from: "You",
    code: "Images",
    title: "Portraits and logos",
    detail: "Fandom’s image CDN: IP address, request information and a possible security cookie.",
    href: "#visiting",
  },
  {
    from: "You",
    code: "Form",
    title: "Sending a rumor",
    detail: "Resend and the reviewer: your report and any private notes.",
    href: "#rumors",
  },
  {
    from: "Form",
    code: "Public",
    title: "Approved reports",
    detail: "GitHub, Neon and the board: the report and its public source. Private notes stay out.",
    href: "#records",
  },
  {
    from: "You",
    code: "Mail",
    title: "Privacy requests",
    detail: "Google Gmail: your request and the information you include.",
    href: "#rights",
  },
];

const sections = [
  { id: "visiting", title: "Visiting the site" },
  { id: "rumors", title: "Sending a rumor" },
  { id: "records", title: "Player profiles and transfer records" },
  { id: "providers", title: "Providers and international transfers" },
  { id: "rights", title: "Your rights" },
];

const copyState = ref<"idle" | "copied" | "failed">("idle");
let resetCopied: ReturnType<typeof setTimeout> | undefined;

const copyLabel = {
  idle: "Copy email",
  copied: "Email copied",
  failed: "Couldn’t copy. Select the address above.",
};

async function copyEmail() {
  try {
    await navigator.clipboard.writeText(privacyEmail);
    copyState.value = "copied";
  } catch {
    copyState.value = "failed";
  }

  clearTimeout(resetCopied);
  resetCopied = setTimeout(() => (copyState.value = "idle"), 3000);
}
</script>

<template>
  <header class="mb-8 grid max-w-prose gap-2">
    <h1 class="type-heading-lg">Privacy</h1>
    <p class="text-text-muted">
      {{ privacyController }}, based in Spain, is responsible for Transfer Board. For privacy
      requests, contact
      <a :href="`mailto:${privacyEmail}`" class="text-text underline hover:text-primary">{{
        privacyEmail
      }}</a
      >.
    </p>
  </header>

  <div class="grid gap-12 lg:grid-cols-12 lg:gap-x-6">
    <article class="grid gap-12 type-body-md lg:col-span-8">
      <section aria-labelledby="flows-heading">
        <h2 id="flows-heading" class="mb-4 type-heading-md">Where your data goes</h2>
        <ul class="flex flex-col gap-px overflow-hidden rounded-panel bg-line">
          <InfoRow v-for="flow in flows" :key="flow.title" v-bind="flow" />
        </ul>
      </section>

      <div class="grid max-w-prose gap-12">
        <section id="visiting" class="grid scroll-mt-6 gap-2" aria-labelledby="visiting-heading">
          <h2 id="visiting-heading" class="type-heading-md">Visiting the site</h2>
          <p>
            Vercel handles traffic and may process IP addresses, request details and technical logs
            to serve the site, diagnose errors and limit abuse. The legal basis is the legitimate
            interest in keeping the site available and secure (GDPR Article 6(1)(f)). Technical
            records are kept for as long as needed for those purposes, subject to the provider’s log
            retention settings.
          </p>
          <p>
            Your browser loads player portraits and team logos from Fandom’s image CDN, which
            receives your IP address and request information to deliver and protect the images. Its
            service may set a Cloudflare security cookie to detect automated traffic. This
            processing supports the legitimate interests in displaying the board and securing
            delivery.
          </p>
        </section>

        <section id="rumors" class="grid scroll-mt-6 gap-2" aria-labelledby="rumors-heading">
          <h2 id="rumors-heading" class="type-heading-md">Sending a rumor</h2>
          <p>
            To review a submission, we use the move details, public source, date, reliability rating
            and any private notes you choose to include. You can browse the site without sending
            anything. The form marks the information needed to review a report. If you omit it, the
            report cannot be sent. Leave out private information about yourself and other people.
          </p>
          <p>
            Resend delivers your submission to the reviewer by email. The reviewer decides whether
            to publish it. If approved, the report and its public source are added to the GitHub
            repository, stored in Neon and shown on the board. Private notes stay in the review
            email. Review links expire after 14 days. Review emails are kept only while needed to
            decide a submission or handle a related dispute or request.
          </p>
        </section>

        <section id="records" class="grid scroll-mt-6 gap-2" aria-labelledby="records-heading">
          <h2 id="records-heading" class="type-heading-md">Player profiles and transfer records</h2>
          <p>
            Leaguepedia and public esports reporting provide names or handles, portraits, country,
            playing role, team affiliations, dates and reports about moves. Approved contributions
            can also inform the transfer record. Player profiles, transfers and published rumors are
            available to anyone on the website or its public API. Approved reports also appear in
            the public GitHub history, which others may have copied.
          </p>
          <p>
            The legal basis for reviewing submissions and publishing the record is the legitimate
            interest in documenting professional esports moves and distinguishing confirmed moves
            from reports (GDPR Article 6(1)(f)). Records are kept while relevant to the public
            transfer history and are subject to correction or removal requests.
          </p>
        </section>

        <section id="providers" class="grid scroll-mt-6 gap-2" aria-labelledby="providers-heading">
          <h2 id="providers-heading" class="type-heading-md">
            Providers and international transfers
          </h2>
          <p>
            Vercel hosts the site, Resend and the reviewer’s email service handle submissions,
            GitHub stores and processes the public project records, and Neon hosts the database.
            Fandom and Cloudflare deliver and protect the images. Google Gmail handles email to the
            privacy contact. Providers and their subcontractors may process information outside the
            European Economic Area, including in the United States. Transfer safeguards depend on
            the provider and transfer, including adequacy decisions where applicable and standard
            contractual clauses. You can request details about the safeguards using the contact
            above.
          </p>
        </section>

        <section id="rights" class="grid scroll-mt-6 gap-2" aria-labelledby="rights-heading">
          <h2 id="rights-heading" class="type-heading-md">Your rights</h2>
          <p>
            You can request access, correction, erasure or restriction, object to processing based
            on legitimate interests and request portability where it applies. Use the email above
            and include the page or report you are writing about. We use the information you provide
            to meet our legal obligations under the GDPR (Article 6(1)(c)) and keep it for as long
            as needed to handle the request and demonstrate how it was resolved. You can also
            complain to the
            <a href="https://www.aepd.es/" class="underline hover:text-primary"
              >Spanish Data Protection Agency (AEPD)</a
            >.
          </p>
        </section>
      </div>
    </article>

    <aside
      class="grid content-start gap-6 self-start rounded-panel bg-surface p-6 max-lg:hidden lg:sticky lg:top-6 lg:col-span-4"
      aria-label="Privacy contact"
    >
      <div class="grid gap-3">
        <p class="type-label-md">Privacy contact</p>
        <p class="type-body-sm text-text-muted">{{ privacyController }}, Spain</p>
        <a
          :href="`mailto:${privacyEmail}`"
          class="type-data-tabular wrap-anywhere underline hover:text-primary"
          >{{ privacyEmail }}</a
        >
        <button
          type="button"
          class="inline-flex h-10 items-center gap-2 justify-self-start rounded-control bg-surface-raised px-4 type-label-md hover:bg-line"
          @click="copyEmail"
        >
          <svg
            class="size-5"
            viewBox="0 0 20 20"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path v-if="copyState === 'copied'" d="m4.5 10.5 3.5 3.5 7.5-8" class="text-primary" />
            <template v-else>
              <rect x="7" y="7" width="10" height="10" rx="1.5" />
              <path
                d="M13 7V4.5A1.5 1.5 0 0 0 11.5 3h-7A1.5 1.5 0 0 0 3 4.5v7A1.5 1.5 0 0 0 4.5 13H7"
              />
            </template>
          </svg>
          <span aria-live="polite">{{ copyLabel[copyState] }}</span>
        </button>
      </div>

      <nav aria-labelledby="toc-heading" class="grid gap-2 border-t border-line pt-6">
        <p id="toc-heading" class="type-label-md">On this page</p>
        <a
          v-for="section in sections"
          :key="section.id"
          :href="`#${section.id}`"
          class="type-body-sm text-text-muted hover:text-text"
          >{{ section.title }}</a
        >
      </nav>
    </aside>
  </div>
</template>
