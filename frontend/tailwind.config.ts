import type { Config } from 'tailwindcss'
import typography = require('@tailwindcss/typography')

export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {},
  },
  plugins: [
    typography,
  ],
} satisfies Config