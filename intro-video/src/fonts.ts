import { loadFont as loadInter } from "@remotion/google-fonts/Inter";
import { loadFont as loadJetBrainsMono } from "@remotion/google-fonts/JetBrainsMono";
import { loadFont as loadNotoSansTC } from "@remotion/google-fonts/NotoSansTC";

// Latin wordmark only ("dedent-paste")
export const inter = loadInter("normal", {
  weights: ["400", "500", "600", "700", "800"],
  subsets: ["latin"],
}).fontFamily;

// All Chinese copy
export const sans = loadNotoSansTC("normal", {
  weights: ["400", "500", "700", "900"],
  ignoreTooManyRequestsWarning: true,
}).fontFamily;

const jetBrainsMono = loadJetBrainsMono("normal", {
  weights: ["400", "500", "700"],
  subsets: ["latin"],
}).fontFamily;

// Monospace with CJK fallback for mixed code/Chinese lines
export const mono = `${jetBrainsMono}, ${sans}`;
