import { useEffect, useState, useCallback } from 'react'
import api, { reorderVault } from '../../lib/api'
import type { ClipboardItem } from '../../types'
import { Plus, Trash2, Copy, Search, Check, ChevronUp, ChevronDown, X } from 'lucide-react'
import LoadingSpinner from '../LoadingSpinner'
import AutoDirText from "../AutoDirText.tsx";
import {useConfirmation} from "../../context/ConfirmationContext.tsx";
import { useViewStore } from '../../stores/viewStore'
import ViewSwitcher from '../ViewSwitcher'
import ListView from '../views/ListView'
import GridView from '../views/GridView'
import CompactListView from '../views/CompactListView'

export default function Clipboard() {
  const { confirm } = useConfirmation()
  const [items, setItems] = useState<ClipboardItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [showCreate, setShowCreate] = useState(false)
  const [newContent, setNewContent] = useState('')
  const [copiedId, setCopiedId] = useState<string | null>(null)
  const [editOrder, setEditOrder] = useState(false)
  const view = useViewStore((s) => s.views.clipboard || 'list')
  const [selectedItem, setSelectedItem] = useState<ClipboardItem | null>(null)

  const getPlainTextSnippet = (text: string, maxLen = 150) => {
    if (text.length <= maxLen) return text
    return text.slice(0, maxLen).trimEnd() + '…'
  }

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

  const moveItem = async (index: number, direction: 'up' | 'down') => {
    const newItems = [...items]
    const targetIndex = direction === 'up' ? index - 1 : index + 1
    if (targetIndex < 0 || targetIndex >= newItems.length) return

    const itemA = newItems[index]
    const itemB = newItems[targetIndex]

    ;[newItems[index], newItems[targetIndex]] = [newItems[targetIndex], newItems[index]]

    const tempPos = itemA.position
    itemA.position = itemB.position
    itemB.position = tempPos

    newItems.sort((a, b) => a.position - b.position)

    setItems(newItems)

    try {
      await reorderVault('clipboard', [
        { id: itemA.id, position: itemA.position },
        { id: itemB.id, position: itemB.position },
      ])
    } catch (err: any) {
      fetchItems()
      alert('Failed to reorder: ' + (err.response?.data?.error || err.message))
    }
  }

  const handleDelete = async (id: string) => {
    const ok = await confirm('Delete this snippet?')
    if (!ok) return
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

  const renderItem = (item: ClipboardItem, index: number) => (
      <div
          key={item.id}
          className={`rounded bg-white p-4 shadow dark:bg-gray-800 group relative flex flex-col ${
              view === 'grid'
                  ? 'h-64 overflow-hidden cursor-pointer'
                  : ''
          }`}
          onClick={() => view === 'grid' && setSelectedItem(item)}
      >
        <div className="flex justify-between items-start mb-2">
          {view === 'grid' ? (
              <p className="text-sm text-gray-600 dark:text-gray-300 whitespace-pre-wrap line-clamp-3 flex-1">
                {getPlainTextSnippet(item.content, 150)}
              </p>
          ) : (
              <AutoDirText
                  text={item.content}
                  className="whitespace-pre-wrap font-sans text-gray-700 dark:text-gray-300 flex-1"
                  as="div"
              />
          )}
          <div className="flex gap-2 ml-2">
            {editOrder && (
                <div className="flex flex-col gap-0.5">
                  <button
                      onClick={(e) => { e.stopPropagation(); moveItem(index, 'up'); }}
                      disabled={index === 0}
                      className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                      title="Move up"
                  >
                    <ChevronUp size={18} />
                  </button>
                  <button
                      onClick={(e) => { e.stopPropagation(); moveItem(index, 'down'); }}
                      disabled={index === filtered.length - 1}
                      className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                      title="Move down"
                  >
                    <ChevronDown size={18} />
                  </button>
                </div>
            )}
            <button
                onClick={(e) => { e.stopPropagation(); copyToClipboard(item.content, item.id); }}
                className="text-gray-400 hover:text-blue-500 transition p-1"
                title="Copy to clipboard"
            >
              {copiedId === item.id ? <Check size={20} className="text-green-500" /> : <Copy size={20} />}
            </button>
            <button
                onClick={(e) => { e.stopPropagation(); handleDelete(item.id); }}
                className="text-gray-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity p-1"
                title="Delete"
            >
              <Trash2 size={20} />
            </button>
          </div>
        </div>
        <p className="text-xs text-gray-400 mt-1">
          {new Date(item.updated_at * 1000).toLocaleString()}
        </p>
      </div>
  );

  return (
      <div>
        <div className="flex items-center justify-between mb-6">
            <h2 className="text-2xl font-bold dark:text-white">Clipboard</h2>
            <div className="flex gap-2 items-center">
              <ViewSwitcher vaultKey="clipboard" />
              <button
                  onClick={() => setEditOrder(!editOrder)}
                  className={`flex items-center gap-2 rounded px-4 py-2 ${
                      editOrder ? 'bg-green-600 hover:bg-green-700 text-white' : 'bg-gray-200 dark:bg-gray-700 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-600'
                  }`}
              >
                {editOrder ? 'Done' : 'Edit Order'}
              </button>
              <button
                  onClick={() => setShowCreate(!showCreate)}
                  className="flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
              >
                <Plus size={18} />
                New Snippet
              </button>
            </div>
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

        {loading && <LoadingSpinner message="Loading snippets..." />}
        {error && <p className="text-red-500">Error: {error}</p>}
        {!loading && !error && filtered.length === 0 && (
            <p className="text-gray-500">No snippets found.</p>
        )}

        {view === 'list' && <ListView items={filtered} renderItem={renderItem} />}
        {view === 'grid' && <GridView items={filtered} renderItem={renderItem} />}
        {view === 'compact' && <CompactListView items={filtered} renderItem={renderItem} />}


        {/* Detail modal */}
        {selectedItem && (
            <div
                className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4"
                onClick={() => setSelectedItem(null)}
            >
              <div
                  className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl max-h-[85vh] overflow-y-auto"
                  onClick={(e) => e.stopPropagation()}
              >
                <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                  <h3 className="text-xl font-bold dark:text-white">Clipboard Snippet</h3>
                  <button
                      onClick={() => setSelectedItem(null)}
                      className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1"
                  >
                    <X size={20} />
                  </button>
                </div>
                <div className="p-6">
                  <AutoDirText
                      text={selectedItem.content}
                      className="whitespace-pre-wrap font-sans text-gray-700 dark:text-gray-300"
                      as="div"
                  />
                </div>
                <div className="border-t border-gray-200 dark:border-gray-700 p-4 text-xs text-gray-400">
                  Last updated: {new Date(selectedItem.updated_at * 1000).toLocaleString()}
                </div>
              </div>
            </div>
        )}
      </div>
  )
}