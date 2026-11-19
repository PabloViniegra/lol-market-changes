<script setup lang="ts">
import { privacyController, privacyEmail } from "../privacy";
</script>

<template>
  <article class="grid gap-6 type-body-md lg:w-2/3">
    <header class="grid gap-2">
      <h1 class="type-heading-lg">Privacy</h1>
      <p>
        {{ privacyController }}, based in Spain, is responsible for Transfer Board. For privacy
        requests, contact
        <a :href="`mailto:${privacyEmail}`" class="underline hover:text-primary">{{
          privacyEmail
        }}</a
        >.
      </p>
    </header>

    <section class="grid gap-2">
      <h2 class="type-heading-md">Visiting the site</h2>
      <p>
        Vercel handles traffic and may process IP addresses, request details and technical logs to
        serve the site, diagnose errors and limit abuse. The legal basis is the legitimate interest
        in keeping the site available and secure (GDPR Article 6(1)(f)). Technical records are kept
        for as long as needed for those purposes, subject to the provider’s log retention settings.
      </p>
      <p>
        Your browser loads player portraits and team logos from Fandom’s image CDN, which receives
        your IP address and request information to deliver and protect the images. Its service may
        set a Cloudflare security cookie to detect automated traffic. This processing supports the
        legitimate interests in displaying the board and securing delivery.
      </p>
    </section>

    <section class="grid gap-2">
      <h2 class="type-heading-md">Sending a rumor</h2>
      <p>
        To review a submission, we use the move details, public source, date, reliability rating and
        any private notes you choose to include. You can browse the site without sending anything.
        The form marks the information needed to review a report. If you omit it, the report cannot
        be sent. Leave out private information about yourself and other people.
      </p>
      <p>
        Resend delivers your submission to the reviewer by email. The reviewer decides whether to
        publish it. If approved, the report and its public source are added to the GitHub
        repository, stored in Neon and shown on the board. Private notes stay in the review email.
        Review links expire after 14 days. Review emails are kept only while needed to decide a
        submission or handle a related dispute or request.
      </p>
    </section>

    <section class="grid gap-2">
      <h2 class="type-heading-md">Player profiles and transfer records</h2>
      <p>
        Leaguepedia and public esports reporting provide names or handles, portraits, country,
        playing role, team affiliations, dates and reports about moves. Approved contributions can
        also inform the transfer record. Player profiles, transfers and published rumors are
        available to anyone on the website or its public API. Approved reports also appear in the
        public GitHub history, which others may have copied.
      </p>
      <p>
        The legal basis for reviewing submissions and publishing the record is the legitimate
        interest in documenting professional esports moves and distinguishing confirmed moves from
        reports (GDPR Article 6(1)(f)). Records are kept while relevant to the public transfer
        history and are subject to correction or removal requests.
      </p>
    </section>

    <section class="grid gap-2">
      <h2 class="type-heading-md">Providers and international transfers</h2>
      <p>
        Vercel hosts the site, Resend and the reviewer’s email service handle submissions, GitHub
        stores and processes the public project records, and Neon hosts the database. Fandom and
        Cloudflare deliver and protect the images. Google Gmail handles email to the privacy
        contact. Providers and their subcontractors may process information outside the European
        Economic Area, including in the United States. Transfer safeguards depend on the provider
        and transfer, including adequacy decisions where applicable and standard contractual
        clauses. You can request details about the safeguards using the contact above.
      </p>
    </section>

    <section class="grid gap-2">
      <h2 class="type-heading-md">Your rights</h2>
      <p>
        You can request access, correction, erasure or restriction, object to processing based on
        legitimate interests and request portability where it applies. Use the email above and
        include the page or report you are writing about. We use the information you provide to meet
        our legal obligations under the GDPR (Article 6(1)(c)) and keep it for as long as needed to
        handle the request and demonstrate how it was resolved. You can also complain to the
        <a href="https://www.aepd.es/" class="underline hover:text-primary"
          >Spanish Data Protection Agency (AEPD)</a
        >.
      </p>
    </section>
  </article>
</template>
