import { useEffect, useState, useCallback } from 'react'
import api from '../../lib/api'
import type { ClipboardItem } from '../../types'
import { Plus, Trash2, Copy, Search, Check } from 'lucide-react'

export default function Clipboard() {
  const [items, setItems] = useState<ClipboardItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [newContent, setNewContent] = useState('')
  const [copiedId, setCopiedId] = useState<string | null>(null)

  const fetchItems = useCallback(async () => {
    try {
      setLoading(true)
      const res = await api.get('/clipboard')
      setItems(res.data)
    } catch (err: any) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchItems()
  }, [fetchItems])

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newContent.trim()) return
    try {
      await api.post('/clipboard', { content: newContent, persist_to_disk: true })
      setNewContent('')
      setShowCreate(false)
      fetchItems()
    } catch (err: any) {
      alert('Failed to create: ' + err.message)
    }
  }

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this snippet?')) return
    try {
      await api.delete(`/clipboard/${id}`)
      fetchItems()
    } catch (err: any) {
      alert('Failed to delete: ' + err.message)
    }
  }

  const copyToClipboard = async (text: string, id: string) => {
    try {
      await navigator.clipboard.writeText(text)
      setCopiedId(id)
      setTimeout(() => setCopiedId(null), 2000)
    } catch {
      // fallback for older browsers
      const textarea = document.createElement('textarea')
      textarea.value = text
      document.body.appendChild(textarea)
      textarea.select()
      document.execCommand('copy')
      document.body.removeChild(textarea)
      setCopiedId(id)
      setTimeout(() => setCopiedId(null), 2000)
    }
  }

  const filtered = items.filter((item) =>
      item.content.toLowerCase().includes(search.toLowerCase())
  )

  return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold dark:text-white">Clipboard</h2>
          <button
              onClick={() => setShowCreate(!showCreate)}
              className="flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
          >
            <Plus size={18} />
            New Snippet
          </button>
        </div>

        <div className="relative mb-4">
          <Search className="absolute left-3 top-2.5 text-gray-400" size={18} />
          <input
              type="text"
              placeholder="Search snippets..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="w-full rounded border pl-10 pr-4 py-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
          />
        </div>

        {showCreate && (
            <form onSubmit={handleCreate} className="mb-6 rounded bg-white p-4 shadow dark:bg-gray-800">
          <textarea
              placeholder="Paste your text here..."
              value={newContent}
              onChange={(e) => setNewContent(e.target.value)}
              rows={4}
              className="mb-3 w-full rounded border p-2 font-mono dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              required
          />
              <div className="flex gap-2">
                <button
                    type="submit"
                    className="rounded bg-green-600 px-4 py-2 text-white hover:bg-green-700"
                >
                  Save
                </button>
                <button
                    type="button"
                    onClick={() => setShowCreate(false)}
                    className="rounded bg-gray-300 px-4 py-2 dark:bg-gray-600 dark:text-white"
                >
                  Cancel
                </button>
              </div>
            </form>
        )}

        {loading && <p className="text-gray-500">Loading...</p>}
        {error && <p className="text-red-500">Error: {error}</p>}
        {!loading && !error && filtered.length === 0 && (
            <p className="text-gray-500">No snippets found.</p>
        )}

        <div className="space-y-3">
          {filtered.map((item) => (
              <div
                  key={item.id}
                  className="rounded bg-white p-4 shadow dark:bg-gray-800 group relative flex flex-col"
              >
                <div className="flex justify-between items-start mb-2">
              <pre className="whitespace-pre-wrap font-sans text-gray-700 dark:text-gray-300 flex-1">
                {item.content}
              </pre>
                  <div className="flex gap-2 ml-2">
                    <button
                        onClick={() => copyToClipboard(item.content, item.id)}
                        className="text-gray-400 hover:text-blue-500 transition"
                        title="Copy to clipboard"
                    >
                      {copiedId === item.id ? <Check size={18} className="text-green-500" /> : <Copy size={18} />}
                    </button>
                    <button
                        onClick={() => handleDelete(item.id)}
                        className="text-gray-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity"
                        title="Delete"
                    >
                      <Trash2 size={18} />
                    </button>
                  </div>
                </div>
                <p className="text-xs text-gray-400 mt-1">
                  {new Date(item.updated_at * 1000).toLocaleString()}
                </p>
              </div>
          ))}
        </div>
      </div>
  )
}