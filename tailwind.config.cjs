import { fontFamily } from 'tailwindcss/defaultTheme';

/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'selector',
  content: ['./templates/**/*.html'],
  theme: {
    extend: {
      backgroundImage: {
        egg: "url('/assets/egg-bg.svg')",
      },
      fontFamily: {
        sans: ['Inter var', ...fontFamily.sans],
      },
    },
  },
  plugins: [require('@tailwindcss/forms')],
};
