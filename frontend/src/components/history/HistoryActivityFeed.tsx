// frontend/src/components/history/HistoryActivityFeed.tsx
// Global activity feed over the version history: the most recent changes
// across every vault (optionally filtered to a single item type).
// Self-contained — mount it anywhere, e.g. a dashboard or settings page.

import { useCallback, useEffect, useState } from 'react'
import { Bookmark, CheckSquare, Clipboard, Clock, FileText, Lock, RefreshCw, User } from 'lucide-react'
import {
    OPERATION_CLASSES,
    OPERATION_LABELS,
    TYPE_LABELS,
    formatTimestamp,
    recentActivity,
} from '../../lib/versioning'
import type { ActivityEntry } from '../../lib/versioning'

const TYPE_ICONS: Record<string, typeof FileText> = {
    note: FileText,
    clipboard: Clipboard,
    todo: CheckSquare,
    bookmark: Bookmark,
    contact: User,
    credential: Lock,
}

interface HistoryActivityFeedProps {
    limit?: number
    itemTypeFilter?: string
    className?: string
}

export default function HistoryActivityFeed({
    limit = 50,
    itemTypeFilter,
    className = '',
}: HistoryActivityFeedProps) {
    const [entries, setEntries] = useState<ActivityEntry[] | null>(null)
    const [error, setError] = useState<string | null>(null)
    const [loading, setLoading] = useState(false)

    const load = useCallback(async () => {
        setLoading(true)
        setError(null)
        try {
            setEntries(await recentActivity({ limit, itemType: itemTypeFilter }))
        } catch (err: unknown) {
            setError(err instanceof Error ? err.message : String(err))
        } finally {
            setLoading(false)
        }
    }, [limit, itemTypeFilter])

    useEffect(() => {
        void load()
    }, [load])

    return (
        <section
            className={`rounded-xl border border-gray-200 bg-white p-3 dark:border-gray-700 dark:bg-gray-800 ${className}`}
        >
            <div className="mb-2 flex items-center gap-2">
                <span className="flex h-7 w-7 items-center justify-center rounded-full bg-indigo-100 text-indigo-600 dark:bg-indigo-900/50 dark:text-indigo-300">
                    <Clock size={14} />
                </span>
                <h3 className="flex-1 text-sm font-semibold text-gray-900 dark:text-white">Recent activity</h3>
                <button
                    onClick={() => void load()}
                    title="Refresh"
                    className="rounded p-1 text-gray-400 hover:bg-gray-100 hover:text-gray-600 dark:hover:bg-gray-700 dark:hover:text-gray-300"
                >
                    <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
                </button>
            </div>
            {error && (
                <p className="rounded-lg bg-red-50 p-2 text-xs text-red-700 dark:bg-red-900/30 dark:text-red-300">
                    {error}
                </p>
            )}
            {!error && entries === null && (
                <p className="py-4 text-center text-xs text-gray-500 dark:text-gray-400">Loading activity…</p>
            )}
            {!error && entries !== null && entries.length === 0 && (
                <p className="py-4 text-center text-xs text-gray-500 dark:text-gray-400">
                    Nothing recorded yet — create or edit items to build history.
                </p>
            )}
            <ul className="space-y-1">
                {entries?.map((entry) => {
                    const Icon = TYPE_ICONS[entry.item_type] ?? FileText
                    return (
                        <li
                            key={entry.id}
                            className="flex items-start gap-2.5 rounded-lg p-2 hover:bg-gray-50 dark:hover:bg-gray-700/50"
                        >
                            <span className="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-gray-100 text-gray-500 dark:bg-gray-700 dark:text-gray-300">
                                <Icon size={12} />
                            </span>
                            <div className="min-w-0 flex-1">
                                <div className="flex flex-wrap items-baseline gap-1.5">
                                    <span dir="auto" className="truncate text-xs font-medium text-gray-900 dark:text-white">
                                        {entry.title || '(no title)'}
                                    </span>
                                    <span className="text-[10px] uppercase tracking-wide text-gray-400">
                                        {TYPE_LABELS[entry.item_type] ?? entry.item_type} · v{entry.version}
                                    </span>
                                </div>
                                <div className="mt-0.5 flex items-center gap-1.5">
                                    <span
                                        className={`rounded-full px-1.5 py-0.5 text-[10px] font-medium ${
                                            OPERATION_CLASSES[entry.operation] ?? ''
                                        }`}
                                    >
                                        {OPERATION_LABELS[entry.operation] ?? entry.operation}
                                    </span>
                                    <span className="text-[10px] text-gray-400">
                                        {formatTimestamp(entry.created_at)}
                                    </span>
                                </div>
                            </div>
                        </li>
                    )
                })}
            </ul>
        </section>
    )
}
