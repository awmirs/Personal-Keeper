import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { VitePWA } from 'vite-plugin-pwa'

export default defineConfig({
    plugins: [
        react(),
        VitePWA({
            registerType: 'autoUpdate',
            includeAssets: ['favicon.ico', 'apple-touch-icon.png'],
            manifest: {
                name: 'Personal Keeper',
                short_name: 'PK',
                description: 'Offline-first personal knowledge base',
                theme_color: '#ffffff',
                icons: [
                    {
                        src: 'pwa-192x192.png',
                        sizes: '192x192',
                        type: 'image/png'
                    },
                    {
                        src: 'pwa-512x512.png',
                        sizes: '512x512',
                        type: 'image/png'
                    }
                ]
            },
            workbox: {
                globPatterns: ['**/*.{js,css,html,ico,png,svg,woff2}']
            }
        })
    ],
    server: {
        proxy: {
            '/auth': 'http://localhost:8080',
            '/notes': 'http://localhost:8080',
            '/clipboard': 'http://localhost:8080',
            '/todos': 'http://localhost:8080',
            '/bookmarks': 'http://localhost:8080',
            '/contacts': 'http://localhost:8080',
            '/health': 'http://localhost:8080'
        }
    }
})