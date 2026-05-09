import { create } from 'zustand'
import api from './api'

interface AuthState {
    accessToken: string | null
    refreshToken: string | null
    login: (username: string, password: string) => Promise<boolean>
    register: (username: string, password: string) => Promise<boolean>
    logout: () => void
}

export const useAuthStore = create<AuthState>((set) => ({
    accessToken: localStorage.getItem('accessToken'),
    refreshToken: localStorage.getItem('refreshToken'),
    login: async (username, password) => {
        try {
            const res = await api.post('/auth/login', { username, password })
            const { access_token, refresh_token } = res.data
            localStorage.setItem('accessToken', access_token)
            localStorage.setItem('refreshToken', refresh_token)
            set({ accessToken: access_token, refreshToken: refresh_token })
            return true
        } catch {
            return false
        }
    },
    register: async (username, password) => {
        try {
            const res = await api.post('/auth/register', { username, password })
            const { access_token, refresh_token } = res.data
            localStorage.setItem('accessToken', access_token)
            localStorage.setItem('refreshToken', refresh_token)
            set({ accessToken: access_token, refreshToken: refresh_token })
            return true
        } catch {
            return false
        }
    },
    logout: () => {
        localStorage.removeItem('accessToken')
        localStorage.removeItem('refreshToken')
        set({ accessToken: null, refreshToken: null })
    },
}))