// frontend/src/components/timeline/ChatBubble.tsx
// A single vault item rendered as a Telegram-style outgoing message bubble.
import { useState } from 'react'
import {
    Bookmark,
    Check,
    CheckSquare,
    Clipboard,
    Copy,
    ExternalLink,
    FileText,
    Lock,
    Mail,
    Pencil,
    Phone,
    Trash2,
    User,
} from 'lucide-react'
import AutoDirText from '../AutoDirText'
import { KIND_ACCENTS, KIND_LABELS, formatDayLabel, formatTime } from '../../lib/timeline'
import type { TimelineItem, VaultKind } from '../../lib/timeline'

const KIND_ICONS: Record<VaultKind, typeof FileText> = {
    note: FileText,
    clipboard: Clipboard,
    todo: CheckSquare,
    bookmark: Bookmark,
    contact: User,
    credential: Lock,
}

interface ChatBubbleProps {
    item: TimelineItem
    highlighted: boolean
    onEdit: () => void
    onDelete: () => void
    onToggleTodo: () => void
}

export default function ChatBubble({ item, highlighted, onEdit, onDelete, onToggleTodo }: ChatBubbleProps) {
    const [copied, setCopied] = useState(false)
    const Icon = KIND_ICONS[item.kind]

    const copyText = async () => {
        const text = item.kind === 'clipboard' ? String(item.raw?.content ?? '') : String(item.raw?.url ?? '')
        if (!text) return
        try {
            await navigator.clipboard.writeText(text)
            setCopied(true)
            setTimeout(() => setCopied(false), 1600)
        } catch {
            /* clipboard unavailable — ignore */
        }
    }

    return (
        <div className="group flex justify-end px-2 py-0.5 sm:px-4">
            <div
                className={[
                    'relative max-w-[78%] rounded-2xl rounded-br-md px-3.5 py-2.5 shadow-sm',
                    'bg-emerald-100/90 dark:bg-emerald-900/50 dark:shadow-none transition-shadow',
                    highlighted ? 'ring-2 ring-sky-400 dark:ring-sky-500 animate-pulse' : '',
                ].join(' ')}
            >
                <div className="mb-1 flex flex-wrap items-center gap-1.5">
                    <span
                        className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium ${KIND_ACCENTS[item.kind]}`}
                    >
                        <Icon size={12} />
                        {KIND_LABELS[item.kind]}
                    </span>
                    {item.isFavorite && (
                        <span className="text-[11px] text-amber-500" title="Favorite">★</span>
                    )}
                    {item.tags.slice(0, 4).map((tag) => (
                        <span
                            key={tag}
                            className="rounded-full bg-black/5 px-2 py-0.5 text-[11px] text-gray-600 dark:bg-white/10 dark:text-gray-300"
                        >
                            #{tag}
                        </span>
                    ))}
                </div>

                <BubbleBody item={item} onToggleTodo={onToggleTodo} />

                <div className="mt-1 flex items-center justify-end gap-0.5">
                    {(item.kind === 'clipboard' || item.kind === 'bookmark') && (
                        <button
                            onClick={copyText}
                            title="Copy"
                            className="rounded p-1 text-gray-500 opacity-0 hover:bg-black/10 focus:opacity-100 group-hover:opacity-100 dark:text-gray-400 dark:hover:bg-white/10"
                        >
                            {copied ? <Check size={13} className="text-emerald-600 dark:text-emerald-400" /> : <Copy size={13} />}
                        </button>
                    )}
                    <button
                        onClick={onEdit}
                        title="Edit"
                        className="rounded p-1 text-gray-500 opacity-0 hover:bg-black/10 focus:opacity-100 group-hover:opacity-100 dark:text-gray-400 dark:hover:bg-white/10"
                    >
                        <Pencil size={13} />
                    </button>
                    <button
                        onClick={onDelete}
                        title="Delete"
                        className="rounded p-1 text-gray-500 opacity-0 hover:bg-black/10 hover:text-red-500 focus:opacity-100 group-hover:opacity-100 dark:text-gray-400 dark:hover:bg-white/10"
                    >
                        <Trash2 size={13} />
                    </button>
                    <span
                        className="ml-1 select-none text-[11px] text-gray-500 dark:text-gray-400"
                        title={new Date(item.updatedAt * 1000).toLocaleString()}
                    >
                        {formatTime(item.updatedAt)}
                    </span>
                </div>
            </div>
        </div>
    )
}

function BubbleBody({ item, onToggleTodo }: { item: TimelineItem; onToggleTodo: () => void }) {
    const raw = item.raw ?? {}

    if (item.kind === 'todo') {
        return (
            <div className="min-w-0">
                <div className="flex items-start gap-2">
                    <button
                        onClick={onToggleTodo}
                        title={item.completed ? 'Mark as pending' : 'Mark as done'}
                        className={[
                            'mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded border transition-colors',
                            item.completed
                                ? 'border-emerald-600 bg-emerald-600 text-white'
                                : 'border-gray-400 hover:border-emerald-500 dark:border-gray-500',
                        ].join(' ')}
                    >
                        {item.completed && <Check size={13} />}
                    </button>
                    <div className="min-w-0">
                        <div
                            dir="auto"
                            className={`break-words text-sm font-semibold ${
                                item.completed
                                    ? 'text-gray-500 line-through dark:text-gray-400'
                                    : 'text-gray-900 dark:text-gray-100'
                            }`}
                        >
                            {item.title}
                        </div>
                        {item.subtitle && (
                            <AutoDirText text={item.subtitle} className="mt-0.5 line-clamp-4 text-sm text-gray-700 dark:text-gray-300" />
                        )}
                        {raw.due_date != null && (
                            <span className="mt-1 inline-flex items-center rounded-full bg-black/5 px-2 py-0.5 text-[11px] text-gray-600 dark:bg-white/10 dark:text-gray-300">
                                Due {formatDayLabel(Number(raw.due_date))}
                            </span>
                        )}
                    </div>
                </div>
            </div>
        )
    }

    if (item.kind === 'bookmark') {
        return (
            <a
                href={String(raw.url ?? '#')}
                target="_blank"
                rel="noreferrer"
                className="block min-w-0 rounded-xl bg-white/70 p-2.5 hover:bg-white dark:bg-gray-900/40 dark:hover:bg-gray-900/70"
            >
                <div className="flex items-center gap-2">
                    <span className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full ${KIND_ACCENTS.bookmark}`}>
                        <Bookmark size={15} />
                    </span>
                    <div className="min-w-0 flex-1">
                        <div dir="auto" className="truncate text-sm font-semibold text-gray-900 dark:text-gray-100">
                            {item.title}
                        </div>
                        <div className="flex items-center gap-1 text-xs text-violet-600 dark:text-violet-400">
                            <ExternalLink size={11} />
                            <span className="truncate">{String(raw.url ?? '')}</span>
                        </div>
                    </div>
                </div>
                {item.subtitle && item.subtitle !== raw.url && (
                    <AutoDirText text={item.subtitle} className="mt-1.5 line-clamp-3 text-sm text-gray-600 dark:text-gray-400" />
                )}
            </a>
        )
    }

    if (item.kind === 'contact') {
        const phones: string[] = Array.isArray(raw.phones) ? raw.phones : []
        const emails: string[] = Array.isArray(raw.emails) ? raw.emails : []
        return (
            <div className="min-w-0">
                <div className="flex items-center gap-2">
                    <span className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-bold uppercase ${KIND_ACCENTS.contact}`}>
                        {item.title.charAt(0) || '?'}
                    </span>
                    <div dir="auto" className="break-words text-sm font-semibold text-gray-900 dark:text-gray-100">
                        {item.title}
                    </div>
                </div>
                {phones.length > 0 && (
                    <div className="mt-1.5 flex items-center gap-1.5 text-xs text-gray-600 dark:text-gray-300">
                        <Phone size={12} />
                        <span dir="ltr">{phones.join(', ')}</span>
                    </div>
                )}
                {emails.length > 0 && (
                    <div className="mt-1 flex items-center gap-1.5 text-xs text-gray-600 dark:text-gray-300">
                        <Mail size={12} />
                        <span dir="ltr">{emails.join(', ')}</span>
                    </div>
                )}
                {typeof raw.notes === 'string' && raw.notes.trim() !== '' && (
                    <AutoDirText text={raw.notes} className="mt-1.5 line-clamp-4 text-sm text-gray-700 dark:text-gray-300" />
                )}
            </div>
        )
    }

    if (item.kind === 'credential') {
        return (
            <div className="min-w-0 rounded-xl bg-white/70 p-2.5 dark:bg-gray-900/40">
                <div className="flex items-center gap-2">
                    <span className={`flex h-8 w-8 shrink-0 items-center justify-center rounded-full ${KIND_ACCENTS.credential}`}>
                        <Lock size={15} />
                    </span>
                    <div className="min-w-0">
                        <div dir="auto" className="truncate text-sm font-semibold text-gray-900 dark:text-gray-100">
                            {item.title}
                        </div>
                        {item.subtitle && (
                            <div dir="auto" className="truncate text-xs text-gray-500 dark:text-gray-400">
                                {item.subtitle}
                            </div>
                        )}
                    </div>
                </div>
                <p className="mt-1.5 flex items-center gap-1 text-[11px] text-gray-500 dark:text-gray-400">
                    <Lock size={11} />
                    Secrets stay encrypted — manage them in the Credentials vault.
                </p>
            </div>
        )
    }

    // note + clipboard
    return (
        <div className="min-w-0">
            {item.kind === 'note' && (
                <div dir="auto" className="break-words text-sm font-semibold text-gray-900 dark:text-gray-100">
                    {item.title}
                </div>
            )}
            {item.subtitle && (
                <AutoDirText
                    text={item.subtitle}
                    className={`mt-0.5 line-clamp-8 text-sm text-gray-700 dark:text-gray-300 ${
                        item.kind === 'clipboard' ? 'font-mono break-all' : ''
                    }`}
                />
            )}
        </div>
    )
}
