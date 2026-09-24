// frontend/src/lib/timeline.ts
// Shared types and helpers for the "Saved Messages" chat timeline — a
// Telegram-style unified view over every vault (notes, todos, clipboard
// snippets, bookmarks, contacts, credentials).

export type VaultKind = 'note' | 'clipboard' | 'todo' | 'bookmark' | 'contact' | 'credential'

export interface TimelineItem {
    /** Composite key: `${kind}:${id}` — unique across all vaults. */
    key: string
    kind: VaultKind
    id: string
    /** Primary heading shown inside the bubble / search results. */
    title: string
    /** Secondary text used for previews and search snippets. */
    subtitle: string
    createdAt: number
    updatedAt: number
    tags: string[]
    isFavorite: boolean
    completed?: boolean
    /** Untouched API payload — spread + merged when editing. */
    raw: any
}

export const VAULT_KINDS: VaultKind[] = ['note', 'clipboard', 'todo', 'bookmark', 'contact', 'credential']

export const KIND_LABELS: Record<VaultKind, string> = {
    note: 'Note',
    clipboard: 'Clipboard',
    todo: 'Todo',
    bookmark: 'Bookmark',
    contact: 'Contact',
    credential: 'Credential',
}

export const KIND_API_BASES: Record<VaultKind, string> = {
    note: '/notes',
    clipboard: '/clipboard',
    todo: '/todos',
    bookmark: '/bookmarks',
    contact: '/contacts',
    credential: '/credentials',
}

/** Tailwind classes for the kind badge used in bubbles and search results. */
export const KIND_ACCENTS: Record<VaultKind, string> = {
    note: 'bg-sky-100 text-sky-700 dark:bg-sky-900/50 dark:text-sky-300',
    clipboard: 'bg-amber-100 text-amber-700 dark:bg-amber-900/50 dark:text-amber-300',
    todo: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/50 dark:text-emerald-300',
    bookmark: 'bg-violet-100 text-violet-700 dark:bg-violet-900/50 dark:text-violet-300',
    contact: 'bg-rose-100 text-rose-700 dark:bg-rose-900/50 dark:text-rose-300',
    credential: 'bg-slate-200 text-slate-700 dark:bg-slate-700 dark:text-slate-300',
}

/* ------------------------------------------------------------------ */
/* Date formatting (Telegram style)                                    */
/* ------------------------------------------------------------------ */

const MONTHS = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December',
]

/** API timestamps are unix seconds. */
export function toDate(ts: number): Date {
    return new Date(ts * 1000)
}

function startOfDay(d: Date): number {
    return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime()
}

/** "14:32" — the time shown inside every bubble. */
export function formatTime(ts: number): string {
    const d = toDate(ts)
    return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
}

/** Telegram-style day separator: Today / Yesterday / "February 14" / "March 3, 2022". */
export function formatDayLabel(ts: number): string {
    const d = toDate(ts)
    const now = new Date()
    const diffDays = Math.round((startOfDay(now) - startOfDay(d)) / 86400000)
    if (diffDays === 0) return 'Today'
    if (diffDays === 1) return 'Yesterday'
    const label = `${MONTHS[d.getMonth()]} ${d.getDate()}`
    return d.getFullYear() === now.getFullYear() ? label : `${label}, ${d.getFullYear()}`
}

function dayKey(ts: number): string {
    const d = toDate(ts)
    return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
}

export interface DayGroup {
    key: string
    label: string
    items: TimelineItem[]
}

/** Groups already sorted items into day buckets (in order). */
export function groupByDay(items: TimelineItem[]): DayGroup[] {
    const groups: DayGroup[] = []
    for (const item of items) {
        const key = dayKey(item.createdAt)
        const last = groups[groups.length - 1]
        if (last && last.key === key) {
            last.items.push(item)
        } else {
            groups.push({ key, label: formatDayLabel(item.createdAt), items: [item] })
        }
    }
    return groups
}

/* ------------------------------------------------------------------ */
/* Mapping raw API payloads to timeline items                          */
/* ------------------------------------------------------------------ */

function firstLine(text: string): string {
    const idx = text.indexOf('\n')
    return (idx === -1 ? text : text.slice(0, idx)).trim()
}

function tagsOf(raw: any): string[] {
    return Array.isArray(raw?.tags) ? raw.tags : []
}

