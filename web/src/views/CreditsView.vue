<script setup lang="ts">
import InfoRow from "../components/InfoRow.vue";

const licenses = [
  {
    code: "Code",
    title: "Transfer Board",
    detail: "MIT License",
    href: "/licenses/transfer-board.txt",
    titleClass: "font-display",
  },
  {
    code: "Font",
    title: "Atkinson Hyperlegible Next",
    detail: "SIL Open Font License",
    href: "/licenses/atkinson-hyperlegible-next.txt",
    titleClass: "font-body",
  },
  {
    code: "Font",
    title: "Big Shoulders Display",
    detail: "SIL Open Font License",
    href: "/licenses/big-shoulders-display.txt",
    titleClass: "font-display",
  },
];
</script>

<template>
  <h1 class="mb-8 type-heading-lg">Data and credits</h1>

  <article class="grid gap-12 type-body-md lg:w-2/3">
    <div class="grid max-w-prose gap-12">
      <section class="grid gap-2" aria-labelledby="data-heading">
        <h2 id="data-heading" class="type-heading-md">Roster and transfer data</h2>
        <p>
          Official roster and player data comes from
          <a
            href="https://lol.fandom.com/wiki/Leaguepedia:Copyrights"
            class="underline hover:text-primary"
            >Leaguepedia and its contributors</a
          >. Transfer Board standardizes names and groups related roster events into transfers. The
          Leaguepedia data and those adaptations are available under
          <a
            href="https://creativecommons.org/licenses/by-sa/3.0/"
            class="underline hover:text-primary"
            >CC BY-SA 3.0</a
          >. Rumors link to the reports they are based on.
        </p>
      </section>

      <section class="grid gap-2" aria-labelledby="images-heading">
        <h2 id="images-heading" class="type-heading-md">Images and trademarks</h2>
        <p>
          Player portraits and team logos are sourced through Leaguepedia and served by Fandom’s
          image CDN. Rights to these images and trademarks belong to their respective holders.
          League of Legends and the League of Legends logo are Riot Games marks. Transfer Board is
          an independent fan project. Contact us through the
          <RouterLink to="/privacy" class="underline hover:text-primary">privacy page</RouterLink>
          to report an image or attribution issue.
        </p>
      </section>
    </div>

    <section aria-labelledby="software-heading">
      <h2 id="software-heading" class="mb-4 type-heading-md">Software and fonts</h2>
      <ul class="flex flex-col gap-px overflow-hidden rounded-panel bg-line">
        <InfoRow v-for="license in licenses" :key="license.href" v-bind="license" />
      </ul>
    </section>
  </article>
</template>
