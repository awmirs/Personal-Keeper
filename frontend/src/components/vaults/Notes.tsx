import { useEffect, useState, useCallback } from 'react'
import api, { reorderVault } from '../../lib/api'
import type { Note } from '../../types'
import ReactMarkdown from 'react-markdown'
import { markdownComponents } from '../../lib/markdown'
import remarkGfm from 'remark-gfm'
import rehypeHighlight from 'rehype-highlight'
import { Plus, Trash2, Search, Edit3, ChevronUp, ChevronDown } from 'lucide-react'
import LoadingSpinner from '../LoadingSpinner'
import {useConfirmation} from "../../context/ConfirmationContext.tsx";

export default function Notes() {
    const { confirm } = useConfirmation()
    const [notes, setNotes] = useState<Note[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)
    const [search, setSearch] = useState('')
    const [showCreate, setShowCreate] = useState(false)
    const [editingId, setEditingId] = useState<string | null>(null)
    const [editForm, setEditForm] = useState({ title: '', content: '' })
    const [newTitle, setNewTitle] = useState('')
    const [newContent, setNewContent] = useState('')
    const [editOrder, setEditOrder] = useState(false)

    const fetchNotes = useCallback(async () => {
        try {
            setLoading(true)
            const res = await api.get('/notes')
            setNotes(res.data)
        } catch (err: any) {
            setError(err.message)
        } finally {
            setLoading(false)
        }
    }, [])

    useEffect(() => {
        fetchNotes()
    }, [fetchNotes])

    const handleCreate = async (e: React.FormEvent) => {
        e.preventDefault()
        if (!newTitle.trim()) return
        try {
            await api.post('/notes', { title: newTitle, content: newContent })
            setNewTitle('')
            setNewContent('')
            setShowCreate(false)
            fetchNotes()
        } catch (err: any) {
            alert('Failed to create note: ' + err.message)
        }
    }

    const startEdit = (note: Note) => {
        setEditingId(note.id)
        setEditForm({ title: note.title, content: note.content })
    }

    const cancelEdit = () => setEditingId(null)

    const handleUpdate = async (id: string) => {
        try {
            // No PUT endpoint for notes yet – add it in backend quickly (1 line)
            await api.put(`/notes/${id}`, editForm)
            setEditingId(null)
            fetchNotes()
        } catch (err: any) {
            alert('Failed to update: ' + err.message)
        }
    }

    const handleDelete = async (id: string) => {
        const ok = await confirm('Delete this note?')
        if (!ok) return
        try {
            await api.delete(`/notes/${id}`)
            fetchNotes()
        } catch (err: any) {
            alert('Failed to delete note: ' + err.message)
        }
    }

    const moveNote = async (index: number, direction: 'up' | 'down') => {
        const newNotes = [...notes]
        const targetIndex = direction === 'up' ? index - 1 : index + 1
        if (targetIndex < 0 || targetIndex >= newNotes.length) return

        // Capture the two items before any mutation
        const itemA = newNotes[index]
        const itemB = newNotes[targetIndex]

            // Swap the items in the array
        ;[newNotes[index], newNotes[targetIndex]] = [newNotes[targetIndex], newNotes[index]]

        // Swap their positions
        const tempPos = itemA.position
        itemA.position = itemB.position
        itemB.position = tempPos

        // Sort the whole array by position so the visual order matches the new positions
        newNotes.sort((a, b) => a.position - b.position)

        // Optimistic update
        setNotes(newNotes)

        try {
            await reorderVault('notes', [
                { id: itemA.id, position: itemA.position },
                { id: itemB.id, position: itemB.position },
            ])
        } catch (err: any) {
            fetchNotes()
            alert('Failed to reorder: ' + (err.response?.data?.error || err.message))
        }
    }

    const filteredNotes = notes.filter(
        (note) =>
            note.title.toLowerCase().includes(search.toLowerCase()) ||
            note.content.toLowerCase().includes(search.toLowerCase())
    )

    return (
        <div>
            <div className="flex items-center justify-between mb-6">
                <h2 className="text-2xl font-bold dark:text-white">Notes</h2>
                <div className="flex gap-2">
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
                        New Note
                    </button>
                </div>
            </div>

            <div className="relative mb-4">
                <Search className="absolute left-3 top-2.5 text-gray-400" size={18} />
                <input
                    type="text"
                    placeholder="Search notes..."
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                    className="w-full rounded border pl-10 pr-4 py-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                />
            </div>

            {showCreate && (
                <form onSubmit={handleCreate} className="mb-6 rounded bg-white p-4 shadow dark:bg-gray-800">
                    <input
                        type="text"
                        placeholder="Title"
                        value={newTitle}
                        onChange={(e) => setNewTitle(e.target.value)}
                        className="mb-3 w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                        required
                    />
                    <textarea
                        placeholder="Content (Markdown supported)"
                        value={newContent}
                        onChange={(e) => setNewContent(e.target.value)}
                        rows={4}
                        className="mb-3 w-full rounded border p-2 font-mono dark:bg-gray-700 dark:border-gray-600 dark:text-white"
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

            {loading && <LoadingSpinner message="Loading notes..." />}
            {error && <p className="text-red-500">Error: {error}</p>}
            {!loading && !error && filteredNotes.length === 0 && (
                <p className="text-gray-500">No notes found.</p>
            )}

            <div className="space-y-4">
                {filteredNotes.map((note, index) => (
                    <div key={note.id} className="rounded bg-white p-4 shadow dark:bg-gray-800 group relative">
                        {editingId === note.id ? (
                            <div className="space-y-3">
                                <input
                                    type="text"
                                    value={editForm.title}
                                    onChange={(e) => setEditForm({ ...editForm, title: e.target.value })}
                                    className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                />
                                <textarea
                                    value={editForm.content}
                                    onChange={(e) => setEditForm({ ...editForm, content: e.target.value })}
                                    rows={5}
                                    className="w-full rounded border p-2 font-mono dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                />
                                <div className="flex gap-2">
                                    <button
                                        onClick={() => handleUpdate(note.id)}
                                        className="rounded bg-green-600 px-3 py-1 text-white"
                                    >
                                        Save
                                    </button>
                                    <button
                                        onClick={cancelEdit}
                                        className="rounded bg-gray-300 px-3 py-1 dark:bg-gray-600 dark:text-white"
                                    >
                                        Cancel
                                    </button>
                                </div>
                            </div>
                        ) : (
                            <>
                                <div className="flex justify-between items-start">
                                    <h3 className="text-lg font-semibold dark:text-white mb-2">{note.title}</h3>
                                    <div className="flex items-center gap-1">
                                        {editOrder && (
                                            <div className="flex flex-col gap-0.5">
                                                <button
                                                    onClick={() => moveNote(index, 'up')}
                                                    disabled={index === 0}
                                                    className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                                                    title="Move up"
                                                >
                                                    <ChevronUp size={18} />
                                                </button>
                                                <button
                                                    onClick={() => moveNote(index, 'down')}
                                                    disabled={index === filteredNotes.length - 1}
                                                    className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                                                    title="Move down"
                                                >
                                                    <ChevronDown size={18} />
                                                </button>
                                            </div>
                                        )}
                                        <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-2">
                                            <button onClick={() => startEdit(note)} className="text-gray-400 hover:text-blue-500 p-1">
                                                <Edit3 size={20} />
                                            </button>
                                            <button onClick={() => handleDelete(note.id)} className="text-gray-400 hover:text-red-500 p-1">
                                                <Trash2 size={20} />
                                            </button>
                                        </div>
                                    </div>
                                </div>
                                <hr className="my-1 border-gray-200 dark:border-gray-700" />
                                <div className="prose dark:prose-invert max-w-none text-gray-600 dark:text-gray-300">
                                    <ReactMarkdown
                                        remarkPlugins={[remarkGfm]}
                                        rehypePlugins={[rehypeHighlight]}
                                        components={markdownComponents}
                                    >
                                        {note.content}
                                    </ReactMarkdown>
                                </div>
                                <p className="text-xs text-gray-400 mt-2">
                                    {new Date(note.updated_at * 1000).toLocaleString()}
                                </p>
                            </>
                        )}
                    </div>
                ))}
            </div>
        </div>
    )
}