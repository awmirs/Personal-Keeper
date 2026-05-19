import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from './lib/auth'
import Login from './components/Login'
import Layout from './components/layout/Layout'
import Notes from './components/vaults/Notes'
import Clipboard from './components/vaults/Clipboard'
import Todos from './components/vaults/Todos'
import Bookmarks from './components/vaults/Bookmarks'
import Contacts from './components/vaults/Contacts'
import Credentials from './components/vaults/Credentials'
import Settings from './components/Settings'
import { useEffect } from "react";
import api from "./lib/api.ts";
import { useThemeStore } from './stores/themeStore'
import { themeCSS, getEffectiveTheme } from './lib/highlightThemes'

const ProtectedRoute = ({ children }: { children: JSX.Element }) => {
    const token = useAuthStore((s) => s.accessToken)
    return token ? children : <Navigate to="/login" replace />
}

export default function App() {

    const { accessToken, logout } = useAuthStore()
    const dark = useThemeStore((s) => s.dark)
    const codeTheme = useThemeStore((s) => s.codeTheme)

    // Inject highlight.js theme CSS
    useEffect(() => {
        const effective = getEffectiveTheme(codeTheme, dark)
        const css = themeCSS[effective]
        if (!css) return

        let styleEl = document.getElementById('hljs-theme') as HTMLStyleElement | null
        if (!styleEl) {
            styleEl = document.createElement('style')
            styleEl.id = 'hljs-theme'
            document.head.appendChild(styleEl)
        }
        styleEl.textContent = css
    }, [dark, codeTheme])

    useEffect(() => {
        if (accessToken) {
            api.get('/auth/me').catch(() => logout())
        }
    }, [])

    return (
        <BrowserRouter>
            <Routes>
                <Route path="/login" element={<Login />} />
                <Route
                    path="/"
                    element={
                        <ProtectedRoute>
                            <Layout />
                        </ProtectedRoute>
                    }
                >
                    <Route index element={<Notes />} />
                    <Route path="clipboard" element={<Clipboard />} />
                    <Route path="todos" element={<Todos />} />
                    <Route path="bookmarks" element={<Bookmarks />} />
                    <Route path="contacts" element={<Contacts />} />
                    <Route path="credentials" element={<Credentials />} />
                    <Route path="settings" element={<Settings />} />
                </Route>
            </Routes>
        </BrowserRouter>
    )
}