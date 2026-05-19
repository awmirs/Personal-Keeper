import { useEffect, useState, useCallback } from 'react'
import api, { reorderVault } from '../../lib/api'
import type { Bookmark } from '../../types'
import { Plus, Trash2, Search, ExternalLink, Edit3, ChevronUp, ChevronDown, X } from 'lucide-react'
import AutoDirText from "../AutoDirText.tsx";
import LoadingSpinner from "../LoadingSpinner.tsx";
import { useConfirmation } from '../../context/ConfirmationContext';
import { useViewStore } from '../../stores/viewStore'
import ViewSwitcher from '../ViewSwitcher'
import ListView from '../views/ListView'
import GridView from '../views/GridView'
import CompactListView from '../views/CompactListView'

export default function Bookmarks() {
  const { confirm } = useConfirmation()
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
  const [editOrder, setEditOrder] = useState(false)
  const view = useViewStore((s) => s.views.bookmarks || 'list')
  const [selectedBookmark, setSelectedBookmark] = useState<Bookmark | null>(null)
  const [editingInModal, setEditingInModal] = useState(false)

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

  const startEditInModal = (b: Bookmark) => {
    setSelectedBookmark(b)
    setEditForm({ url: b.url, title: b.title, description: b.description })
    setEditingInModal(true)
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
    const ok = await confirm('Delete this bookmark?')
    if (!ok) return
    try {
      await api.delete(`/bookmarks/${id}`)
      fetchBookmarks()
    } catch (err: any) { alert('Failed to delete: ' + err.message) }
  }

  const moveBookmark = async (index: number, direction: 'up' | 'down') => {
    const newBookmarks = [...bookmarks]
    const targetIndex = direction === 'up' ? index - 1 : index + 1
    if (targetIndex < 0 || targetIndex >= newBookmarks.length) return

    const itemA = newBookmarks[index]
    const itemB = newBookmarks[targetIndex]

    ;[newBookmarks[index], newBookmarks[targetIndex]] = [newBookmarks[targetIndex], newBookmarks[index]]

    const tempPos = itemA.position
    itemA.position = itemB.position
    itemB.position = tempPos

    newBookmarks.sort((a, b) => a.position - b.position)

    setBookmarks(newBookmarks)

    try {
      await reorderVault('bookmarks', [
        { id: itemA.id, position: itemA.position },
        { id: itemB.id, position: itemB.position },
      ])
    } catch (err: any) {
      fetchBookmarks()
      alert('Failed to reorder: ' + (err.response?.data?.error || err.message))
    }
  }

  const filtered = bookmarks.filter(b =>
      b.url.toLowerCase().includes(search.toLowerCase()) ||
      b.title.toLowerCase().includes(search.toLowerCase()) ||
      b.description.toLowerCase().includes(search.toLowerCase())
  )

  const getPlainTextSnippet = (text: string, maxLen = 120) => {
    if (!text) return ''
    if (text.length <= maxLen) return text
    return text.slice(0, maxLen).trimEnd() + '…'
  }

  const renderBookmark = (b: Bookmark, index: number) => (
      <div
          key={b.id}
          className={`rounded bg-white p-4 shadow dark:bg-gray-800 group flex ${
              view === 'grid'
                  ? 'flex-col h-48 overflow-hidden cursor-pointer'
                  : 'items-start gap-3'
          }`}
          onClick={() => view === 'grid' && setSelectedBookmark(b)}
      >
        {editingId === b.id ? (
            <div className="flex-1 space-y-2" onClick={(e) => e.stopPropagation()}>
              <input type="url" value={editForm.url} onChange={e => setEditForm({...editForm, url: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
              <input type="text" value={editForm.title} onChange={e => setEditForm({...editForm, title: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
              <textarea value={editForm.description} onChange={e => setEditForm({...editForm, description: e.target.value})} rows={2} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
              <div className="flex gap-2">
                <button onClick={(e) => { e.stopPropagation(); handleUpdate(b.id); }} className="rounded bg-green-600 px-3 py-1 text-white">Save</button>
                <button onClick={(e) => { e.stopPropagation(); cancelEdit(); }} className="rounded bg-gray-300 px-3 py-1 dark:bg-gray-600 dark:text-white">Cancel</button>
              </div>
            </div>
        ) : (
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between">
                <a href={b.url} target="_blank" rel="noreferrer" className="font-semibold text-blue-600 dark:text-blue-400 hover:underline flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
                  {b.title || b.url} <ExternalLink size={16} />
                </a>
                <div className="flex items-center gap-1">
                  {editOrder && (
                      <div className="flex flex-col gap-0.5">
                        <button
                            onClick={(e) => { e.stopPropagation(); moveBookmark(index, 'up'); }}
                            disabled={index === 0}
                            className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                            title="Move up"
                        >
                          <ChevronUp size={18} />
                        </button>
                        <button
                            onClick={(e) => { e.stopPropagation(); moveBookmark(index, 'down'); }}
                            disabled={index === filtered.length - 1}
                            className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                            title="Move down"
                        >
                          <ChevronDown size={18} />
                        </button>
                      </div>
                  )}
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button onClick={(e) => {
                      e.stopPropagation();
                      if (view === 'grid') {
                        startEditInModal(b);
                      } else {
                        startEdit(b);
                      }
                    }} className="text-gray-400 hover:text-blue-500 p-1"><Edit3 size={20} /></button>
                    <button onClick={(e) => { e.stopPropagation(); handleDelete(b.id); }} className="text-gray-400 hover:text-red-500 p-1"><Trash2 size={20} /></button>
                  </div>
                </div>
              </div>
              <hr className="my-2 border-gray-200 dark:border-gray-700" />
              {view === 'grid' ? (
                  <p className="text-sm text-gray-600 dark:text-gray-400 line-clamp-2">
                    {getPlainTextSnippet(b.description, 100)}
                  </p>
              ) : (
                  b.description && <AutoDirText text={b.description} as="p" className="text-gray-600 dark:text-gray-400 text-sm mt-1" />
              )}
              <p className="text-xs text-gray-400 mt-1">{new Date(b.updated_at * 1000).toLocaleString()}</p>
            </div>
        )}
      </div>
  );

  return (
      <div>
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold dark:text-white">Bookmarks</h2>
          <div className="flex gap-2 items-center">
            <ViewSwitcher vaultKey="bookmarks" />
            <button
                onClick={() => setEditOrder(!editOrder)}
                className={`flex items-center gap-2 rounded px-4 py-2 ${
                    editOrder ? 'bg-green-600 hover:bg-green-700 text-white' : 'bg-gray-200 dark:bg-gray-700 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-600'
                }`}
            >
              {editOrder ? 'Done' : 'Edit Order'}
            </button>
            <button onClick={() => setShowCreate(!showCreate)} className="flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700">
              <Plus size={18} /> Add Bookmark
            </button>
          </div>
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

        {loading && <LoadingSpinner message="Loading bookmarks..." />}
        {error && <p className="text-red-500">Error: {error}</p>}
        {!loading && !error && filtered.length === 0 && <p className="text-gray-500">No bookmarks found.</p>}

        {view === 'list' && <ListView items={filtered} renderItem={renderBookmark} />}
        {view === 'grid' && <GridView items={filtered} renderItem={renderBookmark} />}
        {view === 'compact' && <CompactListView items={filtered} renderItem={renderBookmark} />}


        {/* Detail modal */}
        {selectedBookmark && (
            <div
                className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4"
                onClick={() => { setSelectedBookmark(null); setEditingInModal(false); }}
            >
              <div
                  className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl max-h-[85vh] overflow-y-auto"
                  onClick={(e) => e.stopPropagation()}
              >
                {editingInModal ? (
                    <>
                      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                        <h3 className="text-xl font-bold dark:text-white">Edit Bookmark</h3>
                        <button onClick={() => setEditingInModal(false)} className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1"><X size={20} /></button>
                      </div>
                      <div className="p-6 space-y-3">
                        <input type="url" value={editForm.url} onChange={e => setEditForm({...editForm, url: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                        <input type="text" value={editForm.title} onChange={e => setEditForm({...editForm, title: e.target.value})} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                        <textarea value={editForm.description} onChange={e => setEditForm({...editForm, description: e.target.value})} rows={6} className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white" />
                        <div className="flex gap-2">
                          <button onClick={() => { handleUpdate(selectedBookmark.id); setEditingInModal(false); setSelectedBookmark(null); }} className="rounded bg-green-600 px-4 py-2 text-white">Save</button>
                          <button onClick={() => { setSelectedBookmark(null); setEditingInModal(false); }} className="rounded bg-gray-300 px-4 py-2 dark:bg-gray-600 dark:text-white">Cancel</button>
                        </div>
                      </div>
                    </>
                ) : (
                    <>
                      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                        <h3 className="text-xl font-bold dark:text-white">{selectedBookmark.title || selectedBookmark.url}</h3>
                        <button onClick={() => setSelectedBookmark(null)} className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1"><X size={20} /></button>
                      </div>
                      <div className="p-6 space-y-3">
                        <div>
                          <label className="text-xs text-gray-500 uppercase">URL</label>
                          <a href={selectedBookmark.url} target="_blank" rel="noreferrer" className="text-blue-500 hover:underline block">{selectedBookmark.url}</a>
                        </div>
                        {selectedBookmark.description && (
                            <div>
                              <label className="text-xs text-gray-500 uppercase">Description</label>
                              <AutoDirText text={selectedBookmark.description} as="p" className="dark:text-white whitespace-pre-wrap" />
                            </div>
                        )}
                      </div>
                      <div className="border-t border-gray-200 dark:border-gray-700 p-4 text-xs text-gray-400">
                        Last updated: {new Date(selectedBookmark.updated_at * 1000).toLocaleString()}
                      </div>
                    </>
                )}
              </div>
            </div>
        )}
      </div>
  )
}