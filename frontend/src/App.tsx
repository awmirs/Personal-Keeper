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
import {useEffect} from "react";
import api from "./lib/api.ts";

const ProtectedRoute = ({ children }: { children: JSX.Element }) => {
    const token = useAuthStore((s) => s.accessToken)
    return token ? children : <Navigate to="/login" replace />
}

export default function App() {

    const { accessToken, logout } = useAuthStore()

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