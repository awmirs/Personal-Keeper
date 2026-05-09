import { useEffect, useState, useCallback } from 'react'
import api from '../../lib/api'
import type { Bookmark } from '../../types'
import { Plus, Trash2, Search, ExternalLink, Edit3 } from 'lucide-react'

export default function Bookmarks() {
  const [bookmarks, setBookmarks] = useState<Bookmark[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editForm, setEditForm] = useState({ url: '', title: '', description: '' })
  const [newUrl, setNewUrl] = useState('')
  const [newTitle, setNewTitle] = useState('')
  const [newDescription, setNewDescription] = useState('')

  const fetchBookmarks = useCallback(async () => {
    try {
      setLoading(true)
      const res = await api.get('/bookmarks')
      setBookmarks(res.data)
    } catch (err: any) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { fetchBookmarks() }, [fetchBookmarks])

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!newUrl.trim()) return
    try {
      await api.post('/bookmarks', { url: newUrl, title: newTitle, description: newDescription })
      setNewUrl(''); setNewTitle(''); setNewDescription(''); setShowCreate(false)
      fetchBookmarks()
    } catch (err: any) { alert('Failed to create: ' + err.message) }
  }

  const startEdit = (b: Bookmark) => {
    setEditingId(b.id)
    setEditForm({ url: b.url, title: b.title, description: b.description })
  }

  const cancelEdit = () => setEditingId(null)

  const handleUpdate = async (id: string) => {
    try {
      await api.put(`/bookmarks/${id}`, editForm)
      setEditingId(null)
      fetchBookmarks()
    } catch (err: any) { alert('Failed to update: ' + err.message) }
  }

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this bookmark?')) return
    try {
      await api.delete(`/bookmarks/${id}`)
      fetchBookmarks()
    } catch (err: any) { alert('Failed to delete: ' + err.message) }
  }

  const filtered = bookmarks.filter(b =>
      b.url.toLowerCase().includes(search.toLowerCase()) ||
      b.title.toLowerCase().includes(search.toLowerCase()) ||
      b.description.toLowerCase().includes(search.toLowerCase())
  )

  return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold dark:text-white">Bookmarks</h2>
          <button onClick={() => setShowCreate(!showCreate)} className="flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700">
            <Plus size={18} /> Add Bookmark
          </button>
        </div>

        <div className="relative mb-4">
          <Search className="absolute left-3 top-2.5 text-gray-400" size={18} />
          <input type="text" placeholder="Search bookmarks..." value={search} onChange={(e) => setSearch(e.target.value)} className="w-full rounded border pl-10 pr-4 py-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
        </div>

        {showCreate && (
            <form onSubmit={handleCreate} className="mb-6 rounded bg-white p-4 shadow dark:bg-gray-800">
              <input type="url" placeholder="URL (required)" value={newUrl} onChange={(e) => setNewUrl(e.target.value)} className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" required />
              <input type="text" placeholder="Title" value={newTitle} onChange={(e) => setNewTitle(e.target.value)} className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
              <textarea placeholder="Description" value={newDescription} onChange={(e) => setNewDescription(e.target.value)} rows={2} className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
              <div className="flex gap-2">
                <button type="submit" className="rounded bg-green-600 px-4 py-2 text-white hover:bg-green-700">Save</button>
                <button type="button" onClick={() => setShowCreate(false)} className="rounded bg-gray-300 px-4 py-2 dark:bg-gray-600 dark:text-white">Cancel</button>
              </div>
            </form>
        )}

        {loading && <p className="text-gray-500">Loading...</p>}
        {error && <p className="text-red-500">Error: {error}</p>}
        {!loading && !error && filtered.length === 0 && <p className="text-gray-500">No bookmarks found.</p>}

        <div className="space-y-3">
          {filtered.map(b => (
              <div key={b.id} className="rounded bg-white p-4 shadow dark:bg-gray-800 group flex items-start gap-3">
                {editingId === b.id ? (
                    <div className="flex-1 space-y-2">
                      <input type="url" value={editForm.url} onChange={e => setEditForm({...editForm, url: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                      <input type="text" value={editForm.title} onChange={e => setEditForm({...editForm, title: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                      <textarea value={editForm.description} onChange={e => setEditForm({...editForm, description: e.target.value})} rows={2} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                      <div className="flex gap-2">
                        <button onClick={() => handleUpdate(b.id)} className="rounded bg-green-600 px-3 py-1 text-white">Save</button>
                        <button onClick={cancelEdit} className="rounded bg-gray-300 px-3 py-1 dark:bg-gray-600 dark:text-white">Cancel</button>
                      </div>
                    </div>
                ) : (
                    <>
                      <div className="flex-1 min-w-0">
                        <a href={b.url} target="_blank" rel="noreferrer" className="font-semibold text-blue-600 dark:text-blue-400 hover:underline flex items-center gap-1">
                          {b.title || b.url} <ExternalLink size={14} />
                        </a>
                        {b.description && <p className="text-gray-600 dark:text-gray-400 text-sm mt-1">{b.description}</p>}
                        <p className="text-xs text-gray-400 mt-1">{new Date(b.updated_at * 1000).toLocaleString()}</p>
                      </div>
                      <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                        <button onClick={() => startEdit(b)} className="text-gray-400 hover:text-blue-500"><Edit3 size={18} /></button>
                        <button onClick={() => handleDelete(b.id)} className="text-gray-400 hover:text-red-500"><Trash2 size={18} /></button>
                      </div>
                    </>
                )}
              </div>
          ))}
        </div>
      </div>
  )
}