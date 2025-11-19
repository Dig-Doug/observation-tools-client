/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./crates/observation-tools-server/templates/**/*.html"],
  darkMode: "media",
  theme: {
    extend: {
      colors: {
        primary: {
          light: "#000",
          dark: "#e0e0e0",
        },
      },
      fontFamily: {
        mono: ["monospace"],
      },
    },
  },
  plugins: [],
};
