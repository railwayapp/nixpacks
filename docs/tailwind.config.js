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
    },
  },
  plugins: [require("@tailwindcss/typography")],
};
