import { useState, useEffect, useRef } from 'react'
import { Search, X, FileText, Clipboard, CheckSquare, Bookmark, User } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import api from '../lib/api'

type SearchResult = {
    vault: string
    id: string
    title: string
    subtitle: string
    url?: string
}

export default function GlobalSearch({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
    const [query, setQuery] = useState('')
    const [results, setResults] = useState<SearchResult[]>([])
    const [loading, setLoading] = useState(false)
    const inputRef = useRef<HTMLInputElement>(null)
    const navigate = useNavigate()

    useEffect(() => {
        if (isOpen) {
            setTimeout(() => inputRef.current?.focus(), 100)
        } else {
            setQuery('')
            setResults([])
        }
    }, [isOpen])

    useEffect(() => {
        const trimmed = query.trim()
        if (!trimmed) {
            setResults([])
            return
        }

        const delay = setTimeout(async () => {
            setLoading(true)
            try {
                const res = await api.get('/search', { params: { q: trimmed } })
                setResults(res.data)
            } catch {
                setResults([])
            } finally {
                setLoading(false)
            }
        }, 250)

        return () => clearTimeout(delay)
    }, [query])

    const iconMap: Record<string, JSX.Element> = {
        Notes: <FileText size={18} />,
        Clipboard: <Clipboard size={18} />,
        Todos: <CheckSquare size={18} />,
        Bookmarks: <Bookmark size={18} />,
        Contacts: <User size={18} />,
    }

    if (!isOpen) return null

    return (
        <div className="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/50" onClick={onClose}>
            <div
                className="w-full max-w-2xl bg-white dark:bg-gray-800 rounded-lg shadow-2xl overflow-hidden"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center border-b dark:border-gray-700 px-4">
                    <Search size={20} className="text-gray-400 mr-2" />
                    <input
                        ref={inputRef}
                        type="text"
                        placeholder="Search across all vaults..."
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        className="w-full py-4 bg-transparent dark:text-white outline-none"
                    />
                    <button onClick={onClose} className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 ml-2">
                        <X size={20} />
                    </button>
                </div>

                <div className="max-h-[60vh] overflow-y-auto p-2">
                    {loading && <p className="text-center text-gray-500 p-4">Searching...</p>}
                    {!loading && query && results.length === 0 && (
                        <p className="text-center text-gray-500 p-4">No results found</p>
                    )}
                    {results.map((r) => (
                        <div
                            key={`${r.vault}-${r.id}`}
                            className="flex items-start gap-3 p-3 rounded hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer"
                            onClick={() => {
                                const vaultPath =
                                    r.vault === 'Notes' ? '/' :
                                        r.vault === 'Clipboard' ? '/clipboard' :
                                            r.vault === 'Todos' ? '/todos' :
                                                r.vault === 'Bookmarks' ? '/bookmarks' : '/contacts'
                                navigate(vaultPath)
                                onClose()
                            }}
                        >
                            <div className="text-gray-500 dark:text-gray-400 mt-1">{iconMap[r.vault] ?? <FileText size={18} />}</div>
                            <div className="flex-1 min-w-0">
                                <div className="flex items-center gap-2">
                                    <span className="text-xs font-medium uppercase text-gray-400">{r.vault}</span>
                                </div>
                                <div className="font-medium dark:text-white truncate">{r.title}</div>
                                {r.subtitle && <div className="text-sm text-gray-500 dark:text-gray-400 truncate">{r.subtitle}</div>}
                            </div>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    )
}
