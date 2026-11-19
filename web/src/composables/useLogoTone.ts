import { reactive } from "vue";

// Leaguepedia logos are drawn for a light page; mostly near-black ones vanish on petrol.
const DARK_LUMINANCE = 0.06;
const DARK_SHARE = 0.5;
const SAMPLE = 32;

const darkLogos = reactive(new Map<string, boolean>());

const linear = (channel: number) => {
  const value = channel / 255;

  return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
};

export function measureLogo(src: string, image: HTMLImageElement): void {
  if (darkLogos.has(src)) return;

  const canvas = document.createElement("canvas");
  canvas.width = SAMPLE;
  canvas.height = SAMPLE;
  const context = canvas.getContext("2d", { willReadFrequently: true });

  if (!context) return;

  try {
    context.drawImage(image, 0, 0, SAMPLE, SAMPLE);
    const pixels = context.getImageData(0, 0, SAMPLE, SAMPLE).data;
    let opaque = 0;
    let dark = 0;

    for (let index = 0; index < pixels.length; index += 4) {
      if ((pixels[index + 3] ?? 0) < 128) continue;

      opaque += 1;

      const luminance =
        0.2126 * linear(pixels[index] ?? 0) +
        0.7152 * linear(pixels[index + 1] ?? 0) +
        0.0722 * linear(pixels[index + 2] ?? 0);

      if (luminance < DARK_LUMINANCE) dark += 1;
    }

    darkLogos.set(src, opaque > 0 && dark / opaque >= DARK_SHARE);
  } catch {
    darkLogos.set(src, false);
  }
}

export function isDarkLogo(src: string | null | undefined): boolean {
  return src ? darkLogos.get(src) === true : false;
}
