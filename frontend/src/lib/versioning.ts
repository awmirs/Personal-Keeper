// frontend/src/lib/versioning.ts
// Client-side support for the item history / versioning feature:
// API access (with auth token discovery), diff algorithms and display helpers.

// ---------- Types ----------

export type ItemOperation = 'created' | 'updated' | 'deleted' | 'restored'

export interface ItemVersion {
    id: string
    item_type: string
    item_id: string
    version: number
    operation: ItemOperation | string
    created_at: number
    title: string | null
    data: Record<string, unknown> | null
}

export interface FieldDiff {
    field: string
    old_value: unknown
    new_value: unknown
}

export interface VersionDiff {
    item_type: string
    item_id: string
    version_a: number
    version_b: number
    operation_a: string
    operation_b: string
    created_at_a: number
    created_at_b: number
    fields: FieldDiff[]
}

export interface ActivityEntry {
    id: string
    item_type: string
    item_id: string
    version: number
    operation: string
    created_at: number
    title: string | null
}

// ---------- API ----------

const env = (import.meta as unknown as { env?: Record<string, unknown> }).env
const rawBase = env && typeof env.VITE_API_URL === 'string' ? env.VITE_API_URL : ''
const API_BASE = rawBase.replace(/\/+$/, '')

const TOKEN_KEYS = ['access_token', 'accessToken', 'auth_token', 'authToken', 'token', 'jwt']
const TOKEN_CONTAINER_KEYS = ['auth', 'authState', 'session', 'user']

function extractToken(raw: string | null): string | null {
    if (!raw) return null
    if (raw.startsWith('{')) {
        try {
            const parsed = JSON.parse(raw) as Record<string, unknown>
            for (const key of ['access_token', 'accessToken', 'token', 'jwt']) {
                const value = parsed[key]
                if (typeof value === 'string' && value.length > 0) return value
            }
            return null
        } catch {
            return null
        }
    }
    return raw
}

/**
 * Best-effort discovery of the auth token used by the rest of the app,
 * so this module works standalone regardless of the storage key used.
 */
export function getAuthToken(): string | null {
    try {
        for (const key of TOKEN_KEYS) {
            const token = extractToken(localStorage.getItem(key))
            if (token) return token
        }
        for (const key of TOKEN_CONTAINER_KEYS) {
            const token = extractToken(localStorage.getItem(key))
            if (token) return token
        }
        for (const key of TOKEN_KEYS) {
            const token = extractToken(sessionStorage.getItem(key))
            if (token) return token
        }
    } catch {
        /* storage unavailable */
    }
    return null
}

async function historyFetch<T>(path: string, init?: RequestInit): Promise<T> {
    const headers: Record<string, string> = {}
    if (init?.body != null) headers['Content-Type'] = 'application/json'
    const token = getAuthToken()
    if (token) headers['Authorization'] = token.startsWith('Bearer ') ? token : `Bearer ${token}`
    const response = await fetch(`${API_BASE}${path}`, { ...init, headers })
    if (!response.ok) {
        let message = `${response.status} ${response.statusText}`
        try {
            const body = (await response.json()) as { error?: string }
            if (body && body.error) message = body.error
        } catch {
            /* non-JSON error body */
        }
        throw new Error(message)
    }
    if (response.status === 204) return undefined as T
    return (await response.json()) as T
}

export async function listVersions(itemType: string, itemId: string): Promise<ItemVersion[]> {
    return historyFetch<ItemVersion[]>(
        `/api/history/${encodeURIComponent(itemType)}/${encodeURIComponent(itemId)}`,
    )
}

export async function getVersion(
    itemType: string,
    itemId: string,
    version: number,
): Promise<ItemVersion> {
    return historyFetch<ItemVersion>(
        `/api/history/${encodeURIComponent(itemType)}/${encodeURIComponent(itemId)}/versions/${version}`,
    )
}

export async function getDiff(
    itemType: string,
    itemId: string,
    versionA: number,
    versionB: number,
): Promise<VersionDiff> {
    return historyFetch<VersionDiff>(
        `/api/history/${encodeURIComponent(itemType)}/${encodeURIComponent(itemId)}/diff/${versionA}/${versionB}`,
    )
}

export async function restoreVersion(
    itemType: string,
    itemId: string,
    version: number,
): Promise<ItemVersion> {
    return historyFetch<ItemVersion>(
        `/api/history/${encodeURIComponent(itemType)}/${encodeURIComponent(itemId)}/versions/${version}/restore`,
        { method: 'POST' },
    )
}

export async function purgeHistory(itemType: string, itemId: string): Promise<{ purged: number }> {
    return historyFetch<{ purged: number }>(
        `/api/history/${encodeURIComponent(itemType)}/${encodeURIComponent(itemId)}`,
        { method: 'DELETE' },
    )
}

export async function recentActivity(options?: {
    limit?: number
    itemType?: string
}): Promise<ActivityEntry[]> {
    const params = new URLSearchParams()
    if (options && options.limit != null) params.set('limit', String(options.limit))
    if (options && options.itemType) params.set('item_type', options.itemType)
    const suffix = params.toString() ? `?${params.toString()}` : ''
    return historyFetch<ActivityEntry[]>(`/api/history/recent${suffix}`)
}

// ---------- Line diff (LCS) ----------

export interface DiffLine {
    type: 'same' | 'add' | 'remove'
    text: string
    oldNo: number
    newNo: number
}

const cell = (array: Int32Array, index: number): number => array[index] ?? 0

