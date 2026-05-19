import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type CodeThemeOption = 'auto' | 'github' | 'github-dark' | 'monokai' | 'atom-one-light' | 'atom-one-dark'

interface ThemeState {
    dark: boolean
    codeTheme: CodeThemeOption
    toggleDark: () => void
    setCodeTheme: (theme: CodeThemeOption) => void
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
            codeTheme: (localStorage.getItem('theme-preferences') &&
                JSON.parse(localStorage.getItem('theme-preferences') || '{}').codeTheme) || 'auto' as CodeThemeOption,
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
            setCodeTheme: (theme) => set({ codeTheme: theme }),
        }),
        {
            name: 'theme-preferences',
            partialize: (state) => ({ dark: state.dark, codeTheme: state.codeTheme }),
        }
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