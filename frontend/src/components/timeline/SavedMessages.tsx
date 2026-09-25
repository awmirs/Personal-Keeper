// frontend/src/components/timeline/SavedMessages.tsx
// Telegram-style "Saved Messages": every vault item (note, todo, clipboard
// snippet, bookmark, contact, credential) rendered as a chat message,
// ordered by creation time, with day separators, an in-page search sidebar,
// an edit dialog and a composer for capturing new items.
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { ChangeEvent, KeyboardEvent as ReactKeyboardEvent } from 'react'
import { ArrowDown, Bookmark, Loader2, Plus, Search, Send, X } from 'lucide-react'
import api from '../../lib/api'
import {
    KIND_API_BASES,
    KIND_LABELS,
    buildComposerPayload,
    detectKind,
    groupByDay,
    isActive,
    itemMatchesQuery,
    sortTimeline,
    toTimelineItem,
} from '../../lib/timeline'
import type { TimelineItem, VaultKind } from '../../lib/timeline'
import ChatBubble from './ChatBubble'
import EditItemModal from './EditItemModal'
import TimelineSearchSidebar from './TimelineSearchSidebar'

const COMPOSER_KINDS: VaultKind[] = ['note', 'clipboard', 'todo', 'bookmark']

function errorMessage(err: any, fallback: string): string {
    const data = err?.response?.data
    if (data && typeof data.error === 'string') return data.error
    if (data && typeof data.message === 'string') return data.message
    return typeof err?.message === 'string' ? err.message : fallback
}

