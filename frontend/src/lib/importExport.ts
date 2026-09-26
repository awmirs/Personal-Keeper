// Client-side import/export helpers for vault data.
//
// Export formats:
//   - JSON:     full-fidelity array of the vault's items (best for backups).
//   - CSV:      flattened table; arrays and nested objects are embedded as
//               JSON text inside their cells (spreadsheet-friendly).
//   - Markdown: a readable table of every item field.
//
// Import accepts exactly the formats produced by export (JSON, CSV and
// Markdown tables), plus a content-sniffing fallback for unknown file
// extensions. Before anything is written, every parsed item is classified
// against the vault's current contents as NEW, DUPLICATE (matched by id or
// by content fingerprint) or INVALID (missing required fields); the UI
// shows a review step where the user decides what happens to duplicates.
// Writes go through the shared axios instance (lib/api.ts) so
// authentication and token refresh are handled centrally.

import api from './api'

export type ExportFormat = 'json' | 'csv' | 'md'

export type ImportStatus = 'new' | 'duplicate' | 'invalid'

export type ImportStrategy = 'skip' | 'replace' | 'copy'

export interface ImportItem {
    key: string
    data: Record<string, unknown>
    title: string
    detail: string
    status: ImportStatus
    matchId: string | null
    reason: string | null
}

export interface ImportResult {
    created: number
    replaced: number
    skipped: number
    failed: number
    errors: string[]
}

type Item = Record<string, unknown>

// ---------------------------------------------------------------------------
// Per-vault definitions
// ---------------------------------------------------------------------------

interface VaultSpec {
    endpoint: string
    required: string[]
    titleFields: string[]
    detailFields: string[]
    fingerprint: (item: Item) => string
}

function normalizeText(value: unknown): string {
    if (value === null || value === undefined) return ''
    if (Array.isArray(value)) return value.map((entry) => normalizeText(entry)).join(',')
    if (typeof value === 'object') return JSON.stringify(value)
    return String(value).trim().replace(/\s+/g, ' ')
}

function normalizeList(value: unknown): string {
    if (Array.isArray(value)) {
        return value
            .map((entry) => normalizeText(entry))
            .filter((entry) => entry !== '')
            .sort()
            .join(',')
    }
    return normalizeText(value)
}

function fingerprintOf(...parts: unknown[]): string {
    return parts.map((part) => normalizeText(part)).join('\u0000')
}

const VAULT_SPECS: Record<string, VaultSpec> = {
    notes: {
        endpoint: 'notes',
        required: ['title'],
        titleFields: ['title'],
        detailFields: ['content'],
        fingerprint: (item) => fingerprintOf(item.title, item.content),
    },
    clipboard: {
        endpoint: 'clipboard',
        required: ['content'],
        titleFields: ['content'],
        detailFields: [],
        fingerprint: (item) => fingerprintOf(item.content),
    },
    todos: {
        endpoint: 'todos',
        required: ['title'],
        titleFields: ['title'],
        detailFields: ['description'],
        fingerprint: (item) => fingerprintOf(item.title, item.description),
    },
    bookmarks: {
        endpoint: 'bookmarks',
        required: ['url'],
        titleFields: ['title', 'url'],
        detailFields: ['url', 'description'],
        fingerprint: (item) => fingerprintOf(typeof item.url === 'string' ? item.url.toLowerCase() : item.url),
    },
    contacts: {
        endpoint: 'contacts',
        required: ['name'],
        titleFields: ['name'],
        detailFields: ['phones', 'emails', 'notes'],
        fingerprint: (item) =>
            fingerprintOf(
                typeof item.name === 'string' ? item.name.toLowerCase() : item.name,
                normalizeList(item.phones),
                normalizeList(item.emails)
            ),
    },
    credentials: {
        endpoint: 'credentials',
        required: ['website', 'username'],
        titleFields: ['website', 'username'],
        detailFields: ['username', 'url'],
        fingerprint: (item) =>
            fingerprintOf(
                typeof item.website === 'string' ? item.website.toLowerCase() : item.website,
                item.username
            ),
    },
}

