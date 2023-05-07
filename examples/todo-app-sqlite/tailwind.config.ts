import type { Config } from "tailwindcss";
import tailwindForms from "@tailwindcss/forms";

export default {
  content: ["./src/**/*.{rs,html}"],
  theme: {
    extend: {},
  },
  plugins: [tailwindForms],
} satisfies Config;
