import React, { useRef, useState } from 'react'
import { Download, Plus, Search, Upload } from 'lucide-react'
import { classifyImportedItems, exportItems, fetchVaultItems, isImportSupported, parseImportedFile, runImport } from '../../lib/importExport'
import type { ImportItem, ImportResult, ImportStrategy } from '../../lib/importExport'
import ImportReviewModal from '../ImportReviewModal'
import LoadingSpinner from '../LoadingSpinner'
import ViewSwitcher from '../ViewSwitcher'
import ListView from '../views/ListView'
import GridView from '../views/GridView'
import CompactListView from '../views/CompactListView'
import { useViewStore, ViewType } from '../../stores/viewStore'

interface VaultLayoutProps<T> {
    title: string
    vaultKey: string
    search: string
    onSearchChange: (value: string) => void
    searchPlaceholder?: string
    editOrder: boolean
    onToggleEditOrder: () => void
    onAdd?: () => void
    addLabel?: string
    extraActions?: React.ReactNode
    createForm?: React.ReactNode
    loading: boolean
    loadingMessage?: string
    error?: string | null
    emptyMessage?: string
    items: T[]
    renderItem: (item: T, index: number) => React.ReactNode
    detailModal?: React.ReactNode
}

export default function VaultLayout<T>({
    title,
    vaultKey,
    search,
    onSearchChange,
    searchPlaceholder,
    editOrder,
    onToggleEditOrder,
    onAdd,
    addLabel = 'Add',
    extraActions,
    createForm,
    loading,
    loadingMessage,
    error,
    emptyMessage,
    items,
    renderItem,
    detailModal,
}: VaultLayoutProps<T>) {
    const view = useViewStore((s) => (s.views[vaultKey] || 'list') as ViewType)

    const [exportOpen, setExportOpen] = useState(false)
    const [importReview, setImportReview] = useState<ImportItem[] | null>(null)
    const [importBusy, setImportBusy] = useState(false)
    const [importResult, setImportResult] = useState<ImportResult | null>(null)
    const [importError, setImportError] = useState<string | null>(null)
    const fileInputRef = useRef<HTMLInputElement>(null)

    const handleExport = (format: 'json' | 'csv' | 'md') => {
        setExportOpen(false)
        exportItems(items as unknown as Record<string, unknown>[], format, title || vaultKey)
    }

    const handleImportFile = async (file: File) => {
        setImportError(null)
        setImportResult(null)
        if (!isImportSupported(vaultKey)) {
            setImportError(
                'Import is not available for the Credentials vault: secrets are stored encrypted and cannot be re-imported through the API.'
            )
            return
        }
        try {
            const parsed = await parseImportedFile(file)
            if (parsed.length === 0) {
                setImportError('No items found in the selected file.')
                return
            }
            // Classify against the full vault contents rather than the
            // currently displayed (possibly search-filtered) items.
            const existing = await fetchVaultItems(vaultKey)
            setImportReview(classifyImportedItems(vaultKey, parsed, existing))
        } catch (err) {
            setImportError(err instanceof Error ? err.message : 'Could not read the selected file.')
        }
    }

    const handleConfirmImport = async (strategy: ImportStrategy, selectedKeys: string[]) => {
        if (!importReview) return
        const keySet = new Set(selectedKeys)
        const selected = importReview.filter((entry) => keySet.has(entry.key))
        if (selected.length === 0) return
        setImportBusy(true)
        setImportError(null)
        try {
            const result = await runImport(vaultKey, selected, strategy)
            setImportResult(result)
        } catch (err) {
            setImportError(err instanceof Error ? err.message : 'Import failed.')
        } finally {
            setImportBusy(false)
        }
    }

    const handleFinishImport = () => {
        const changed = (importResult?.created ?? 0) > 0 || (importResult?.replaced ?? 0) > 0
        setImportReview(null)
        setImportResult(null)
        setImportError(null)
        if (changed) window.location.reload()
    }

    return (
        <div>
            <div className="flex items-center justify-between mb-6">
                <h2 className="text-2xl font-bold dark:text-white">{title}</h2>
                <div className="flex gap-2 items-center">
                    <ViewSwitcher vaultKey={vaultKey} />
                    <button
                        type="button"
                        onClick={onToggleEditOrder}
                        className={`flex items-center gap-2 rounded px-4 py-2 ${
                            editOrder
                                ? 'bg-green-600 hover:bg-green-700 text-white'
                                : 'bg-gray-200 dark:bg-gray-700 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-600'
                        }`}
                    >
                        {editOrder ? 'Done' : 'Edit Order'}
                    </button>
                    {onAdd && (
                        <button
                            type="button"
                            onClick={onAdd}
                            className="flex items-center gap-2 rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
                        >
                            <Plus size={18} /> {addLabel}
                        </button>
                    )}
                    <div className="relative">
                        <button
                            type="button"
                            onClick={() => setExportOpen((open) => !open)}
                            disabled={items.length === 0}
                            title="Export the items currently listed in this vault"
                            className="flex items-center gap-2 rounded bg-gray-200 px-4 py-2 dark:bg-gray-700 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-50"
                        >
                            <Download size={16} /> Export
                        </button>
                        {exportOpen && (
                            <div className="absolute right-0 z-20 mt-1 w-36 rounded border border-gray-200 bg-white py-1 shadow-lg dark:border-gray-600 dark:bg-gray-700">
                                <button type="button" onClick={() => handleExport('json')} className="block w-full px-4 py-2 text-left text-sm hover:bg-gray-100 dark:text-white dark:hover:bg-gray-600">JSON</button>
                                <button type="button" onClick={() => handleExport('csv')} className="block w-full px-4 py-2 text-left text-sm hover:bg-gray-100 dark:text-white dark:hover:bg-gray-600">CSV</button>
                                <button type="button" onClick={() => handleExport('md')} className="block w-full px-4 py-2 text-left text-sm hover:bg-gray-100 dark:text-white dark:hover:bg-gray-600">Markdown</button>
                            </div>
                        )}
                    </div>
                    <button
                        type="button"
                        onClick={() => fileInputRef.current?.click()}
                        disabled={importBusy}
                        title="Import items from a JSON, CSV or Markdown file"
                        className="flex items-center gap-2 rounded bg-gray-200 px-4 py-2 dark:bg-gray-700 dark:text-white hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-50"
                    >
                        <Upload size={16} /> {importBusy ? 'Importing…' : 'Import'}
                    </button>
                    <input
                        ref={fileInputRef}
                        type="file"
                        accept=".json,.csv,.md,.markdown"
                        className="hidden"
                        onChange={(e) => {
                            const file = e.target.files?.[0]
                            if (file) void handleImportFile(file)
                            e.target.value = ''
                        }}
                    />
                    {extraActions}
                </div>
            </div>

            {importError && !importReview && (
                <p className="mb-4 text-sm text-red-500">{importError}</p>
            )}

            <div className="relative mb-4">
                <Search className="absolute left-3 top-2.5 text-gray-400" size={18} />
                <input
                    type="text"
                    placeholder={searchPlaceholder || `Search ${title.toLowerCase()}...`}
                    value={search}
                    onChange={(e) => onSearchChange(e.target.value)}
                    className="w-full rounded border pl-10 pr-4 py-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                />
            </div>

            {createForm}

            {loading && <LoadingSpinner message={loadingMessage || `Loading ${title.toLowerCase()}...`} />}
            {error && <p className="text-red-500">Error: {error}</p>}
            {!loading && !error && items.length === 0 && (
                <p className="text-gray-500">{emptyMessage || `No ${title.toLowerCase()} found.`}</p>
            )}

            {view === 'list' && <ListView items={items} renderItem={renderItem} />}
            {view === 'grid' && <GridView items={items} renderItem={renderItem} />}
            {view === 'compact' && <CompactListView items={items} renderItem={renderItem} />}

            {detailModal}

            {importReview && (
                <ImportReviewModal
                    vaultTitle={title}
                    items={importReview}
                    busy={importBusy}
                    result={importResult}
                    error={importError}
                    onCancel={() => {
                        setImportReview(null)
                        setImportResult(null)
                        setImportError(null)
                    }}
                    onConfirm={handleConfirmImport}
                    onFinish={handleFinishImport}
                />
            )}
        </div>
    )
}
