import { Outlet, Link, useNavigate } from 'react-router-dom'
import { useAuthStore } from '../../lib/auth'

export default function Layout() {
    const navigate = useNavigate()
    const logout = useAuthStore((s) => s.logout)

    const handleLogout = () => {
        logout()
        navigate('/login')
    }

    return (
        <div className="flex min-h-screen">
            <aside className="w-64 bg-gray-800 text-white p-4">
                <h1 className="text-xl font-bold mb-6">Personal Keeper</h1>
                <nav className="space-y-2">
                    <Link to="/" className="block py-2 px-3 rounded hover:bg-gray-700">Notes</Link>
                    <Link to="/clipboard" className="block py-2 px-3 rounded hover:bg-gray-700">Clipboard</Link>
                    <Link to="/todos" className="block py-2 px-3 rounded hover:bg-gray-700">Todos</Link>
                    <Link to="/bookmarks" className="block py-2 px-3 rounded hover:bg-gray-700">Bookmarks</Link>
                    <Link to="/contacts" className="block py-2 px-3 rounded hover:bg-gray-700">Contacts</Link>
                </nav>
                <button
                    onClick={handleLogout}
                    className="mt-auto block w-full text-left py-2 px-3 rounded hover:bg-red-600"
                >
                    Logout
                </button>
            </aside>
            <main className="flex-1 p-6">
                <Outlet />
            </main>
        </div>
    )
}
