/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "jit",
  content: [
    "./pages/**/*.{js,ts,jsx,tsx}",
    "./components/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
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
