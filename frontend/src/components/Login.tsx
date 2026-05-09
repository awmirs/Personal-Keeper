import { useState } from 'react'
import { useAuthStore } from '../lib/auth'
import { useNavigate } from 'react-router-dom'

export default function Login() {
    const [username, setUsername] = useState('')
    const [password, setPassword] = useState('')
    const [isRegister, setIsRegister] = useState(false)
    const [error, setError] = useState('')
    const login = useAuthStore((s) => s.login)
    const register = useAuthStore((s) => s.register)
    const navigate = useNavigate()

    const validate = (): string | null => {
        if (!username.trim()) return 'Username is required'
        if (password.length < 8) return 'Password must be at least 8 characters'
        return null
    }

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        setError('')
        const validationError = validate()
        if (validationError) {
            setError(validationError)
            return
        }
        const success = isRegister ? await register(username, password) : await login(username, password)
        if (success) {
            navigate('/')
        } else {
            setError(isRegister ? 'Registration failed' : 'Invalid credentials')
        }
    }

    return (
        <div className="flex min-h-screen items-center justify-center bg-gray-100 dark:bg-gray-900">
            <form onSubmit={handleSubmit} className="w-80 rounded bg-white p-6 shadow dark:bg-gray-800">
                <h1 className="mb-4 text-xl font-bold dark:text-white">
                    {isRegister ? 'Register' : 'Login'}
                </h1>
                {error && <div className="mb-3 text-red-500 text-sm">{error}</div>}
                <input
                    className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    type="text"
                    placeholder="Username"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                />
                <input
                    className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    type="password"
                    placeholder="Password (min 8 chars)"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                />
                <button
                    type="submit"
                    className="w-full rounded bg-blue-600 py-2 text-white hover:bg-blue-700 disabled:opacity-50"
                    disabled={!username.trim() || password.length < 8}
                >
                    {isRegister ? 'Register' : 'Login'}
                </button>
                <p className="mt-2 text-center text-sm text-gray-600 dark:text-gray-400">
                    {isRegister ? 'Already have an account?' : "Don't have an account?"}{' '}
                    <button
                        type="button"
                        className="text-blue-500 underline"
                        onClick={() => setIsRegister(!isRegister)}
                    >
                        {isRegister ? 'Login' : 'Register'}
                    </button>
                </p>
            </form>
        </div>
    )
}