const VAULT_ALIASES: Record<string, string> = {
    note: 'notes',
    todo: 'todos',
    bookmark: 'bookmarks',
    contact: 'contacts',
    credential: 'credentials',
}

function vaultSpec(vaultKey: string): VaultSpec {
    const key = (vaultKey || '').trim().toLowerCase()
    const alias = VAULT_ALIASES[key]
    const spec = VAULT_SPECS[key] ?? (alias !== undefined ? VAULT_SPECS[alias] : undefined)
    if (!spec) {
        throw new Error(`Unsupported vault "${vaultKey}".`)
    }
    return spec
}

// The Credentials vault stores encrypted secrets; its create endpoint
// expects plaintext values, so exported items cannot be re-imported.
export function isImportSupported(vaultKey: string): boolean {
    const key = (vaultKey || '').trim().toLowerCase()
    const alias = VAULT_ALIASES[key]
    return key !== 'credentials' && alias !== 'credentials'
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

export function exportItems(items: Item[], format: ExportFormat, baseName: string): void {
    if (!items || items.length === 0) return
    const safe = sanitizeBaseName(baseName)
    const stamp = timestamp()
    if (format === 'json') {
        downloadFile(`${safe}-${stamp}.json`, JSON.stringify(items, null, 2), 'application/json')
    } else if (format === 'csv') {
        downloadFile(`${safe}-${stamp}.csv`, toCsv(items), 'text/csv;charset=utf-8')
    } else {
        downloadFile(`${safe}-${stamp}.md`, toMarkdown(items), 'text/markdown;charset=utf-8')
    }
}

function timestamp(): string {
    const d = new Date()
    const pad = (n: number) => String(n).padStart(2, '0')
    return (
        `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}` +
        `-${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`
    )
}

function sanitizeBaseName(name: string): string {
    const cleaned = name
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '')
    return cleaned || 'export'
}

function downloadFile(filename: string, content: string, mimeType: string): void {
    const blob = new Blob([content], { type: mimeType })
    const url = URL.createObjectURL(blob)
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = filename
    document.body.appendChild(anchor)
    anchor.click()
    anchor.remove()
    URL.revokeObjectURL(url)
}

function collectColumns(items: Item[]): string[] {
    const columns: string[] = []
    for (const item of items) {
        for (const key of Object.keys(item)) {
            if (!columns.includes(key)) columns.push(key)
        }
    }
    return columns
}

function toCell(value: unknown): string {
    if (value === null || value === undefined) return ''
    if (typeof value === 'object') return JSON.stringify(value)
    return String(value)
}

