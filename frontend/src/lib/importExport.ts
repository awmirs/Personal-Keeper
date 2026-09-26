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
// extensions. Nothing is written until the user has reviewed the
// classification of every parsed item (NEW / DUPLICATE / INVALID) and
// chosen a duplicate strategy; the confirmed items are then sent in a
// single full-fidelity bulk request to the vault's /import endpoint,
// which restores ids, tags, colors, favorites, timestamps and ordering.
// All requests go through the shared axios client (lib/api.ts) so
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
// Import execution (full-fidelity bulk endpoint)
// ---------------------------------------------------------------------------

// Fetches the FULL vault contents (not the search-filtered view) so that
// duplicate detection is complete.
export async function fetchVaultItems(vaultKey: string): Promise<Item[]> {
    const spec = vaultSpec(vaultKey)
    const res = await api.get(`/${spec.endpoint}`)
    const data: unknown = res.data
    if (!Array.isArray(data)) return []
    return data.filter((entry): entry is Item => entry !== null && typeof entry === 'object')
}

// Returns a copy of the item with its id (top-level flattened metadata or a
// nested meta object) replaced, so the backend upserts over that row.
function withItemId(item: Item, id: string): Item {
    const copy: Item = { ...item, id }
    const meta = copy['meta']
    if (meta !== null && typeof meta === 'object' && !Array.isArray(meta)) {
        copy['meta'] = { ...(meta as Item), id }
    }
    return copy
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

// Sends the confirmed items to the vault's bulk /import endpoint in one
// request. The client applies its classification decisions first (skip
// filtering, and id-rewriting so content-level duplicates overwrite the
// matched item in place); the backend re-checks every id, restores full
// item fidelity and reports per-item failures.
export async function runImport(
    vaultKey: string,
    selected: ImportItem[],
    strategy: ImportStrategy
): Promise<ImportResult> {
    const spec = vaultSpec(vaultKey)

    const payloads: Item[] = []
    let clientSkipped = 0

    for (const entry of selected) {
        if (entry.status === 'invalid') {
            clientSkipped += 1
            continue
        }
        if (entry.status === 'duplicate' && strategy === 'skip') {
            clientSkipped += 1
            continue
        }
        if (entry.status === 'duplicate' && strategy === 'replace' && entry.matchId !== null) {
            payloads.push(withItemId(entry.data, entry.matchId))
            continue
        }
        payloads.push(entry.data)
    }

    if (payloads.length === 0) {
        return { created: 0, replaced: 0, skipped: clientSkipped, failed: 0, errors: [] }
    }

    try {
        const res = await api.post(`/${spec.endpoint}/import`, {
            items: payloads,
            strategy,
        })
        const data = (res.data ?? {}) as Partial<ImportResult>
        const errors = Array.isArray(data.errors)
            ? data.errors.filter((message): message is string => typeof message === 'string')
            : []
        return {
            created: data.created ?? 0,
            replaced: data.replaced ?? 0,
            skipped: (data.skipped ?? 0) + clientSkipped,
            failed: data.failed ?? 0,
            errors,
        }
    } catch (err) {
        throw new Error(`Import failed: ${errorText(err)}`)
    }
}
