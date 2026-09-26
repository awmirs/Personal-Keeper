// frontend/src/components/ImportReviewModal.tsx
// Review step for vault imports: every parsed item is classified as
// NEW, DUPLICATE or INVALID against the vault's current contents. The
// user selects which items to import and how duplicates should be
// handled before anything is written; the outcome is then reported.
import { useEffect, useMemo, useState } from 'react'
import { X } from 'lucide-react'
import type { ImportItem, ImportResult, ImportStrategy } from '../lib/importExport'

interface ImportReviewModalProps {
    vaultTitle: string
    items: ImportItem[]
    busy: boolean
    result: ImportResult | null
    error: string | null
    onCancel: () => void
    onConfirm: (strategy: ImportStrategy, selectedKeys: string[]) => void
    onFinish: () => void
}

const STRATEGIES: Array<{ value: ImportStrategy; label: string; hint: string }> = [
    {
        value: 'skip',
        label: 'Skip duplicates',
        hint: 'Keep the existing items untouched — only new items are imported.',
    },
    {
        value: 'replace',
        label: 'Replace duplicates',
        hint: 'Re-create duplicates with the imported data. The old item is removed only after its replacement exists.',
    },
    {
        value: 'copy',
        label: 'Keep both',
        hint: 'Import duplicates as additional copies alongside the existing items.',
    },
]

export default function ImportReviewModal({
    vaultTitle,
    items,
    busy,
    result,
    error,
    onCancel,
    onConfirm,
    onFinish,
}: ImportReviewModalProps) {
    const importable = useMemo(() => items.filter((item) => item.status !== 'invalid'), [items])
    const newCount = useMemo(() => importable.filter((item) => item.status === 'new').length, [importable])
    const duplicateCount = useMemo(
        () => importable.filter((item) => item.status === 'duplicate').length,
        [importable]
    )
    const invalidCount = items.length - importable.length

    const [strategy, setStrategy] = useState<ImportStrategy>(duplicateCount > 0 ? 'skip' : 'copy')
    const [selected, setSelected] = useState<Set<string>>(() => new Set(importable.map((item) => item.key)))

    useEffect(() => {
        setSelected(new Set(importable.map((item) => item.key)))
    }, [importable])

    const toggleItem = (key: string) => {
        setSelected((prev) => {
            const next = new Set(prev)
            if (next.has(key)) next.delete(key)
            else next.add(key)
            return next
        })
    }

    const toggleAll = () => {
        setSelected((prev) =>
            prev.size === importable.length ? new Set() : new Set(importable.map((item) => item.key))
        )
    }

    const confirmImport = () => {
        onConfirm(strategy, Array.from(selected))
    }

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
            <div className="flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-lg bg-white shadow-xl dark:bg-gray-800">
                <div className="flex items-center justify-between border-b border-gray-200 px-5 py-3 dark:border-gray-700">
                    <h3 className="text-lg font-semibold dark:text-white">Import into {vaultTitle}</h3>
                    <button
                        type="button"
                        onClick={result ? onFinish : onCancel}
                        disabled={busy}
                        title="Close"
                        className="rounded p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    >
                        <X size={18} />
                    </button>
                </div>

                {result ? (
                    <ImportOutcome result={result} />
                ) : (
                    <div className="flex-1 overflow-y-auto px-5 py-4">
                        {error && <p className="mb-3 text-sm text-red-500">{error}</p>}

                        <div className="mb-4 flex flex-wrap items-center gap-2 text-xs">
                            <span className="rounded-full bg-blue-100 px-2.5 py-0.5 font-medium text-blue-700 dark:bg-blue-900/60 dark:text-blue-300">
                                {newCount} new
                            </span>
                            <span className="rounded-full bg-amber-100 px-2.5 py-0.5 font-medium text-amber-700 dark:bg-amber-900/60 dark:text-amber-300">
                                {duplicateCount} duplicate{duplicateCount === 1 ? '' : 's'}
                            </span>
                            {invalidCount > 0 && (
                                <span className="rounded-full bg-red-100 px-2.5 py-0.5 font-medium text-red-700 dark:bg-red-900/60 dark:text-red-300">
                                    {invalidCount} invalid
                                </span>
                            )}
                            <span className="text-gray-500 dark:text-gray-400">
                                {items.length} item{items.length === 1 ? '' : 's'} found in the file
                            </span>
                        </div>

                        {duplicateCount > 0 && (
                            <div className="mb-4">
                                <p className="mb-2 text-sm font-medium dark:text-white">
                                    What should happen to duplicates?
                                </p>
                                <div className="space-y-1.5">
                                    {STRATEGIES.map((option) => (
                                        <label
                                            key={option.value}
                                            className="flex cursor-pointer items-start gap-2 rounded border border-gray-200 p-2 text-sm dark:border-gray-600 dark:text-white"
                                        >
                                            <input
                                                type="radio"
                                                name="import-strategy"
                                                checked={strategy === option.value}
                                                onChange={() => setStrategy(option.value)}
                                                disabled={busy}
                                                className="mt-0.5"
                                            />
                                            <span>
                                                <span className="font-medium">{option.label}</span>
                                                <span className="block text-xs text-gray-500 dark:text-gray-400">
                                                    {option.hint}
                                                </span>
                                            </span>
                                        </label>
                                    ))}
                                </div>
                            </div>
                        )}

                        <div className="mb-2 flex items-center justify-between text-sm dark:text-white">
                            <label className="flex cursor-pointer items-center gap-2">
                                <input
                                    type="checkbox"
                                    checked={selected.size === importable.length && importable.length > 0}
                                    onChange={toggleAll}
                                    disabled={busy || importable.length === 0}
                                />
                                Select all
                            </label>
                            <span className="text-xs text-gray-500 dark:text-gray-400">
                                {selected.size} of {importable.length} selected
                            </span>
                        </div>

                        <ul className="divide-y divide-gray-100 rounded border border-gray-200 dark:divide-gray-700 dark:border-gray-600">
                            {items.map((item) => (
                                <li key={item.key} className="flex items-start gap-3 px-3 py-2">
                                    <input
                                        type="checkbox"
                                        checked={selected.has(item.key)}
                                        onChange={() => toggleItem(item.key)}
                                        disabled={item.status === 'invalid' || busy}
                                        className="mt-1"
                                    />
                                    <div className="min-w-0 flex-1">
                                        <div className="flex flex-wrap items-center gap-2">
                                            {statusBadge(item.status)}
                                            <span dir="auto" className="truncate text-sm font-medium dark:text-white">
                                                {item.title}
                                            </span>
                                        </div>
                                        {item.detail && (
                                            <p dir="auto" className="truncate text-xs text-gray-500 dark:text-gray-400">
                                                {item.detail}
                                            </p>
                                        )}
                                        {item.reason && (
                                            <p className="text-xs text-gray-400 dark:text-gray-500">{item.reason}</p>
                                        )}
                                    </div>
                                </li>
                            ))}
                        </ul>

                        <p className="mt-3 text-xs text-gray-400 dark:text-gray-500">
                            Ids, timestamps, ordering and other server-managed metadata are not restored; items are
                            re-created through the vault's API.
                        </p>
                    </div>
                )}

                <div className="flex items-center justify-end gap-2 border-t border-gray-200 px-5 py-3 dark:border-gray-700">
                    {result ? (
                        <button
                            type="button"
                            onClick={onFinish}
                            className="rounded bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700"
                        >
                            Done
                        </button>
                    ) : (
                        <>
                            <button
                                type="button"
                                onClick={onCancel}
                                disabled={busy}
                                className="rounded bg-gray-200 px-4 py-2 text-sm hover:bg-gray-300 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600"
                            >
                                Cancel
                            </button>
                            <button
                                type="button"
                                onClick={confirmImport}
                                disabled={busy || selected.size === 0}
                                className="rounded bg-blue-600 px-4 py-2 text-sm text-white hover:bg-blue-700 disabled:opacity-50"
                            >
                                {busy ? 'Importing…' : `Import ${selected.size} item${selected.size === 1 ? '' : 's'}`}
                            </button>
                        </>
                    )}
                </div>
            </div>
        </div>
    )
}