/** Classic LCS line diff with a guard for pathological inputs. */
export function diffLines(oldText: string, newText: string): DiffLine[] {
    const a = oldText.length > 0 ? oldText.split('\n') : []
    const b = newText.length > 0 ? newText.split('\n') : []
    const n = a.length
    const m = b.length
    if (n === 0 && m === 0) return []
    if (n * m > 4000000) {
        const out: DiffLine[] = []
        let oldNo = 0
        let newNo = 0
        for (const line of a) out.push({ type: 'remove', text: line, oldNo: ++oldNo, newNo: 0 })
        for (const line of b) out.push({ type: 'add', text: line, oldNo: 0, newNo: ++newNo })
        return out
    }
    const width = m + 1
    const dp = new Int32Array((n + 1) * width)
    for (let i = n - 1; i >= 0; i--) {
        for (let j = m - 1; j >= 0; j--) {
            dp[i * width + j] =
                a[i] === b[j]
                    ? cell(dp, (i + 1) * width + (j + 1)) + 1
                    : Math.max(cell(dp, (i + 1) * width + j), cell(dp, i * width + (j + 1)))
        }
    }
    const out: DiffLine[] = []
    let i = 0
    let j = 0
    let oldNo = 0
    let newNo = 0
    while (i < n && j < m) {
        if (a[i] === b[j]) {
            out.push({ type: 'same', text: a[i] ?? '', oldNo: ++oldNo, newNo: ++newNo })
            i++
            j++
        } else if (cell(dp, (i + 1) * width + j) >= cell(dp, i * width + (j + 1))) {
            out.push({ type: 'remove', text: a[i] ?? '', oldNo: ++oldNo, newNo: 0 })
            i++
        } else {
            out.push({ type: 'add', text: b[j] ?? '', oldNo: 0, newNo: ++newNo })
            j++
        }
    }
    while (i < n) out.push({ type: 'remove', text: a[i++] ?? '', oldNo: ++oldNo, newNo: 0 })
    while (j < m) out.push({ type: 'add', text: b[j++] ?? '', oldNo: 0, newNo: ++newNo })
    return out
}

// ---------- Display helpers ----------

export const FIELD_LABELS: Record<string, string> = {
    title: 'Title',
    content: 'Content',
    description: 'Description',
    url: 'URL',
    website: 'Website',
    username: 'Username',
    name: 'Name',
    phones: 'Phone numbers',
    emails: 'Email addresses',
    addresses: 'Addresses',
    notes: 'Notes',
    tags: 'Tags',
    color_name: 'Color',
    color_hex: 'Color code',
    is_favorite: 'Favorite',
    is_pinned: 'Pinned',
    completed: 'Completed',
    due_date: 'Due date',
    trash_status: 'Status',
    position: 'Position',
    persist_to_disk: 'Persist to disk',
    password_encrypted: 'Password',
    notes_encrypted: 'Encrypted notes',
    totp_secret_encrypted: 'TOTP secret',
    created_at: 'Created',
    updated_at: 'Last modified',
}

export function prettyFieldName(field: string): string {
    return FIELD_LABELS[field] ?? field.replace(/_/g, ' ')
}

export const OPERATION_LABELS: Record<string, string> = {
    created: 'Created',
    updated: 'Updated',
    deleted: 'Deleted',
    restored: 'Restored',
}

export const OPERATION_CLASSES: Record<string, string> = {
    created: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/50 dark:text-emerald-300',
    updated: 'bg-sky-100 text-sky-700 dark:bg-sky-900/50 dark:text-sky-300',
    deleted: 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300',
    restored: 'bg-violet-100 text-violet-700 dark:bg-violet-900/50 dark:text-violet-300',
}

export const TYPE_LABELS: Record<string, string> = {
    note: 'Note',
    clipboard: 'Clipboard',
    todo: 'Todo',
    bookmark: 'Bookmark',
    contact: 'Contact',
    credential: 'Credential',
}

export function formatTimestamp(seconds: number): string {
    if (!seconds) return '—'
    return new Date(seconds * 1000).toLocaleString()
}

const BOOLEAN_FIELDS = new Set(['is_favorite', 'is_pinned', 'completed', 'persist_to_disk'])
const TIMESTAMP_FIELDS = new Set(['due_date', 'created_at', 'updated_at'])

/** Human-friendly rendering of a snapshot field value. */
export function formatVersionValue(field: string, value: unknown): string {
    if (value === null || value === undefined) return '—'
    if (typeof value === 'boolean') return value ? 'Yes' : 'No'
    if (typeof value === 'number') {
        if (BOOLEAN_FIELDS.has(field)) return value !== 0 ? 'Yes' : 'No'
        if (TIMESTAMP_FIELDS.has(field)) return value > 0 ? new Date(value * 1000).toLocaleString() : '—'
        return String(value)
    }
    if (Array.isArray(value)) {
        const rendered = value
            .map((item) => (typeof item === 'string' ? item : JSON.stringify(item)))
            .filter((item) => item !== null && item !== undefined && item !== '')
        return rendered.length > 0 ? rendered.join(', ') : '—'
    }
    if (typeof value === 'string') {
        const trimmed = value.trim()
        if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
            try {
                const parsed = JSON.parse(trimmed) as unknown
                if (Array.isArray(parsed)) return formatVersionValue(field, parsed)
            } catch {
                /* plain string */
            }
        }
        return value === '' ? '—' : value
    }
    return JSON.stringify(value)
}

/** Whether a field should be rendered with a line-level diff. */
export function wantsLineDiff(field: string, value: unknown): value is string {
    if (typeof value !== 'string') return false
    if (value.includes('\n')) return true
    return value.length > 120 && ['content', 'description', 'notes'].includes(field)
}
