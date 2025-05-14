// See the Tailwind configuration guide for advanced usage
// https://tailwindcss.com/docs/configuration

let plugin = require('tailwindcss/plugin')

module.exports = {
  daisyui: {
    themes: [
      {
        afterdark: {
          "primary": "#7B79B5",
          "secondary": "#ACABD5",
          "accent": "#fef3c7",
          "neutral": "#38357F",
          "base-100": "#201D65",
          "info": "#7dd3fc",
          "success": "#a7f3d0",
          "warning": "#fef08a",
          "error": "#fca5a5",
        },
        her: {
          "primary": "#b57979",
          "secondary": "#d5abab",
          "accent": "#fef3c7",
          "neutral": "#7f3535",
          "base-100": "#651d1d",
          "info": "#7dd3fc",
          "success": "#a7f3d0",
          "warning": "#fef08a",
          "error": "#fca5a5",
        },
      },
      "lofi",
    ]
  },
  content: [
    './f/js/**/*.js',
    '../templates/**/*.html',
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require("daisyui"),
    require('@tailwindcss/forms'),
  ]
}
