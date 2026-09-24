// frontend/src/components/timeline/TimelineSearchSidebar.tsx
// Right-hand search panel: live full-text search across every vault item
// shown in the timeline, with kind filters, jump-to-result navigation and
// prev/next cycling (Telegram style).
import { useEffect, useMemo, useRef } from 'react'
import type { KeyboardEvent } from 'react'
import {
    Bookmark,
    CheckSquare,
    ChevronDown,
    ChevronUp,
    Clipboard,
    FileText,
    Lock,
    Search,
    User,
    X,
} from 'lucide-react'
import { KIND_ACCENTS, KIND_LABELS, VAULT_KINDS, formatDayLabel, formatTime, snippetFor } from '../../lib/timeline'
import type { TimelineItem, VaultKind } from '../../lib/timeline'

const KIND_ICONS: Record<VaultKind, typeof FileText> = {
    note: FileText,
    clipboard: Clipboard,
    todo: CheckSquare,
    bookmark: Bookmark,
    contact: User,
    credential: Lock,
}

interface TimelineSearchSidebarProps {
    query: string
    onQueryChange: (query: string) => void
    kindFilter: VaultKind | 'all'
    onKindFilterChange: (kind: VaultKind | 'all') => void
    matches: TimelineItem[]
    results: TimelineItem[]
    activeIndex: number
    onNavigate: (item: TimelineItem) => void
    onPrev: () => void
    onNext: () => void
    onClose: () => void
}

export default function TimelineSearchSidebar({
    query,
    onQueryChange,
    kindFilter,
    onKindFilterChange,
    matches,
    results,
    activeIndex,
    onNavigate,
    onPrev,
    onNext,
    onClose,
}: TimelineSearchSidebarProps) {
    const inputRef = useRef<HTMLInputElement>(null)

    useEffect(() => {
        inputRef.current?.focus()
    }, [])

    const counts = useMemo(() => {
        const map = new Map<VaultKind, number>()
        for (const match of matches) {
            map.set(match.kind, (map.get(match.kind) ?? 0) + 1)
        }
        return map
    }, [matches])

    const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter') {
            e.preventDefault()
            if (e.shiftKey) {
                onPrev()
            } else {
                onNext()
            }
        }
    }

    const trimmedQuery = query.trim()

    return (
        <aside className="flex h-full w-72 shrink-0 flex-col border-l border-gray-200 bg-white dark:border-gray-700 dark:bg-gray-800 sm:w-80">
            <div className="flex items-center gap-2 border-b border-gray-200 px-3 py-2 dark:border-gray-700">
                <Search size={15} className="shrink-0 text-gray-400" />
                <input
                    ref={inputRef}
                    type="text"
                    dir="auto"
                    value={query}
                    onChange={(e) => onQueryChange(e.target.value)}
                    onKeyDown={handleKeyDown}
                    placeholder="Search all saved items…"
                    className="min-w-0 flex-1 bg-transparent py-1 text-sm outline-none dark:text-white"
                />
                <button
                    onClick={onClose}
                    title="Close search"
                    className="rounded p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                >
                    <X size={16} />
                </button>
            </div>

            <div className="flex flex-wrap gap-1 border-b border-gray-200 px-3 py-2 dark:border-gray-700">
                <button onClick={() => onKindFilterChange('all')} className={filterChipClass(kindFilter === 'all')}>
                    All{matches.length > 0 ? ` (${matches.length})` : ''}
                </button>
                {VAULT_KINDS.map((kind) => (
                    <button
                        key={kind}
                        onClick={() => onKindFilterChange(kindFilter === kind ? 'all' : kind)}
                        className={filterChipClass(kindFilter === kind)}
                    >
                        {KIND_LABELS[kind]}
                        {counts.get(kind) ? ` (${counts.get(kind)})` : ''}
                    </button>
                ))}
            </div>

            <div className="flex items-center justify-between px-3 py-1.5 text-xs text-gray-500 dark:text-gray-400">
                <span>
                    {trimmedQuery ? `${results.length} result${results.length === 1 ? '' : 's'}` : 'Type to search'}
                </span>
                {results.length > 0 && (
                    <span className="flex items-center gap-1">
                        <span className="tabular-nums">
                            {activeIndex >= 0 ? activeIndex + 1 : '–'} / {results.length}
                        </span>
                        <button
                            onClick={onPrev}
                            title="Previous result (Shift+Enter)"
                            className="rounded p-1 hover:bg-gray-100 dark:hover:bg-gray-700"
                        >
                            <ChevronUp size={14} />
                        </button>
                        <button
                            onClick={onNext}
                            title="Next result (Enter)"
                            className="rounded p-1 hover:bg-gray-100 dark:hover:bg-gray-700"
                        >
                            <ChevronDown size={14} />
                        </button>
                    </span>
                )}
            </div>

            <div className="flex-1 overflow-y-auto p-2">
                {results.map((result, index) => {
                    const Icon = KIND_ICONS[result.kind]
                    return (
                        <button
                            key={result.key}
                            onClick={() => onNavigate(result)}
                            className={`mb-1 flex w-full items-start gap-2.5 rounded-lg p-2.5 text-left transition-colors ${
                                index === activeIndex
                                    ? 'bg-blue-50 dark:bg-blue-900/40'
                                    : 'hover:bg-gray-100 dark:hover:bg-gray-700'
                            }`}
                        >
                            <span className={`mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-full ${KIND_ACCENTS[result.kind]}`}>
                                <Icon size={14} />
                            </span>
                            <span className="min-w-0 flex-1">
                                <span className="flex items-baseline justify-between gap-2">
                                    <span dir="auto" className="truncate text-sm font-medium dark:text-white">
                                        <HighlightedText text={result.title} query={trimmedQuery} />
                                    </span>
                                    <span className="shrink-0 text-[10px] uppercase tracking-wide text-gray-400">
                                        {KIND_LABELS[result.kind]}
                                    </span>
                                </span>
                                <span dir="auto" className="mt-0.5 block truncate text-xs text-gray-500 dark:text-gray-400">
                                    <HighlightedText text={snippetFor(result, trimmedQuery.toLowerCase())} query={trimmedQuery} />
                                </span>
                                <span className="mt-0.5 block text-[11px] text-gray-400">
                                    {formatDayLabel(result.createdAt)} · {formatTime(result.createdAt)}
                                </span>
                            </span>
                        </button>
                    )
                })}
                {trimmedQuery && results.length === 0 && (
                    <p className="px-2 py-6 text-center text-sm text-gray-500 dark:text-gray-400">Nothing found</p>
                )}
            </div>
        </aside>
    )
}

function filterChipClass(active: boolean): string {
    return [
        'rounded-full border px-2 py-0.5 text-[11px] font-medium transition-colors',
        active
            ? 'border-blue-500 bg-blue-500 text-white'
            : 'border-gray-300 text-gray-600 hover:bg-gray-100 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-700',
    ].join(' ')
}

function HighlightedText({ text, query }: { text: string; query: string }) {
    if (!query) {
        return <>{text}</>
    }
    const index = text.toLowerCase().indexOf(query.toLowerCase())
    if (index === -1) {
        return <>{text}</>
    }
    return (
        <>
            {text.slice(0, index)}
            <mark className="rounded bg-yellow-200 px-0.5 text-inherit dark:bg-yellow-500/40">
                {text.slice(index, index + query.length)}
            </mark>
            {text.slice(index + query.length)}
        </>
    )
}
