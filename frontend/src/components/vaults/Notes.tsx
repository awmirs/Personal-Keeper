import { useEffect, useState, useCallback } from 'react'
import api, { reorderVault } from '../../lib/api'
import type { Note } from '../../types'
import ReactMarkdown from 'react-markdown'
import { markdownPlugins } from '../../lib/markdown'
import {Plus, Trash2, Search, Edit3, ChevronUp, ChevronDown, X} from 'lucide-react'
import { calculateFractionalPosition } from '../../lib/reorder'
import LoadingSpinner from '../LoadingSpinner'
import {useConfirmation} from "../../context/ConfirmationContext.tsx";
import ViewSwitcher from "../ViewSwitcher.tsx";
import {useViewStore} from "../../stores/viewStore.ts";
import ListView from "../views/ListView.tsx";
import GridView from "../views/GridView.tsx";
import CompactListView from "../views/CompactListView.tsx";

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
    const [selectedNote, setSelectedNote] = useState<Note | null>(null)
    const [editingInModal, setEditingInModal] = useState(false)
    const view = useViewStore((s) => s.views.notes || 'list')

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

    const startEditInModal = (note: Note) => {
        setSelectedNote(note)
        setEditForm({ title: note.title, content: note.content })
        setEditingInModal(true)
    }

    const cancelEdit = () => setEditingId(null)

    const handleUpdate = async (id: string) => {
        const previousNotes = notes
        setNotes((prev) =>
            prev.map((n) =>
                n.id === id
                    ? { ...n, title: editForm.title, content: editForm.content, updated_at: Math.floor(Date.now() / 1000) }
                    : n
            )
        )
        setEditingId(null)
        try {
            await api.put(`/notes/${id}`, editForm)
        } catch (err: any) {
            setNotes(previousNotes)
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
        const newPos = calculateFractionalPosition(filteredNotes, index, direction)
        if (newPos === null) return

        const targetItem = filteredNotes[index]
        const updatedNotes = notes.map((n) => (n.id === targetItem.id ? { ...n, position: newPos } : n))
        updatedNotes.sort((a, b) => a.position - b.position)
        setNotes(updatedNotes)

        try {
            await reorderVault('notes', [{ id: targetItem.id, position: newPos }])
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

    const renderNote = (note: Note, index: number) => (
        <div
            key={note.id}
            className={`rounded bg-white p-4 shadow dark:bg-gray-800 group relative ${
                view === 'grid'
                    ? 'h-64 overflow-hidden flex flex-col cursor-pointer'
                    : ''
            }`}
            onClick={() => {
                if (view === 'grid') setSelectedNote(note);
            }}
        >
            {editingId === note.id ? (
                // Inline editing for non-grid views
                <div className="space-y-3" onClick={(e) => e.stopPropagation()}>
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
                            type="button"
                            onClick={(e) => { e.stopPropagation(); handleUpdate(note.id); }}
                            className="rounded bg-green-600 px-3 py-1 text-white"
                        >
                            Save
                        </button>
                        <button
                            type="button"
                            onClick={(e) => { e.stopPropagation(); cancelEdit(); }}
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
                                        onClick={(e) => { e.stopPropagation(); moveNote(index, 'up'); }}
                                        disabled={index === 0}
                                        className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                                        title="Move up"
                                    >
                                        <ChevronUp size={18} />
                                    </button>
                                    <button
                                        onClick={(e) => { e.stopPropagation(); moveNote(index, 'down'); }}
                                        disabled={index === filteredNotes.length - 1}
                                        className="text-gray-400 hover:text-blue-500 disabled:opacity-30 p-0.5"
                                        title="Move down"
                                    >
                                        <ChevronDown size={18} />
                                    </button>
                                </div>
                            )}
                            <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-2">
                                <button
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        if (view === 'grid') {
                                            startEditInModal(note);
                                        } else {
                                            startEdit(note);
                                        }
                                    }}
                                    className="text-gray-400 hover:text-blue-500 p-1"
                                >
                                    <Edit3 size={20} />
                                </button>
                                <button onClick={(e) => { e.stopPropagation(); handleDelete(note.id); }} className="text-gray-400 hover:text-red-500 p-1">
                                    <Trash2 size={20} />
                                </button>
                            </div>
                        </div>
                    </div>
                    <hr className="my-1 border-gray-200 dark:border-gray-700" />
                    {view === 'grid' ? (
                        <div className="flex-1 overflow-hidden prose prose-sm dark:prose-invert max-w-none">
                            <ReactMarkdown {...markdownPlugins}>
                                {note.content}
                            </ReactMarkdown>
                        </div>
                    ) : (
                        <div className="prose dark:prose-invert max-w-none text-gray-600 dark:text-gray-300">
                            <ReactMarkdown {...markdownPlugins}>
                                {note.content}
                            </ReactMarkdown>
                        </div>
                    )}
                    <p className="text-xs text-gray-400 mt-2">
                        {new Date(note.updated_at * 1000).toLocaleString()}
                    </p>
                </>
            )}
        </div>
    );

    return (
        <div>
            <div className="flex items-center justify-between mb-6">
                <h2 className="text-2xl font-bold dark:text-white">Notes</h2>
                <div className="flex gap-2 items-center">
                    <ViewSwitcher vaultKey="notes" />
                    <button
                        type="button"
                        onClick={() => setEditOrder(!editOrder)}
                        className={`flex items-center gap-2 rounded px-4 py-2 ${
                            editOrder ? 'bg-green-600 hover:bg-green-700 text-white' : 'bg-gray-200 dark:bg-gray-700 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-600'
                        }`}
                    >
                        {editOrder ? 'Done' : 'Edit Order'}
                    </button>
                    <button
                        type="button"
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

            {view === 'list' && <ListView items={filteredNotes} renderItem={renderNote} />}
            {view === 'grid' && <GridView items={filteredNotes} renderItem={renderNote} />}
            {view === 'compact' && <CompactListView items={filteredNotes} renderItem={renderNote} />}


            {/* Detail modal */}
            {selectedNote && (
                <div
                    className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4"
                    onClick={() => { setSelectedNote(null); setEditingInModal(false); }}
                >
                    <div
                        className="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl max-h-[85vh] overflow-y-auto"
                        onClick={(e) => e.stopPropagation()}
                    >
                        {editingInModal ? (
                            <>
                                <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                                    <h3 className="text-xl font-bold dark:text-white">Edit Note</h3>
                                    <button
                                        type="button"
                                        onClick={() => { setEditingInModal(false); setSelectedNote(null); }}
                                        className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1"
                                    >
                                        <X size={20} />
                                    </button>
                                </div>
                                <div className="p-6 space-y-3">
                                    <input
                                        type="text"
                                        value={editForm.title}
                                        onChange={(e) => setEditForm({ ...editForm, title: e.target.value })}
                                        className="w-full rounded border p-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                    />
                                    <textarea
                                        value={editForm.content}
                                        onChange={(e) => setEditForm({ ...editForm, content: e.target.value })}
                                        rows={12}
                                        className="w-full rounded border p-2 font-mono dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                    />
                                    <div className="flex gap-2">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                handleUpdate(selectedNote.id);
                                                setEditingInModal(false);
                                                setSelectedNote(null); // optionally close modal after save
                                            }}
                                            className="rounded bg-green-600 px-4 py-2 text-white"
                                        >
                                            Save
                                        </button>
                                        <button
                                            type="button"
                                            onClick={() => { setSelectedNote(null); setEditingInModal(false); }}
                                            className="rounded bg-gray-300 px-4 py-2 dark:bg-gray-600 dark:text-white"
                                        >
                                            Cancel
                                        </button>
                                    </div>
                                </div>
                            </>
                        ) : (
                            <>
                                <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
                                    <h3 className="text-xl font-bold dark:text-white">{selectedNote.title}</h3>
                                    <button
                                        type="button"
                                        onClick={() => setSelectedNote(null)}
                                        className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1"
                                    >
                                        <X size={20} />
                                    </button>
                                </div>
                                <div className="p-6 prose dark:prose-invert max-w-none">
                                    <ReactMarkdown {...markdownPlugins}>
                                        {selectedNote.content}
                                    </ReactMarkdown>
                                </div>
                                <div className="border-t border-gray-200 dark:border-gray-700 p-4 text-xs text-gray-400">
                                    Last updated: {new Date(selectedNote.updated_at * 1000).toLocaleString()}
                                </div>
                            </>
                        )}
                    </div>
                </div>
            )}
        </div>
    )
}