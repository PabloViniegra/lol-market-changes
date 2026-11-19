const FROM = "Transfer Board <rumors@send.pabloviniegra.dev>";

export async function sendEmail(message: { subject: string; html: string; text: string }) {
  const apiKey = process.env.RESEND_API_KEY;
  const to = process.env.REVIEW_EMAIL_TO;
  if (!apiKey || !to) throw new Error("RESEND_API_KEY and REVIEW_EMAIL_TO must be configured");
  const response = await fetch("https://api.resend.com/emails", {
    method: "POST",
    headers: { Authorization: `Bearer ${apiKey}`, "Content-Type": "application/json" },
    body: JSON.stringify({ from: FROM, to: [to], ...message }),
  });
  if (!response.ok) {
    throw new Error(`Resend failed with ${response.status}`);
  }
}
