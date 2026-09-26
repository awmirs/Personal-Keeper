// Client-side import/export helpers for vault data.
//
// Export formats:
//   - JSON:     full-fidelity array of the vault's items (best for backups
//               and for round-tripping back through Import).
//   - CSV:      flattened table; arrays and nested objects are embedded as
//               JSON text inside their cells (spreadsheet-friendly).
//   - Markdown: a readable table of every item field.
//
// Import formats: JSON and CSV (selected by file extension, with a
// content-sniffing fallback for unknown extensions).

export type ExportFormat = 'json' | 'csv' | 'md'

type Item = Record<string, unknown>

const VITE_ENV = (import.meta as unknown as { env?: Record<string, string | undefined> }).env
const API_BASE = VITE_ENV?.VITE_API_URL ?? '/api'

// Maps a vaultKey (which may be singular) onto the API resource segment.
// Unknown keys fall back to the key itself.
const VAULT_ENDPOINTS: Record<string, string> = {
    note: 'notes',
    notes: 'notes',
    clipboard: 'clipboard',
    clipboards: 'clipboard',
    todo: 'todos',
    todos: 'todos',
    bookmark: 'bookmarks',
    bookmarks: 'bookmarks',
    contact: 'contacts',
    contacts: 'contacts',
    credential: 'credentials',
    credentials: 'credentials',
}

const TOKEN_KEYS = ['token', 'auth_token', 'accessToken', 'access_token', 'jwt', 'jwt_token']

function authHeaders(): Record<string, string> {
    for (const key of TOKEN_KEYS) {
        const fromLocal = localStorage.getItem(key)
        if (fromLocal) return { Authorization: `Bearer ${fromLocal}` }
        const fromSession = sessionStorage.getItem(key)
        if (fromSession) return { Authorization: `Bearer ${fromSession}` }
    }
    return {}
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
// Import
// ---------------------------------------------------------------------------

export async function parseImportedFile(file: File): Promise<Item[]> {
    const text = (await file.text()).replace(/^\uFEFF/, '')
    const name = file.name.toLowerCase()
    if (name.endsWith('.json')) return parseJson(text)
    if (name.endsWith('.csv')) return parseCsv(text)
    const trimmed = text.trim()
    if (trimmed.startsWith('[') || trimmed.startsWith('{')) return parseJson(text)
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

// Creates each parsed item through the vault's existing REST endpoint.
// Aborts early with a descriptive error on auth or endpoint problems.
export async function importItems(
    vaultKey: string,
    items: Item[]
): Promise<{ imported: number; failed: number }> {
    const key = vaultKey.toLowerCase()
    const endpoint = VAULT_ENDPOINTS[key] ?? key
    let imported = 0
    let failed = 0
    for (const item of items) {
        const res = await fetch(`${API_BASE}/${endpoint}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', ...authHeaders() },
            body: JSON.stringify(item),
        }).catch(() => null)
        if (res === null) {
            failed += 1
            continue
        }
        if (res.ok) {
            imported += 1
        } else if (res.status === 401 || res.status === 403) {
            throw new Error('Import failed: not authorized (401/403). Sign in again and retry.')
        } else if (res.status === 404) {
            throw new Error(`Import failed: POST ${API_BASE}/${endpoint} returned 404.`)
        } else {
            failed += 1
        }
    }
    return { imported, failed }
}