export function toTimelineItem(kind: VaultKind, raw: any): TimelineItem {
    let title = ''
    let subtitle = ''
    let completed: boolean | undefined

    switch (kind) {
        case 'note':
            title = raw?.title?.trim() || 'Untitled note'
            subtitle = raw?.content ?? ''
            break
        case 'clipboard':
            title = firstLine(String(raw?.content ?? '')) || 'Clipboard item'
            subtitle = String(raw?.content ?? '')
            break
        case 'todo':
            title = raw?.title?.trim() || 'Untitled todo'
            subtitle = raw?.description ?? ''
            completed = Boolean(raw?.completed)
            break
        case 'bookmark':
            title = raw?.title?.trim() || raw?.url || 'Untitled bookmark'
            subtitle = raw?.description?.trim() || raw?.url || ''
            break
        case 'contact':
            title = raw?.name?.trim() || 'Unnamed contact'
            subtitle = [raw?.phones?.join(', '), raw?.emails?.join(', ')].filter(Boolean).join(' · ')
            break
        case 'credential':
            title = raw?.website?.trim() || raw?.username || 'Credential'
            subtitle = [raw?.username, raw?.url].filter(Boolean).join(' · ')
            break
    }

    const createdAt = Number(raw?.created_at ?? 0)
    const updatedAt = Number(raw?.updated_at ?? createdAt)

    return {
        key: `${kind}:${raw?.id}`,
        kind,
        id: String(raw?.id),
        title,
        subtitle: String(subtitle ?? ''),
        createdAt,
        updatedAt,
        tags: tagsOf(raw),
        isFavorite: Boolean(raw?.is_favorite),
        completed,
        raw,
    }
}

/** Keeps only items that are visible in the vaults (hides trashed/deleted). */
export function isActive(raw: any): boolean {
    const status = raw?.trash_status
    return !status || status === 'Active'
}

/** Chat order: strictly by creation time, ties broken by id for stability. */
export function sortTimeline(items: TimelineItem[]): TimelineItem[] {
    return [...items].sort((a, b) => a.createdAt - b.createdAt || a.id.localeCompare(b.id))
}

/* ------------------------------------------------------------------ */
/* Search                                                              */
/* ------------------------------------------------------------------ */

function searchableText(item: TimelineItem): string {
    const raw = item.raw ?? {}
    const parts: unknown[] = [
        item.title,
        item.subtitle,
        item.tags.join(' '),
        raw.content,
        raw.description,
        raw.url,
        raw.website,
        raw.username,
        raw.name,
        raw.notes,
        Array.isArray(raw.phones) ? raw.phones.join(' ') : '',
        Array.isArray(raw.emails) ? raw.emails.join(' ') : '',
        Array.isArray(raw.addresses) ? raw.addresses.join(' ') : '',
    ]
    return parts.map((p) => (p == null ? '' : String(p))).join('\n').toLowerCase()
}

export function itemMatchesQuery(item: TimelineItem, normalizedQuery: string): boolean {
    if (!normalizedQuery) return false
    return searchableText(item).includes(normalizedQuery)
}

/** Short snippet around the first match, used by the search sidebar. */
export function snippetFor(item: TimelineItem, normalizedQuery: string): string {
    const source = (item.subtitle || item.title || '').replace(/\s+/g, ' ').trim()
    if (!normalizedQuery) return source.slice(0, 120)
    const idx = source.toLowerCase().indexOf(normalizedQuery)
    if (idx === -1) return source.slice(0, 120)
    const start = Math.max(0, idx - 40)
    const end = Math.min(source.length, idx + normalizedQuery.length + 80)
    return `${start > 0 ? '…' : ''}${source.slice(start, end)}${end < source.length ? '…' : ''}`
}

/* ------------------------------------------------------------------ */
/* Composer helpers                                                    */
/* ------------------------------------------------------------------ */

/** Quick "smart" detection used by the composer when type is set to Auto. */
export function detectKind(text: string): VaultKind {
    const t = text.trim()
    if (/^https?:\/\/\S+$/i.test(t)) return 'bookmark'
    return 'note'
}

/** Payload used when the composer sends a quick message. */
export function buildComposerPayload(kind: VaultKind, text: string): Record<string, unknown> {
    const trimmed = text.trim()
    switch (kind) {
        case 'bookmark':
            return { url: trimmed, title: '', description: '', tags: [] }
        case 'clipboard':
            return { content: trimmed, tags: [] }
        case 'todo':
            return { title: trimmed.slice(0, 200), description: '', tags: [] }
        case 'contact':
            return { name: trimmed.slice(0, 120), phones: [], emails: [], addresses: [], notes: '', tags: [] }
        case 'credential':
            return { website: trimmed.slice(0, 120), url: '', username: '', tags: [] }
        case 'note':
        default: {
            const lines = trimmed.split('\n')
            const title = lines[0].trim().slice(0, 120) || 'Note'
            const content = lines.slice(1).join('\n')
            return { title, content, tags: [] }
        }
    }
}
