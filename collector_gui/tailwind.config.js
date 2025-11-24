/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
  ],
  theme: {
    extend: {
      colors: {
        // Méthode 1: Couleurs simples
        // 'primary': '#3B82F6',
        // 'secondary': '#10B981',
        // 'danger': '#EF4444',
        
        // Méthode 2: Palette complète avec nuances
        'app': {
          50: '#f0f9ff',
          100: '#e0f2fe',
          200: '#bae6fd',
          300: '#7dd3fc',
          400: '#38bdf8',
          500: '#0e2482',  // Couleur principale
          600: '#0284c7',
          700: '#0369a1',
          800: '#075985',
          900: '#0c4a6e',
          950: '#082f49',
        },        
      },
    },
  },
  plugins: [],
}