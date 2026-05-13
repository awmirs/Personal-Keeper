import { useState, useEffect } from 'react'
import {Outlet, Link, useNavigate, useLocation} from 'react-router-dom'
import { useAuthStore } from '../../lib/auth'
import { Sun, Moon, Search, Menu, X } from 'lucide-react'
import GlobalSearch from '../GlobalSearch'

export default function Layout() {
    const navigate = useNavigate()
    const logout = useAuthStore((s) => s.logout)

    // Dark mode state
    const [dark, setDark] = useState(() => {
        if (typeof window !== 'undefined') {
            const stored = localStorage.getItem('theme')
            if (stored === 'dark') return true
            if (stored === 'light') return false
            return window.matchMedia('(prefers-color-scheme: dark)').matches
        }
        return false
    })

    // Apply dark class to <html> on mount and when toggled
    useEffect(() => {
        const root = document.documentElement
        if (dark) {
            root.classList.add('dark')
            localStorage.setItem('theme', 'dark')
        } else {
            root.classList.remove('dark')
            localStorage.setItem('theme', 'light')
        }
    }, [dark])

    useEffect(() => {
        const down = (e: KeyboardEvent) => {
            if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
                e.preventDefault()
                setGlobalSearchOpen((open) => !open)
            }
        }
        document.addEventListener('keydown', down)
        return () => document.removeEventListener('keydown', down)
    }, [])

    // Determine initial state based on screen size and localStorage
    const [sidebarOpen, setSidebarOpen] = useState(() => {
        const mq = window.matchMedia('(min-width: 1024px)')
        if (!mq.matches) return false // mobile always starts closed
        const stored = localStorage.getItem('desktopSidebarOpen')
        return stored === null ? true : stored === 'true' // desktop default open
    })

    const location = useLocation()

    const [mounted, setMounted] = useState(false)

    useEffect(() => {
        setMounted(true)
    }, [])

    // Persist desktop sidebar state to localStorage whenever it changes
    useEffect(() => {
        if (mounted && window.matchMedia('(min-width: 1024px)').matches) {
            localStorage.setItem('desktopSidebarOpen', sidebarOpen ? 'true' : 'false')
        }
    }, [sidebarOpen, mounted])

    const [globalSearchOpen, setGlobalSearchOpen] = useState(false)

    const handleLogout = () => {
        logout()
        navigate('/login')
    }

    const navLinks = [
        { to: '/', label: 'Notes' },
        { to: '/clipboard', label: 'Clipboard' },
        { to: '/todos', label: 'Todos' },
        { to: '/bookmarks', label: 'Bookmarks' },
        { to: '/contacts', label: 'Contacts' },
        { to: '/credentials', label: 'Credentials' },
    ]

    return (
        <div className="relative flex h-screen overflow-hidden bg-gray-50 dark:bg-gray-900">
            {/* Sidebar */}
            <aside
                className={`
          fixed inset-y-0 left-0 z-50 w-64 p-4
          bg-white dark:bg-gray-800 text-gray-900 dark:text-white
          border-r border-gray-200 dark:border-gray-700
          transform
          ${mounted ? 'transition-transform duration-300 ease-in-out' : ''}
          flex flex-col h-full overflow-y-auto
          ${sidebarOpen ? 'translate-x-0' : '-translate-x-full'}
        `}
            >
                <div className="h-full flex flex-col relative">
                    <div className="flex items-center justify-between mb-5 lg:mb-4">
                        <h1 className="text-xl font-bold flex-shrink-0">Personal Keeper</h1>
                        <button
                            onClick={() => setSidebarOpen(false)}
                            className="lg:hidden flex-shrink-0 p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
                            aria-label="Close sidebar"
                        >
                            <X size={20} className="text-gray-600 dark:text-white" />
                        </button>
                    </div>

                    <nav className="space-y-1">
                        {navLinks.map((link) => {
                            const isActive = link.to === '/'
                                ? location.pathname === '/'
                                : location.pathname.startsWith(link.to)
                            return (
                                <Link
                                    key={link.to}
                                    to={link.to}
                                    className={`block py-2 px-3 rounded hover:bg-gray-100 dark:hover:bg-gray-700 ${
                                        isActive ? 'bg-gray-200 dark:bg-gray-700 font-semibold' : ''
                                    }`}
                                    onClick={() => {
                                        if (!window.matchMedia('(min-width: 1024px)').matches) {
                                            setSidebarOpen(false)
                                        }
                                    }}
                                >
                                    {link.label}
                                </Link>
                            )
                        })}
                    </nav>

                    <div className="space-y-2 mt-auto">
                        <button
                            onClick={() => setDark(!dark)}
                            className="flex items-center gap-2 w-full py-2 px-3 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
                        >
                            {dark ? <Sun size={18} /> : <Moon size={18} />}
                            {dark ? 'Light Mode' : 'Dark Mode'}
                        </button>
                        <button
                            onClick={handleLogout}
                            className="flex items-center gap-2 w-full py-2 px-3 rounded text-left text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/50"
                        >
                            Logout
                        </button>
                    </div>
                </div>
            </aside>

            {/* Overlay for mobile */}
            {sidebarOpen && (
                <div
                    className="fixed inset-0 bg-black/50 z-40 lg:hidden"
                    onClick={() => setSidebarOpen(false)}
                />
            )}

            {/* Main content */}
            <div className={`flex-1 flex flex-col min-w-0 ${mounted ? 'transition-[margin-left] duration-300 ease-in-out' : ''} ${sidebarOpen ? 'lg:ml-64' : 'lg:ml-0'}`}>
                <header className="bg-white dark:bg-gray-800 shadow p-4 flex items-center gap-4">
                    {/* Toggle button – slides with content, always next to search box */}
                    <button
                        onClick={() => setSidebarOpen(prev => !prev)}
                        className={`flex-shrink-0 items-center justify-center w-8 h-8 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors ${
                            sidebarOpen ? 'lg:flex hidden' : 'flex'
                        }`}
                        aria-label="Toggle sidebar"
                    >
                        <Menu size={20} className="dark:text-white lg:text-gray-700 lg:dark:text-gray-200" />
                    </button>
                    <div className="relative flex-1 max-w-md">
                        <Search className="absolute left-3 top-2.5 text-gray-400" size={18} />
                        <input
                            type="text"
                            placeholder="Search all vaults... (Ctrl+K)"
                            readOnly
                            onClick={() => setGlobalSearchOpen(true)}
                            className="w-full rounded border pl-10 pr-4 py-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white cursor-pointer"
                            onKeyDown={(e) => {
                                if (e.key === 'Escape') setGlobalSearchOpen(false)
                            }}
                        />
                    </div>
                </header>

                <main className="flex-1 p-4 lg:p-6 dark:text-white overflow-y-auto">
                    <Outlet />
                </main>
            </div>
            <GlobalSearch isOpen={globalSearchOpen} onClose={() => setGlobalSearchOpen(false)} />
        </div>
    )
}