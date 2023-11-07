import {defineConfig} from 'vite'
import react from '@vitejs/plugin-react'
import browserslistToEsbuild from "browserslist-to-esbuild"

// https://vitejs.dev/config/
export default defineConfig({
    build: {
      outDir: "build",
    },
    plugins: [react()],
    target: browserslistToEsbuild(
        [
            ">0.2%",
            "not dead",
            "not op_mini all"
        ]
    ),
    server: {
        // this ensures that the browser opens upon server start
        open: true,
        // this sets a default port to 3000
        port: 3000,
    },
})
