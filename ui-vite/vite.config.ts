import {defineConfig} from 'vite'
import react from '@vitejs/plugin-react'
import browserslistToEsbuild from "browserslist-to-esbuild"

// https://vitejs.dev/config/
export default defineConfig({
    plugins: [react()],
    target: browserslistToEsbuild(
        [
            ">0.2%",
            "not dead",
            "not op_mini all"
        ]
    ),
})