function csvEscape(value: string): string {
    if (/[",\r\n]/.test(value)) {
        return `"${value.replace(/"/g, '""')}"`
    }
    return value
}

function toCsv(items: Item[]): string {
    const columns = collectColumns(items)
    const lines: string[] = [columns.map((column) => csvEscape(column)).join(',')]
    for (const item of items) {
        lines.push(columns.map((column) => csvEscape(toCell(item[column]))).join(','))
    }
    return lines.join('\r\n')
}

function mdEscape(value: string): string {
    return value.replace(/\|/g, '\\|').replace(/\r?\n/g, '<br>')
}

function toMarkdown(items: Item[]): string {
    const columns = collectColumns(items)
    const header = `| ${columns.map((column) => mdEscape(column)).join(' | ')} |`
    const divider = `| ${columns.map(() => '---').join(' | ')} |`
    const rows = items.map((item) => `| ${columns.map((column) => mdEscape(toCell(item[column]))).join(' | ')} |`)
    return [header, divider, ...rows].join('\n')
}

// ---------------------------------------------------------------------------
// Parsing (JSON / CSV / Markdown — the formats produced by export)
// ---------------------------------------------------------------------------

export async function parseImportedFile(file: File): Promise<Item[]> {
    const text = (await file.text()).replace(/^\uFEFF/, '')
    const name = file.name.toLowerCase()
    if (name.endsWith('.json')) return parseJson(text)
    if (name.endsWith('.csv')) return parseCsv(text)
    if (name.endsWith('.md') || name.endsWith('.markdown')) return parseMarkdownTable(text)
    const trimmed = text.trim()
    if (trimmed.startsWith('[') || trimmed.startsWith('{')) return parseJson(text)
    if (trimmed.startsWith('|')) return parseMarkdownTable(text)
    return parseCsv(text)
}

function parseJson(text: string): Item[] {
    const data: unknown = JSON.parse(text)
    if (Array.isArray(data)) {
        return data.filter((entry) => entry !== null && typeof entry === 'object') as Item[]
    }
    if (data !== null && typeof data === 'object') {
        const record = data as Record<string, unknown>
        for (const value of Object.values(record)) {
            if (
                Array.isArray(value) &&
                value.length > 0 &&
                value.every((entry) => entry !== null && typeof entry === 'object')
            ) {
                return value as Item[]
            }
        }
        return [record]
    }
    throw new Error('Unsupported JSON structure: expected an array of items.')
}

function parseCsv(text: string): Item[] {
    const rows = splitCsvRows(text)
    if (rows.length === 0) return []
    const header = (rows[0] ?? []).map((column) => column.trim())
    const items: Item[] = []
    for (let r = 1; r < rows.length; r += 1) {
        const row = rows[r] ?? []
        if (row.every((cell) => cell === '')) continue
        const item: Item = {}
        header.forEach((key, i) => {
            if (key !== '') item[key] = coerceValue(row[i] ?? '')
        })
        items.push(item)
    }
    return items
}

// Minimal RFC-4180-style CSV splitter: handles quoted cells, escaped double
// quotes (""), and commas or newlines inside quotes.
function splitCsvRows(text: string): string[][] {
    const rows: string[][] = []
    let row: string[] = []
    let cell = ''
    let inQuotes = false
    const src = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n')
    for (let i = 0; i < src.length; i += 1) {
        const ch = src[i] ?? ''
        if (inQuotes) {
            if (ch === '"') {
                if (src[i + 1] === '"') {
                    cell += '"'
                    i += 1
                } else {
                    inQuotes = false
                }
            } else {
                cell += ch
            }
        } else if (ch === '"') {
            inQuotes = true
        } else if (ch === ',') {
            row.push(cell)
            cell = ''
        } else if (ch === '\n') {
            row.push(cell)
            rows.push(row)
            row = []
            cell = ''
        } else {
            cell += ch
        }
    }
    if (cell !== '' || row.length > 0) {
        row.push(cell)
        rows.push(row)
    }
    return rows
}

// Parses the Markdown table emitted by toMarkdown(): "| a | b |" rows,
// "\|" escapes, "<br>" for embedded newlines, "---" divider skipped.
function parseMarkdownTable(text: string): Item[] {
    const lines = text
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter((line) => line.length > 0 && line.startsWith('|'))
    if (lines.length === 0) return []

    const rows = lines.map((line) => splitMarkdownRow(line))
    const header = rows[0] ?? []
    const body = rows.slice(1).filter((cells) => !cells.every((cell) => /^:?-{2,}:?$/.test(cell.trim())))

    const items: Item[] = []
    for (const cells of body) {
        if (cells.every((cell) => cell === '')) continue
        const item: Item = {}
        header.forEach((key, i) => {
            const name = key.trim()
            if (name !== '') {
                item[name] = coerceValue((cells[i] ?? '').replace(/<br\s*\/?>/gi, '\n'))
            }
        })
        items.push(item)
    }
    return items
}

function splitMarkdownRow(line: string): string[] {
    let body = line
    if (body.startsWith('|')) body = body.slice(1)
    if (body.endsWith('|')) body = body.slice(0, -1)
    const cells: string[] = []
    let cell = ''
    for (let i = 0; i < body.length; i += 1) {
        const ch = body[i] ?? ''
        if (ch === '\\' && body[i + 1] === '|') {
            cell += '|'
            i += 1
        } else if (ch === '|') {
            cells.push(cell.trim())
            cell = ''
        } else {
            cell += ch
        }
    }
    cells.push(cell.trim())
    return cells
}

// Best-effort CSV cell coercion so create DTOs receive proper types.
function coerceValue(raw: string): unknown {
    const value = raw.trim()
    if (value === '') return ''
    if (value === 'true') return true
    if (value === 'false') return false
    if (value === 'null') return null
    if (
        (value.startsWith('[') && value.endsWith(']')) ||
        (value.startsWith('{') && value.endsWith('}'))
    ) {
        try {
            return JSON.parse(value) as unknown
        } catch {
            return raw
        }
    }
    if (/^-?\d+(\.\d+)?$/.test(value)) {
        return Number(value)
    }
    return raw
}

// ---------------------------------------------------------------------------
// Classification: NEW / DUPLICATE / INVALID against current vault contents
// ---------------------------------------------------------------------------

function itemId(item: Item): string | null {
    const direct = item['id']
    if (typeof direct === 'string' && direct !== '') return direct
    const meta = item['meta']
    if (typeof meta === 'string') {
        try {
            const parsed = JSON.parse(meta) as Record<string, unknown> | null
            const id = parsed?.['id']
            if (typeof id === 'string' && id !== '') return id
        } catch {
            /* ignore malformed meta cells */
        }
    } else if (meta !== null && meta !== undefined && typeof meta === 'object') {
        const id = (meta as Item)['id']
        if (typeof id === 'string' && id !== '') return id
    }
    return null
}

function displayValue(item: Item, fields: string[]): string {
    for (const field of fields) {
        const value = normalizeText(item[field])
        if (value !== '') return value
    }
    return ''
}

function truncate(value: string, max: number): string {
    if (value.length <= max) return value
    return `${value.slice(0, Math.max(0, max - 1))}…`
}

export function classifyImportedItems(
    vaultKey: string,
    parsed: Item[],
    existing: Item[]
): ImportItem[] {
    const spec = vaultSpec(vaultKey)

    const existingById = new Map<string, Item>()
    const existingByFingerprint = new Map<string, Item>()
    for (const item of existing) {
        const id = itemId(item)
        if (id !== null) existingById.set(id, item)
        existingByFingerprint.set(spec.fingerprint(item), item)
    }

    const seenInFile = new Set<string>()
    const classified: ImportItem[] = []

    parsed.forEach((data) => {
        const missing = spec.required.filter((field) => normalizeText(data[field]) === '')
        let status: ImportStatus
        let matchId: string | null = null
        let reason: string | null = null

        if (missing.length > 0) {
            status = 'invalid'
            reason = `Missing required field${missing.length === 1 ? '' : 's'}: ${missing.join(', ')}`
        } else {
            const id = itemId(data)
            const byId = id !== null ? existingById.get(id) : undefined
            const fingerprint = spec.fingerprint(data)
            const byContent = existingByFingerprint.get(fingerprint)
            const inFile = seenInFile.has(fingerprint)

            if (byId !== undefined || byContent !== undefined || inFile) {
                status = 'duplicate'
                matchId =
                    byId !== undefined
                        ? itemId(byId)
                        : byContent !== undefined
                          ? itemId(byContent)
                          : null
                reason =
                    byId !== undefined
                        ? 'Same id as an existing item in this vault'
                        : byContent !== undefined
                          ? 'Same content as an existing item in this vault'
                          : 'Duplicated inside the imported file'
            } else {
                status = 'new'
                seenInFile.add(fingerprint)
            }
        }

        classified.push({
            key: `item-${classified.length}`,
            data,
            title: truncate(displayValue(data, spec.titleFields) || '(untitled)', 80),
            detail: truncate(displayValue(data, spec.detailFields), 100),
            status,
            matchId,
            reason,
        })
    })

    return classified
}

// ---------------------------------------------------------------------------
// Import execution
// ---------------------------------------------------------------------------

// Server-managed or binary fields that must never be sent back on create.
const SERVER_FIELDS = [
    'id',
    'meta',
    'position',
    'created_at',
    'updated_at',
    'user_id',
    'trash_status',
    'favicon',
    'thumbnail',
    'password_encrypted',
    'notes_encrypted',
    'totp_secret_encrypted',
]

const LIST_FIELDS = ['tags', 'phones', 'emails', 'addresses']

function normalizeListField(value: unknown): string[] {
    if (Array.isArray(value)) {
        return value.map((entry) => String(entry).trim()).filter((entry) => entry !== '')
    }
    if (typeof value === 'string' && value.trim() !== '') {
        return value
            .split(',')
            .map((entry) => entry.trim())
            .filter((entry) => entry !== '')
    }
    if (value === null || value === undefined) return []
    return [String(value)]
}

function buildCreatePayload(item: Item): Item {
    const payload: Item = {}
    for (const [key, value] of Object.entries(item)) {
        if (SERVER_FIELDS.includes(key)) continue
        payload[key] = value
    }
    for (const field of LIST_FIELDS) {
        if (payload[field] !== undefined) payload[field] = normalizeListField(payload[field])
    }
    // Option<number> fields must not arrive as empty or non-numeric strings.
    const due = payload['due_date']
    if (due !== undefined) {
        const numeric = typeof due === 'number' ? due : Number(due)
        if (Number.isFinite(numeric)) {
            payload['due_date'] = numeric
        } else {
            const parsed = Date.parse(String(due))
            if (Number.isNaN(parsed)) {
                delete payload['due_date']
            } else {
                payload['due_date'] = Math.floor(parsed / 1000)
            }
        }
    }
    return payload
}

function errorStatus(err: unknown): number | null {
    if (typeof err === 'object' && err !== null) {
        const response = (err as { response?: { status?: number } }).response
        if (response && typeof response.status === 'number') return response.status
    }
    return null
}

function errorText(err: unknown): string {
    if (typeof err === 'object' && err !== null) {
        const e = err as { response?: { data?: unknown }; message?: string }
        const data = e.response?.data
        if (data && typeof data === 'object') {
            const message = (data as Item)['error']
            if (typeof message === 'string' && message !== '') return message
        }
        if (typeof e.message === 'string' && e.message !== '') return e.message
    }
    return String(err)
}

export async function runImport(
    vaultKey: string,
    selected: ImportItem[],
    strategy: ImportStrategy
): Promise<ImportResult> {
    const spec = vaultSpec(vaultKey)
    const result: ImportResult = { created: 0, replaced: 0, skipped: 0, failed: 0, errors: [] }

    for (const entry of selected) {
        const label = entry.title || '(untitled)'
        try {
            if (entry.status === 'invalid') {
                result.skipped += 1
                continue
            }

            if (entry.status === 'duplicate' && strategy === 'skip') {
                result.skipped += 1
                continue
            }

            if (entry.status === 'duplicate' && strategy === 'replace' && entry.matchId !== null) {
                // Create the replacement first; the old item is removed only
                // after its replacement exists, so a failure never loses data.
                await api.post(`/${spec.endpoint}`, buildCreatePayload(entry.data))
                try {
                    await api.delete(`/${spec.endpoint}/${entry.matchId}`)
                    result.replaced += 1
                } catch (err) {
                    result.created += 1
                    result.errors.push(
                        `Imported "${label}" but could not remove the old item: ${errorText(err)}`
                    )
                }
                continue
            }

            // New items, "keep both" copies, and duplicates matched without a
            // stable id (content match or in-file duplicate) are created.
            await api.post(`/${spec.endpoint}`, buildCreatePayload(entry.data))
            result.created += 1
        } catch (err) {
            const status = errorStatus(err)
            if (status === 401 || status === 403) {
                throw new Error(
                    `Import aborted: not authorized while importing "${label}". Sign in again and retry.`
                )
            }
            result.failed += 1
            result.errors.push(`"${label}": ${errorText(err)}`)
        }
    }

    return result
}
