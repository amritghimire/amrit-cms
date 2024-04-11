/** @type {import('tailwindcss').Config} */
module.exports = {
    mode: "all",
    content: ["./src/**/*.{rs,html,css}", "./index.html"],
    theme: {
        extend: {},
    },
    plugins: [
        require('@tailwindcss/forms'),
    ],
};
