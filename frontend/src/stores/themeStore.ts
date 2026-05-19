import { create } from 'zustand'
import { persist } from 'zustand/middleware'

interface ThemeState {
    dark: boolean
    toggleDark: () => void
}

function getInitialDark(): boolean {
    if (typeof window === 'undefined') return false
    const stored = localStorage.getItem('theme')
    if (stored === 'dark') return true
    if (stored === 'light') return false
    return window.matchMedia('(prefers-color-scheme: dark)').matches
}

export const useThemeStore = create<ThemeState>()(
    persist(
        (set) => ({
            dark: getInitialDark(),
            toggleDark: () =>
                set((state) => {
                    const next = !state.dark
                    const root = document.documentElement
                    if (next) {
                        root.classList.add('dark')
                        localStorage.setItem('theme', 'dark')
                    } else {
                        root.classList.remove('dark')
                        localStorage.setItem('theme', 'light')
                    }
                    return { dark: next }
                }),
        }),
        { name: 'theme-preferences' }
    )
)

// Ensure the HTML class stays in sync on app start
const initial = getInitialDark()
const root = document.documentElement
if (initial) {
    root.classList.add('dark')
    localStorage.setItem('theme', 'dark')
} else {
    root.classList.remove('dark')
    localStorage.setItem('theme', 'light')
}