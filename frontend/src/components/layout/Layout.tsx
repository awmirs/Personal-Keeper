import { useState, useEffect } from 'react'
import { Outlet, Link, useNavigate } from 'react-router-dom'
import { useAuthStore } from '../../lib/auth'
import { Sun, Moon, Search, Menu, X } from 'lucide-react'

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

    // Mobile sidebar toggle
    const [sidebarOpen, setSidebarOpen] = useState(false)

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
    ]

    return (
        <div className="flex min-h-screen bg-gray-50 dark:bg-gray-900">
            {/* Sidebar */}
            <aside
                className={`
          fixed inset-y-0 left-0 z-50 w-64 transform bg-gray-800 text-white p-4
          transition-transform duration-200 ease-in-out
          lg:relative lg:translate-x-0
          ${sidebarOpen ? 'translate-x-0' : '-translate-x-full'}
        `}
            >
                <div className="flex items-center justify-between mb-6">
                    <h1 className="text-xl font-bold">Personal Keeper</h1>
                    <button onClick={() => setSidebarOpen(false)} className="lg:hidden">
                        <X size={20} />
                    </button>
                </div>

                <nav className="space-y-1">
                    {navLinks.map((link) => (
                        <Link
                            key={link.to}
                            to={link.to}
                            className="block py-2 px-3 rounded hover:bg-gray-700"
                            onClick={() => setSidebarOpen(false)}
                        >
                            {link.label}
                        </Link>
                    ))}
                </nav>

                <div className="absolute bottom-4 left-4 right-4 space-y-2">
                    <button
                        onClick={() => setDark(!dark)}
                        className="flex items-center gap-2 w-full py-2 px-3 rounded hover:bg-gray-700"
                    >
                        {dark ? <Sun size={18} /> : <Moon size={18} />}
                        {dark ? 'Light Mode' : 'Dark Mode'}
                    </button>
                    <button
                        onClick={handleLogout}
                        className="flex items-center gap-2 w-full py-2 px-3 rounded hover:bg-red-600 text-left"
                    >
                        Logout
                    </button>
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
            <div className="flex-1 flex flex-col min-w-0">
                <header className="bg-white dark:bg-gray-800 shadow p-4 flex items-center gap-4">
                    <button onClick={() => setSidebarOpen(true)} className="lg:hidden">
                        <Menu size={20} className="dark:text-white" />
                    </button>
                    <div className="relative flex-1 max-w-md">
                        <Search className="absolute left-3 top-2.5 text-gray-400" size={18} />
                        <input
                            type="text"
                            placeholder="Search all vaults..."
                            className="w-full rounded border pl-10 pr-4 py-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                            // Placeholder for future global search
                            onKeyDown={(e) => {
                                if (e.key === 'Enter') {
                                    window.location.href = `/?search=${encodeURIComponent((e.target as HTMLInputElement).value)}`
                                }
                            }}
                        />
                    </div>
                </header>

                <main className="flex-1 p-4 lg:p-6 dark:text-white">
                    <Outlet />
                </main>
            </div>
        </div>
    )
}