export default function SavedMessages() {
    const [items, setItems] = useState<TimelineItem[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)

    // Search sidebar state
    const [searchOpen, setSearchOpen] = useState(false)
    const [query, setQuery] = useState('')
    const [kindFilter, setKindFilter] = useState<VaultKind | 'all'>('all')
    const [activeKey, setActiveKey] = useState<string | null>(null)
    const [flashKey, setFlashKey] = useState<string | null>(null)

    // Dialog state
    const [editing, setEditing] = useState<TimelineItem | null>(null)
    const [creating, setCreating] = useState<VaultKind | null>(null)
    const [saving, setSaving] = useState(false)
    const [modalError, setModalError] = useState<string | null>(null)
    const [confirmDelete, setConfirmDelete] = useState<TimelineItem | null>(null)

    // Composer state
    const [composerText, setComposerText] = useState('')
    const [composerKind, setComposerKind] = useState<'auto' | VaultKind>('auto')
    const [sending, setSending] = useState(false)

    const [toast, setToast] = useState<string | null>(null)
    const [showJump, setShowJump] = useState(false)

    const scrollRef = useRef<HTMLDivElement>(null)
    const bottomRef = useRef<HTMLDivElement>(null)
    const composerRef = useRef<HTMLTextAreaElement>(null)
    const itemRefs = useRef<Map<string, HTMLDivElement>>(new Map())
    const flashTimer = useRef<number | null>(null)
    const toastTimer = useRef<number | null>(null)
    const didInitialScroll = useRef(false)

    /* ------------------------- data loading ------------------------- */

    const loadAll = useCallback(async () => {
        setLoading(true)
        setError(null)
        const responses = await Promise.allSettled([
            api.get('/notes'),
            api.get('/clipboard'),
            api.get('/todos'),
            api.get('/bookmarks'),
            api.get('/contacts'),
            api.get('/credentials'),
        ])
        const kinds: VaultKind[] = ['note', 'clipboard', 'todo', 'bookmark', 'contact', 'credential']
        const collected: TimelineItem[] = []
        let reachable = false
        responses.forEach((response, index) => {
            if (response.status !== 'fulfilled') return
            const list = (response.value as any)?.data
            if (!Array.isArray(list)) return
            reachable = true
            for (const raw of list) {
                if (!raw || raw.id == null || !isActive(raw)) continue
                collected.push(toTimelineItem(kinds[index], raw))
            }
        })
        if (!reachable) {
            setError('Could not load your vaults. Check your connection and try again.')
        }
        setItems(sortTimeline(collected))
        setLoading(false)
    }, [])

    useEffect(() => {
        void loadAll()
    }, [loadAll])

    useEffect(() => {
        if (!loading && !didInitialScroll.current) {
            didInitialScroll.current = true
            bottomRef.current?.scrollIntoView({ block: 'end' })
        }
    }, [loading])

    useEffect(() => {
        return () => {
            itemRefs.current.clear()
            if (flashTimer.current) window.clearTimeout(flashTimer.current)
            if (toastTimer.current) window.clearTimeout(toastTimer.current)
        }
    }, [])

    /* ------------------------------ toast ---------------------------- */

    const showToast = useCallback((message: string) => {
        setToast(message)
        if (toastTimer.current) window.clearTimeout(toastTimer.current)
        toastTimer.current = window.setTimeout(() => setToast(null), 3200)
    }, [])

    /* ----------------------------- search ---------------------------- */

    const normalizedQuery = query.trim().toLowerCase()

    const matches = useMemo(
        () => (normalizedQuery ? items.filter((it) => itemMatchesQuery(it, normalizedQuery)) : []),
        [items, normalizedQuery]
    )

    const searchResults = useMemo(
        () => (kindFilter === 'all' ? matches : matches.filter((it) => it.kind === kindFilter)),
        [matches, kindFilter]
    )

    const activeIndex = useMemo(
        () => searchResults.findIndex((r) => r.key === activeKey),
        [searchResults, activeKey]
    )

    const flash = useCallback((key: string) => {
        setFlashKey(key)
        if (flashTimer.current) window.clearTimeout(flashTimer.current)
        flashTimer.current = window.setTimeout(() => setFlashKey(null), 2500)
    }, [])

    const navigateToItem = useCallback(
        (target: TimelineItem) => {
            setActiveKey(target.key)
            flash(target.key)
            itemRefs.current.get(target.key)?.scrollIntoView({ behavior: 'smooth', block: 'center' })
        },
        [flash]
    )

    const stepResult = useCallback(
        (delta: number) => {
            if (searchResults.length === 0) return
            const base = activeIndex === -1 ? (delta > 0 ? -1 : 0) : activeIndex
            const next = (base + delta + searchResults.length) % searchResults.length
            navigateToItem(searchResults[next])
        },
        [activeIndex, searchResults, navigateToItem]
    )

    /* --------------------------- mutations --------------------------- */

    const toggleTodo = useCallback(
        async (item: TimelineItem) => {
            if (item.kind !== 'todo') return
            const updated = { ...item.raw, completed: !item.raw?.completed }
            setItems((prev) => prev.map((it) => (it.key === item.key ? toTimelineItem('todo', updated) : it)))
            try {
                await api.put(`${KIND_API_BASES.todo}/${item.id}`, updated)
            } catch (err) {
                setItems((prev) => prev.map((it) => (it.key === item.key ? item : it)))
                showToast(errorMessage(err, 'Failed to update the todo'))
            }
        },
        [showToast]
    )

    const saveEdit = useCallback(
        async (payload: Record<string, unknown>) => {
            if (!editing) return
            setSaving(true)
            setModalError(null)
            try {
                const res = await api.put(`${KIND_API_BASES[editing.kind]}/${editing.id}`, { ...editing.raw, ...payload })
                const updated = toTimelineItem(editing.kind, res.data)
                setItems((prev) => prev.map((it) => (it.key === editing.key ? updated : it)))
                setEditing(null)
            } catch (err) {
                setModalError(errorMessage(err, 'Failed to save the changes'))
            } finally {
                setSaving(false)
            }
        },
        [editing]
    )

    const createItem = useCallback(
        async (payload: Record<string, unknown>) => {
            if (!creating) return
            setSaving(true)
            setModalError(null)
            try {
                const res = await api.post(KIND_API_BASES[creating], payload)
                const newItem = toTimelineItem(creating, res.data)
                setItems((prev) => sortTimeline([...prev, newItem]))
                setCreating(null)
                window.setTimeout(() => bottomRef.current?.scrollIntoView({ behavior: 'smooth' }), 200)
            } catch (err) {
                setModalError(errorMessage(err, 'Failed to create the item'))
            } finally {
                setSaving(false)
            }
        },
        [creating]
    )

    const deleteItem = useCallback(async () => {
        if (!confirmDelete) return
        const target = confirmDelete
        setConfirmDelete(null)
        setItems((prev) => prev.filter((it) => it.key !== target.key))
        try {
            await api.delete(`${KIND_API_BASES[target.kind]}/${target.id}`)
        } catch (err) {
            showToast(errorMessage(err, 'Failed to delete the item'))
            await loadAll()
        }
    }, [confirmDelete, loadAll, showToast])

    /* ---------------------------- composer --------------------------- */

    const send = useCallback(async () => {
        const text = composerText.trim()
        if (!text || sending) return
        const kind: VaultKind = composerKind === 'auto' ? detectKind(text) : composerKind
        setSending(true)
        try {
            const res = await api.post(KIND_API_BASES[kind], buildComposerPayload(kind, text))
            const newItem = toTimelineItem(kind, res.data)
            setItems((prev) => sortTimeline([...prev, newItem]))
            setComposerText('')
            if (composerRef.current) composerRef.current.style.height = 'auto'
            window.setTimeout(() => bottomRef.current?.scrollIntoView({ behavior: 'smooth' }), 200)
            showToast(`Saved to your ${KIND_LABELS[kind]} vault`)
        } catch (err) {
            showToast(errorMessage(err, 'Failed to save the message'))
        } finally {
            setSending(false)
        }
    }, [composerKind, composerText, sending, showToast])

    const handleComposerChange = (e: ChangeEvent<HTMLTextAreaElement>) => {
        setComposerText(e.target.value)
        e.target.style.height = 'auto'
        e.target.style.height = `${Math.min(e.target.scrollHeight, 160)}px`
    }

    const handleComposerKeyDown = (e: ReactKeyboardEvent<HTMLTextAreaElement>) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault()
            void send()
        }
    }

    /* --------------------------- keyboard ---------------------------- */

    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
                e.preventDefault()
                setSearchOpen((open) => !open)
                return
            }
            if (e.key === 'Escape' && searchOpen && !editing && !creating && !confirmDelete) {
                setSearchOpen(false)
            }
        }
        window.addEventListener('keydown', handler)
        return () => window.removeEventListener('keydown', handler)
    }, [searchOpen, editing, creating, confirmDelete])

    /* ----------------------------- render ---------------------------- */

    const groups = useMemo(() => groupByDay(items), [items])
    const showSpinner = loading && items.length === 0

    const handleScroll = () => {
        const el = scrollRef.current
        if (!el) return
        setShowJump(el.scrollHeight - el.scrollTop - el.clientHeight > 320)
    }

    return (
        <div className="flex h-full min-h-[70vh] flex-col overflow-hidden bg-gray-50 dark:bg-gray-900">
            {/* Chat header */}
            <header className="z-10 flex items-center gap-3 border-b border-gray-200 bg-white px-4 py-2.5 dark:border-gray-700 dark:bg-gray-800">
                <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-sky-400 to-blue-600 text-white shadow">
                    <Bookmark size={20} />
                </div>
                <div className="min-w-0 flex-1">
                    <h1 className="truncate text-base font-semibold dark:text-white">Saved Messages</h1>
                    <p className="truncate text-xs text-gray-500 dark:text-gray-400">
                        {showSpinner ? 'Loading…' : `${items.length} saved item${items.length === 1 ? '' : 's'}`}
                    </p>
                </div>
                <button
                    onClick={() => { setModalError(null); setCreating('note') }}
                    title="Create a new item"
                    className="rounded-lg p-2 text-gray-500 hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-200"
                >
                    <Plus size={19} />
                </button>
                <button
                    onClick={() => setSearchOpen((open) => !open)}
                    title="Search (Ctrl+K)"
                    className={`rounded-lg p-2 hover:bg-gray-100 dark:hover:bg-gray-700 ${
                        searchOpen
                            ? 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-200'
                            : 'text-gray-500 dark:text-gray-400'
                    }`}
                >
                    <Search size={19} />
                </button>
            </header>

            {/* Messages + search sidebar */}
            <div className="relative flex min-h-0 flex-1">
                <div className="relative flex min-h-0 min-w-0 flex-1 flex-col">
                    <div ref={scrollRef} onScroll={handleScroll} className="min-h-0 flex-1 overflow-y-auto overscroll-contain py-3">
                        <p className="mx-auto my-4 max-w-md rounded-2xl bg-black/5 px-4 py-2 text-center text-xs text-gray-500 dark:bg-white/5 dark:text-gray-400">
                            Everything you save in your vaults shows up here as a chat history — newest at the bottom.
                        </p>

                        {showSpinner && (
                            <div className="flex justify-center py-10">
                                <Loader2 size={28} className="animate-spin text-blue-500 dark:text-blue-400" />
                            </div>
                        )}

                        {error && (
                            <div className="mx-auto my-3 max-w-md rounded-lg bg-red-50 px-4 py-3 text-center text-sm text-red-600 dark:bg-red-900/30 dark:text-red-400">
                                {error}
                                <button onClick={() => void loadAll()} className="ml-2 underline underline-offset-2">
                                    Retry
                                </button>
                            </div>
                        )}

                        {!loading && !error && items.length === 0 && (
                            <div className="flex flex-col items-center justify-center py-16 text-center">
                                <div className="mb-3 flex h-14 w-14 items-center justify-center rounded-full bg-black/5 text-gray-400 dark:bg-white/10">
                                    <Bookmark size={26} />
                                </div>
                                <p className="text-sm font-semibold dark:text-white">No saved messages yet</p>
                                <p className="mt-1 max-w-xs text-xs text-gray-500 dark:text-gray-400">
                                    Write a message below — it will be stored in the matching vault and appear here instantly.
                                </p>
                            </div>
                        )}

                        {groups.map((group) => (
                            <div key={group.key}>
                                <DaySeparator label={group.label} />
                                {group.items.map((item) => (
                                    <div
                                        key={item.key}
                                        ref={(el) => {
                                            if (el) {
                                                itemRefs.current.set(item.key, el)
                                            } else {
                                                itemRefs.current.delete(item.key)
                                            }
                                        }}
                                    >
                                        <ChatBubble
                                            item={item}
                                            highlighted={flashKey === item.key}
                                            onEdit={() => { setModalError(null); setEditing(item) }}
                                            onDelete={() => setConfirmDelete(item)}
                                            onToggleTodo={() => void toggleTodo(item)}
                                        />
                                    </div>
                                ))}
                            </div>
                        ))}

                        <div ref={bottomRef} className="h-1" />
                    </div>

                    {showJump && !loading && (
                        <button
                            onClick={() => bottomRef.current?.scrollIntoView({ behavior: 'smooth' })}
                            title="Jump to the latest messages"
                            className="absolute bottom-4 left-1/2 z-10 -translate-x-1/2 rounded-full border border-gray-200 bg-white p-2 text-gray-600 shadow-lg hover:bg-gray-100 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600"
                        >
                            <ArrowDown size={18} />
                        </button>
                    )}
                </div>

                {searchOpen && (
                    <TimelineSearchSidebar
                        query={query}
                        onQueryChange={setQuery}
                        kindFilter={kindFilter}
                        onKindFilterChange={setKindFilter}
                        matches={matches}
                        results={searchResults}
                        activeIndex={activeIndex}
                        onNavigate={navigateToItem}
                        onPrev={() => stepResult(-1)}
                        onNext={() => stepResult(1)}
                        onClose={() => {
                            setSearchOpen(false)
                            setQuery('')
                            setKindFilter('all')
                            setActiveKey(null)
                        }}
                    />
                )}
            </div>

            {/* Composer */}
            <div className="border-t border-gray-200 bg-white px-3 py-2.5 dark:border-gray-700 dark:bg-gray-800">
                <div className="mx-auto max-w-3xl">
                    <div className="mb-2 flex flex-wrap items-center gap-1">
                        <ComposerChip
                            active={composerKind === 'auto'}
                            label="Auto"
                            title="Links are saved as bookmarks, everything else as notes"
                            onClick={() => setComposerKind('auto')}
                        />
                        {COMPOSER_KINDS.map((kind) => (
                            <ComposerChip
                                key={kind}
                                active={composerKind === kind}
                                label={KIND_LABELS[kind]}
                                onClick={() => setComposerKind(kind)}
                            />
                        ))}
                        <button
                            onClick={() => { setModalError(null); setCreating('contact') }}
                            className="ml-auto inline-flex items-center gap-1 rounded-full border border-gray-300 px-2.5 py-1 text-xs text-gray-600 hover:bg-gray-100 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-700"
                        >
                            <Plus size={12} />
                            Contact
                        </button>
                        <button
                            onClick={() => { setModalError(null); setCreating('credential') }}
                            className="inline-flex items-center gap-1 rounded-full border border-gray-300 px-2.5 py-1 text-xs text-gray-600 hover:bg-gray-100 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-700"
                        >
                            <Plus size={12} />
                            Credential
                        </button>
                    </div>
                    <div className="flex items-end gap-2">
                        <textarea
                            ref={composerRef}
                            rows={1}
                            dir="auto"
                            value={composerText}
                            onChange={handleComposerChange}
                            onKeyDown={handleComposerKeyDown}
                            placeholder="Write a message… (Enter to save, Shift+Enter for a new line)"
                            className="max-h-40 flex-1 resize-none rounded-2xl border border-gray-300 bg-gray-50 px-4 py-2.5 text-sm outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-900 dark:text-white"
                        />
                        <button
                            onClick={() => void send()}
                            disabled={!composerText.trim() || sending}
                            title="Save to your vault"
                            className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-blue-600 text-white shadow hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-40"
                        >
                            {sending ? <Loader2 size={18} className="animate-spin" /> : <Send size={18} />}
                        </button>
                    </div>
                </div>
            </div>

            {/* Dialogs */}
            {editing && (
                <EditItemModal
                    key={editing.key}
                    mode="edit"
                    kind={editing.kind}
                    item={editing}
                    saving={saving}
                    error={modalError}
                    onSave={(payload) => void saveEdit(payload)}
                    onClose={() => { setEditing(null); setModalError(null) }}
                />
            )}

            {creating && (
                <EditItemModal
                    key={`create:${creating}`}
                    mode="create"
                    kind={creating}
                    item={null}
                    saving={saving}
                    error={modalError}
                    onSave={(payload) => void createItem(payload)}
                    onClose={() => { setCreating(null); setModalError(null) }}
                />
            )}

            {confirmDelete && (
                <div
                    className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
                    onClick={() => setConfirmDelete(null)}
                >
                    <div className="w-full max-w-md rounded-lg bg-white shadow-xl dark:bg-gray-800" onClick={(e) => e.stopPropagation()}>
                        <div className="flex items-center justify-between border-b border-gray-200 p-4 dark:border-gray-700">
                            <h3 className="text-lg font-semibold dark:text-white">Delete message?</h3>
                            <button
                                onClick={() => setConfirmDelete(null)}
                                className="rounded p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                            >
                                <X size={20} />
                            </button>
                        </div>
                        <div className="p-6">
                            <p className="text-gray-700 dark:text-gray-300">
                                “{confirmDelete.title}” will be deleted from your {KIND_LABELS[confirmDelete.kind]} vault.
                            </p>
                        </div>
                        <div className="flex justify-end gap-3 border-t border-gray-200 p-4 dark:border-gray-700">
                            <button
                                onClick={() => setConfirmDelete(null)}
                                className="rounded border border-gray-300 bg-white px-4 py-2 text-gray-700 hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={() => void deleteItem()}
                                className="rounded bg-red-600 px-4 py-2 text-white hover:bg-red-700"
                            >
                                Delete
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {toast && (
                <div className="pointer-events-none fixed bottom-24 left-1/2 z-50 -translate-x-1/2 rounded-full bg-gray-900/90 px-4 py-2 text-sm text-white shadow-lg dark:bg-gray-100/95 dark:text-gray-900">
                    {toast}
                </div>
            )}
        </div>
    )
}

function DaySeparator({ label }: { label: string }) {
    return (
        <div className="pointer-events-none sticky top-0 z-[1] flex justify-center py-2">
            <span className="rounded-full bg-gray-500/15 px-3 py-1 text-xs font-medium text-gray-600 backdrop-blur-sm dark:bg-gray-400/20 dark:text-gray-300">
                {label}
            </span>
        </div>
    )
}

function ComposerChip({
    active,
    label,
    title,
    onClick,
}: {
    active: boolean
    label: string
    title?: string
    onClick: () => void
}) {
    return (
        <button
            onClick={onClick}
            title={title}
            className={[
                'rounded-full border px-2.5 py-1 text-xs font-medium transition-colors',
                active
                    ? 'border-blue-500 bg-blue-500 text-white'
                    : 'border-gray-300 text-gray-600 hover:bg-gray-100 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-700',
            ].join(' ')}
        >
            {label}
        </button>
    )
}