function ImportOutcome({ result }: { result: ImportResult }) {
    const rows: Array<{ label: string; value: number; className: string }> = [
        { label: 'Created', value: result.created, className: 'text-green-600 dark:text-green-400' },
        { label: 'Replaced', value: result.replaced, className: 'text-blue-600 dark:text-blue-400' },
        { label: 'Skipped', value: result.skipped, className: 'text-gray-500 dark:text-gray-400' },
        { label: 'Failed', value: result.failed, className: 'text-red-500' },
    ]
    return (
        <div className="flex-1 overflow-y-auto px-5 py-4">
            <p className="mb-3 text-sm font-medium dark:text-white">Import finished</p>
            <ul className="mb-3 space-y-1 text-sm">
                {rows.map((row) => (
                    <li key={row.label} className="flex items-center justify-between">
                        <span className="text-gray-600 dark:text-gray-300">{row.label}</span>
                        <span className={`font-semibold ${row.className}`}>{row.value}</span>
                    </li>
                ))}
            </ul>
            {result.errors.length > 0 && (
                <div className="rounded border border-red-200 bg-red-50 p-2 dark:border-red-800 dark:bg-red-900/30">
                    <p className="text-xs font-medium text-red-600 dark:text-red-400">
                        {result.errors.length} problem{result.errors.length === 1 ? '' : 's'}:
                    </p>
                    <ul className="mt-1 max-h-32 space-y-1 overflow-y-auto text-xs text-red-600 dark:text-red-400">
                        {result.errors.map((message, index) => (
                            <li key={`error-${index}`}>{message}</li>
                        ))}
                    </ul>
                </div>
            )}
            {(result.created > 0 || result.replaced > 0) && (
                <p className="mt-3 text-xs text-gray-400 dark:text-gray-500">
                    The page will reload to show the imported items.
                </p>
            )}
        </div>
    )
}

function statusBadge(status: ImportItem['status']) {
    if (status === 'new') {
        return (
            <span className="shrink-0 rounded-full bg-blue-100 px-2 py-0.5 text-[11px] font-medium text-blue-700 dark:bg-blue-900/60 dark:text-blue-300">
                New
            </span>
        )
    }
    if (status === 'duplicate') {
        return (
            <span className="shrink-0 rounded-full bg-amber-100 px-2 py-0.5 text-[11px] font-medium text-amber-700 dark:bg-amber-900/60 dark:text-amber-300">
                Duplicate
            </span>
        )
    }
    return (
        <span className="shrink-0 rounded-full bg-red-100 px-2 py-0.5 text-[11px] font-medium text-red-700 dark:bg-red-900/60 dark:text-red-300">
            Invalid
        </span>
    )
}
