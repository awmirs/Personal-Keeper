import { Sun, Moon } from 'lucide-react'
import { useThemeStore } from '../stores/themeStore'
import { themeNames } from '../lib/highlightThemes'
import type { CodeThemeOption } from '../stores/themeStore'

export default function Settings() {
    const { dark, toggleDark, codeTheme, setCodeTheme } = useThemeStore()

    return (
        <div>
            <h2 className="text-2xl font-bold dark:text-white mb-6">Settings</h2>
            <div className="max-w-xl space-y-6">
                <div className="rounded bg-white dark:bg-gray-800 shadow p-6">
                    <h3 className="text-lg font-semibold dark:text-white mb-4">Appearance</h3>
                    <button
                        onClick={toggleDark}
                        className="flex items-center gap-3 w-full py-2 px-4 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
                    >
                        {dark ? <Sun size={20} /> : <Moon size={20} />}
                        <span className="dark:text-white">{dark ? 'Light Mode' : 'Dark Mode'}</span>
                    </button>
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                        Toggle between light and dark theme.
                    </p>
                </div>

                <div className="rounded bg-white dark:bg-gray-800 shadow p-6">
                    <h3 className="text-lg font-semibold dark:text-white mb-4">Code Theme</h3>
                    <label className="block text-sm text-gray-500 dark:text-gray-400 mb-2">
                        Syntax highlighting theme for Markdown code blocks
                    </label>
                    <select
                        value={codeTheme}
                        onChange={(e) => setCodeTheme(e.target.value as CodeThemeOption)}
                        className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                    >
                        <option value="auto">Auto (follow app theme)</option>
                        {themeNames.map((name) => (
                            <option key={name} value={name}>
                                {name}
                            </option>
                        ))}
                    </select>
                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                        Choose a style for code blocks. “Auto” will switch between light and dark
                        based on the app theme.
                    </p>
                </div>
            </div>
        </div>
    )
}