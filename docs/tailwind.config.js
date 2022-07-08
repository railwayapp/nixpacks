const fontStack = [
  "Inter",
  "-apple-system",
  "BlinkMacSystemFont",
  "Segoe UI",
  "Roboto",
  "Oxygen-Sans",
  "Ubuntu",
  "Cantarell",
  "Helvetica Neue",
  "sans-serif",
  "Apple Color Emoji",
  "Segoe UI Emoji",
  "Segoe UI Symbol",
].join(",");

const monoStack = [
  "ui-monospace",
  "SFMono-Regular",
  "SF Mono",
  "Consolas",
  "Liberation Mono",
  "Menlo",
  "monospace",
].join(",");

/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "jit",
  content: [
    "./pages/**/*.{js,ts,jsx,tsx}",
    "./components/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: fontStack,
        mono: monoStack,
      },
      colors: {
        fg: "var(--fg)",
        bg: "var(--bg)",
        primary: "var(--primary)",
      },
      typography: (theme) => ({
        DEFAULT: {
          css: {
            color: theme("colors.fg"),

            a: {
              color: theme("colors.fg"),

              "&:hover": {
                color: theme("colors.fuchsia.500"),
              },
            },

            "h1,h2,h3,h4": {
              color: theme("colors.fg"),
            },

            img: {
              borderRadius: theme("borderRadius.DEFAULT"),
            },

            code: {
              fontSize: theme("fontSize.sm"),
              color: theme("colors.fg"),
              backgroundColor: "hsl(230, 1%, 98%)",
              padding: `2px 4px`,
              borderRadius: theme("borderRadius.sm"),
              "&::before": { display: "none" },
              "&::after": { display: "none" },
            },

            pre: {
              background: "blue",
            },
          },
        },
      }),
    },
  },
  plugins: [require("@tailwindcss/typography")],
};
