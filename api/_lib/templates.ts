import type { MoveKind } from "@lol/shared";
import { publicationDraft, type Submission } from "./submission.js";

// Mirrors the DESIGN.md tokens; email clients cannot read web/src/style.css.
const C = {
  primary: "#f2c230",
  onPrimary: "#0e2a2b",
  background: "#0e2a2b",
  surface: "#133536",
  surfaceRaised: "#1a4243",
  line: "#2a5556",
  text: "#eef3ef",
  muted: "#a9bdb8",
};
const DISPLAY = "'Big Shoulders Display','Arial Narrow',Arial,sans-serif";
const BODY = "'Atkinson Hyperlegible Next','Segoe UI',Helvetica,Arial,sans-serif";

const KIND_LABELS: Record<MoveKind, string> = {
  transfer: "Transfer",
  free_agent_signing: "Free agent signing",
  release: "Release",
  loan: "Loan",
  promotion: "Promotion",
  bench: "Benched",
  sub: "Substitute",
  retirement: "Retirement",
};

export function escape(value: string): string {
  return value.replace(
    /[&<>"']/g,
    (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[char]!,
  );
}

const label = `font-family:${BODY};font-size:13px;font-weight:700;line-height:1.2;letter-spacing:0.01em`;
const bodyMd = `font-family:${BODY};font-size:16px;line-height:1.5`;
const bodySm = `font-family:${BODY};font-size:13px;line-height:1.45;letter-spacing:0.01em`;
const headingMd = `font-family:${DISPLAY};font-size:25px;font-weight:700;line-height:1.15;letter-spacing:0.01em`;
const boardCode = `font-family:${DISPLAY};font-size:31px;font-weight:800;line-height:1;letter-spacing:0.02em`;
const button = `display:inline-block;height:40px;padding:0 16px;border-radius:6px;text-decoration:none;${label};line-height:40px`;
const primaryButton = `${button};background:${C.primary};color:${C.onPrimary}`;
const secondaryButton = `${button};background:${C.surfaceRaised};color:${C.text}`;

function flap(team: string | null, fallback: string): string {
  return `<span style="display:inline-block;padding:8px 12px;border-radius:3px;background:${C.surfaceRaised};color:${C.text};${boardCode}">${escape(team ?? fallback)}</span>`;
}

function reliabilityDots(value: number): string {
  return Array.from(
    { length: 5 },
    (_, index) =>
      `<span style="display:inline-block;width:6px;height:6px;margin-right:2px;border-radius:9999px;background:${index < value ? C.primary : C.line}"></span>`,
  ).join("");
}

function row(name: string, value: string): string {
  return `<tr>
    <td style="padding:12px 0;border-top:1px solid ${C.line};color:${C.muted};${bodySm};width:120px;vertical-align:top">${name}</td>
    <td style="padding:12px 0;border-top:1px solid ${C.line};color:${C.text};${bodyMd}">${value}</td>
  </tr>`;
}

function details(rumor: Submission): string {
  // The link text is the real host so a crafted outlet name cannot disguise where it points.
  const source = rumor.sourceUrl
    ? `<a href="${escape(rumor.sourceUrl)}" style="color:${C.text};text-decoration:underline">${escape(new URL(rumor.sourceUrl).hostname)}</a>${rumor.outlet ? ` <span style="color:${C.muted}">· ${escape(rumor.outlet)}</span>` : ""}`
    : "A public source URL is required.";
  const rows = [
    row("Player", escape(rumor.player)),
    row("From", escape(rumor.fromTeam ?? "Free agent")),
    row("To", escape(rumor.toTeam ?? "Leaves the scene")),
    rumor.role && row("Role", escape(rumor.role)),
    rumor.kind && row("Move", escape(KIND_LABELS[rumor.kind])),
    row("Reported", escape(rumor.reportedAt)),
    row("Source", source),
    row(
      "Reliability",
      `${reliabilityDots(rumor.reliability)} <span style="color:${C.muted};${bodySm}">${rumor.reliability} of 5</span>`,
    ),
    rumor.notes && row("Notes", escape(rumor.notes).replaceAll("\n", "<br>")),
  ];
  return `<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="border-collapse:collapse">${rows.filter(Boolean).join("")}</table>`;
}

function board(rumor: Submission): string {
  return `<p style="margin:0 0 8px">
      <span style="display:inline-block;padding:4px 10px;border-radius:9999px;border:1.5px dashed ${C.primary};color:${C.primary};${label}">Rumor</span>
    </p>
    <p style="margin:0 0 4px;color:${C.text};font-family:${DISPLAY};font-size:25px;font-weight:600;line-height:1.1">${escape(rumor.player)}</p>
    <p style="margin:0 0 24px">${flap(rumor.fromTeam, "Free agent")} <span style="color:${C.muted};${bodyMd};padding:0 6px">▸</span> ${flap(rumor.toTeam, "Out")}</p>`;
}

function layout(title: string, content: string): string {
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="color-scheme" content="dark">
<meta name="supported-color-schemes" content="dark">
<title>${escape(title)}</title>
</head>
<body style="margin:0;padding:0;background:${C.background};color:${C.text}">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background:${C.background}">
  <tr><td align="center" style="padding:32px 16px">
    <table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="max-width:560px">
      <tr><td style="padding:0 0 16px;border-bottom:1px solid ${C.line};color:${C.text};${headingMd}">Transfer Board</td></tr>
      <tr><td style="padding:24px 0">
        <div style="background:${C.surface};border-radius:10px;padding:24px">${content}</div>
      </td></tr>
      <tr><td style="color:${C.muted};${bodySm}">Official roster data from Leaguepedia, licensed CC BY-SA.</td></tr>
    </table>
  </td></tr>
</table>
</body>
</html>`;
}

export function reviewEmail(
  rumor: Submission,
  reviewUrl: (decision: "accept" | "reject") => string,
) {
  const subject = `New rumor to review: ${rumor.player}`;
  const html = layout(
    subject,
    `<h1 style="margin:0 0 4px;color:${C.text};${headingMd}">Someone sent a rumor</h1>
    <p style="margin:0 0 24px;color:${C.muted};${bodyMd}">Review it before it goes on the board.</p>
    ${board(rumor)}
    ${details(rumor)}
    <p style="margin:24px 0 0">
      <a href="${escape(reviewUrl("accept"))}" style="${primaryButton}">Review and accept</a>
      &nbsp;
      <a href="${escape(reviewUrl("reject"))}" style="${secondaryButton}">Reject</a>
    </p>
    <p style="margin:16px 0 0;color:${C.muted};${bodySm}">Each button opens a confirmation page. The links expire in 14 days.</p>`,
  );
  const text = [
    "Someone sent a rumor to Transfer Board.",
    "",
    `Player: ${rumor.player}`,
    `From: ${rumor.fromTeam ?? "Free agent"}`,
    `To: ${rumor.toTeam ?? "Leaves the scene"}`,
    rumor.role && `Role: ${rumor.role}`,
    rumor.kind && `Move: ${KIND_LABELS[rumor.kind]}`,
    `Reported: ${rumor.reportedAt}`,
    `Source: ${rumor.outlet ? `${rumor.outlet} ` : ""}${rumor.sourceUrl}`,
    `Reliability: ${rumor.reliability} of 5`,
    rumor.notes && `Notes: ${rumor.notes}`,
    "",
    `Accept: ${reviewUrl("accept")}`,
    `Reject: ${reviewUrl("reject")}`,
  ]
    .filter((line) => line !== null)
    .join("\n");
  return { subject, html, text };
}

export function reviewPage(
  rumor: Submission,
  token: string,
  decision: "accept" | "reject",
  error?: string,
): string {
  const accept = decision === "accept";
  const draft = publicationDraft(rumor);
  const sourceField = `<label for="source-url" style="${label}">Public source URL</label>
      <input id="source-url" name="sourceUrl" type="url" required maxlength="500" value="${escape(draft.sourceUrl)}" aria-describedby="source-help" style="box-sizing:border-box;width:100%;margin:6px 0;padding:8px;${bodySm}">
      <p id="source-help" style="margin:0 0 16px;${bodySm}">Use a link that anyone can open. Remove private or tracking parameters and check that the link still opens the report.</p>`;
  return layout(
    accept ? "Accept rumor" : "Reject rumor",
    `<h1 style="margin:0 0 4px;color:${C.text};${headingMd}">${accept ? "Accept this rumor?" : "Reject this rumor?"}</h1>
    <p style="margin:0 0 24px;color:${C.muted};${bodyMd}">${
      accept
        ? "These details will be added to the public GitHub repository and appear on the board after the next sync. Private notes stay in the review email."
        : "The submission will not be published."
    }</p>
    ${board(rumor)}
    ${details(draft)}
    <form method="post" action="/api/review" style="margin:24px 0 0">
      <input type="hidden" name="token" value="${escape(token)}">
      ${error ? `<p role="alert" style="${bodyMd}">${escape(error)}</p>` : ""}
      ${accept ? sourceField : ""}
      <button type="submit" name="decision" value="${decision}" style="${accept ? primaryButton : secondaryButton};border:0;cursor:pointer">${accept ? "Accept rumor" : "Reject rumor"}</button>
    </form>`,
  );
}

export function messagePage(
  title: string,
  message: string,
  link?: { href: string; text: string },
): string {
  return layout(
    title,
    `<h1 style="margin:0 0 4px;color:${C.text};${headingMd}">${escape(title)}</h1>
    <p style="margin:0;color:${C.muted};${bodyMd}">${escape(message)}</p>
    ${link ? `<p style="margin:24px 0 0"><a href="${escape(link.href)}" style="${secondaryButton}">${escape(link.text)}</a></p>` : ""}`,
  );
}
