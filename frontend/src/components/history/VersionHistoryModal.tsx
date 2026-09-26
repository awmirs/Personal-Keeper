// frontend/src/components/history/VersionHistoryModal.tsx
// Full version-history browser for a single vault item:
//   • chronological list of every recorded version (created / updated / deleted / restored)
//   • snapshot preview of any version
//   • field-level + line-level diffs (default: previous → selected, or any two via checkboxes)
//   • restore an item to any previous version (works even after a hard delete)
//   • purge the recorded history of an item

import { useEffect, useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import { ArrowLeftRight, Eye, History, RefreshCw, Trash2, X } from 'lucide-react'
import {
    diffLines,
    formatTimestamp,
    formatVersionValue,
    getDiff,
    listVersions,
    OPERATION_CLASSES,
    OPERATION_LABELS,
    prettyFieldName,
    purgeHistory,
    restoreVersion,
    TYPE_LABELS,
    wantsLineDiff,
} from '../../lib/versioning'
import type { ItemVersion, VersionDiff } from '../../lib/versioning'

interface VersionHistoryModalProps {
    itemType: string
    itemId: string
    title?: string
    onRestored?: () => void
    onClose: () => void
}

export default function VersionHistoryModal({
    itemType,
    itemId,
    title,
    onRestored,
    onClose,
}: VersionHistoryModalProps) {
    const [versions, setVersions] = useState<ItemVersion[] | null>(null)
    const [loadError, setLoadError] = useState<string | null>(null)
    const [selectedVersion, setSelectedVersion] = useState<number | null>(null)
    const [mode, setMode] = useState<'diff' | 'snapshot'>('diff')
    const [diff, setDiff] = useState<VersionDiff | null>(null)
    const [diffError, setDiffError] = useState<string | null>(null)
    const [diffLoading, setDiffLoading] = useState(false)
    const [compare, setCompare] = useState<number[]>([])
    const [restoring, setRestoring] = useState(false)
    const [notice, setNotice] = useState<string | null>(null)
    const [noticeError, setNoticeError] = useState<string | null>(null)
    const [confirmPurge, setConfirmPurge] = useState(false)
    const [purging, setPurging] = useState(false)

    useEffect(() => {
        let cancelled = false
        setVersions(null)
        setLoadError(null)
        setSelectedVersion(null)
        setCompare([])
        listVersions(itemType, itemId)
            .then((result) => {
                if (cancelled) return
                setVersions(result)
                if (result.length > 0) setSelectedVersion(result[0].version)
            })
            .catch((err: unknown) => {
                if (cancelled) return
                setVersions([])
                setLoadError(err instanceof Error ? err.message : String(err))
            })
        return () => {
            cancelled = true
        }
    }, [itemType, itemId])

    const selected = useMemo(
        () => versions?.find((v) => v.version === selectedVersion) ?? null,
        [versions, selectedVersion],
    )

    const previousVersion = useMemo(() => {
        if (selectedVersion == null || !versions || versions.length === 0) return null
        const smaller = versions.map((v) => v.version).filter((n) => n < selectedVersion)
        if (smaller.length === 0) return null
        return Math.max(...smaller)
    }, [versions, selectedVersion])

    useEffect(() => {
        if (mode !== 'diff') return
        let a: number | null = null
        let b: number | null = null
        if (compare.length === 2) {
            const first = compare[0]
            const second = compare[1]
            if (first != null && second != null) {
                a = Math.min(first, second)
                b = Math.max(first, second)
            }
        } else if (selectedVersion != null) {
            a = previousVersion
            b = selectedVersion
        }
        if (a == null || b == null || a === b) {
            setDiff(null)
            setDiffError(null)
            setDiffLoading(false)
            return
        }
        let cancelled = false
        setDiffLoading(true)
        setDiffError(null)
        getDiff(itemType, itemId, a, b)
            .then((result) => {
                if (!cancelled) setDiff(result)
            })
            .catch((err: unknown) => {
                if (!cancelled) {
                    setDiff(null)
                    setDiffError(err instanceof Error ? err.message : String(err))
                }
            })
            .finally(() => {
                if (!cancelled) setDiffLoading(false)
            })
        return () => {
            cancelled = true
        }
    }, [mode, compare, selectedVersion, previousVersion, itemType, itemId])

    useEffect(() => {
        const handler = (event: KeyboardEvent) => {
            if (event.key === 'Escape') onClose()
        }
        document.addEventListener('keydown', handler)
        return () => document.removeEventListener('keydown', handler)
    }, [onClose])

    const toggleCompare = (version: number) => {
        setCompare((current) => {
            if (current.includes(version)) return current.filter((v) => v !== version)
            const next = [...current, version]
            if (next.length <= 2) return next
            return [next[next.length - 2] ?? version, next[next.length - 1] ?? version]
        })
        setMode('diff')
    }

    const handleRestore = async () => {
        if (selectedVersion == null || restoring) return
        setRestoring(true)
        setNotice(null)
        setNoticeError(null)
        try {
            await restoreVersion(itemType, itemId, selectedVersion)
            const refreshed = await listVersions(itemType, itemId)
            setVersions(refreshed)
            setCompare([])
            setSelectedVersion(refreshed[0]?.version ?? null)
            setNotice(`Version ${selectedVersion} restored — recorded as a new version.`)
            if (onRestored) window.setTimeout(onRestored, 800)
        } catch (err: unknown) {
            setNoticeError(err instanceof Error ? err.message : String(err))
        } finally {
            setRestoring(false)
        }
    }

    const handlePurge = async () => {
        if (purging) return
        setPurging(true)
        setNotice(null)
        setNoticeError(null)
        try {
            await purgeHistory(itemType, itemId)
            setVersions([])
            setSelectedVersion(null)
            setCompare([])
            setDiff(null)
            setConfirmPurge(false)
            setNotice('History purged. Future changes will start a fresh history.')
        } catch (err: unknown) {
            setNoticeError(err instanceof Error ? err.message : String(err))
        } finally {
            setPurging(false)
        }
    }

    const count = versions?.length ?? 0
    const newest = versions != null && versions.length > 0 ? versions[0] : null
    const wasDeleted = newest?.operation === 'deleted'

    return (
        <div
            className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm"
            onClick={onClose}
        >
            <div
                className="flex h-full max-h-[85vh] w-full max-w-4xl flex-col overflow-hidden rounded-xl bg-white shadow-2xl dark:bg-gray-800"
                onClick={(event) => event.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center gap-3 border-b border-gray-200 px-4 py-3 dark:border-gray-700">
                    <span className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-indigo-100 text-indigo-600 dark:bg-indigo-900/50 dark:text-indigo-300">
                        <History size={16} />
                    </span>
                    <div className="min-w-0 flex-1">
                        <h2 className="truncate text-sm font-semibold text-gray-900 dark:text-white">
                            Version history{title ? ` — ${title}` : ''}
                        </h2>
                        <p className="text-xs text-gray-500 dark:text-gray-400">
                            {TYPE_LABELS[itemType] ?? itemType} · {count} version{count === 1 ? '' : 's'}
                            {wasDeleted ? ' · item is deleted (restoring will bring it back)' : ''}
                        </p>
                    </div>
                    <div className="flex shrink-0 items-center gap-0.5 rounded-lg bg-gray-100 p-0.5 dark:bg-gray-700">
                        <ModeButton
                            active={mode === 'diff'}
                            onClick={() => setMode('diff')}
                            icon={<ArrowLeftRight size={13} />}
                            label="Diff"
                        />
                        <ModeButton
                            active={mode === 'snapshot'}
                            onClick={() => setMode('snapshot')}
                            icon={<Eye size={13} />}
                            label="Snapshot"
                        />
                    </div>
                    <button
                        onClick={onClose}
                        title="Close"
                        className="rounded p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    >
                        <X size={16} />
                    </button>
                </div>

                {(notice || noticeError) && (
                    <div
                        className={`border-b px-4 py-2 text-xs ${
                            noticeError
                                ? 'border-red-100 bg-red-50 text-red-700 dark:border-red-900 dark:bg-red-900/30 dark:text-red-300'
                                : 'border-emerald-100 bg-emerald-50 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-900/30 dark:text-emerald-300'
                        }`}
                    >
                        {noticeError ?? notice}
                    </div>
                )}

                {/* Body */}
                <div className="flex min-h-0 flex-1">
                    <div className="w-64 shrink-0 overflow-y-auto border-r border-gray-200 p-2 dark:border-gray-700">
                        {loadError && (
                            <div className="rounded-lg bg-red-50 p-3 text-xs text-red-700 dark:bg-red-900/30 dark:text-red-300">
                                {loadError}
                            </div>
                        )}
                        {versions === null && !loadError && (
                            <p className="px-2 py-6 text-center text-xs text-gray-500 dark:text-gray-400">
                                Loading history…
                            </p>
                        )}
                        {versions !== null && versions.length === 0 && (
                            <p className="px-2 py-6 text-center text-xs text-gray-500 dark:text-gray-400">
                                No versions recorded yet.
                            </p>
                        )}
                        {versions?.map((version) => (
                            <VersionRow
                                key={version.id}
                                version={version}
                                selected={version.version === selectedVersion}
                                compareSelected={compare.includes(version.version)}
                                canCompare={compare.length < 2 || compare.includes(version.version)}
                                onSelect={() => {
                                    setSelectedVersion(version.version)
                                    if (compare.length > 0) setCompare([])
                                }}
                                onToggleCompare={() => toggleCompare(version.version)}
                            />
                        ))}
                    </div>

                    <div className="min-w-0 flex-1 overflow-y-auto p-4">
                        {mode === 'snapshot' ? (
                            <SnapshotView version={selected} />
                        ) : (
                            <DiffPane diff={diff} loading={diffLoading} error={diffError} selected={selected} />
                        )}
                    </div>
                </div>

                {/* Footer */}
                <div className="flex flex-wrap items-center justify-between gap-2 border-t border-gray-200 px-4 py-2.5 dark:border-gray-700">
                    <p className="text-[11px] text-gray-400">
                        {compare.length === 2
                            ? 'Comparing the two checked versions.'
                            : 'Select a version to view it; check two boxes to compare any pair.'}
                    </p>
                    <div className="flex flex-wrap items-center gap-2">
                        {confirmPurge ? (
                            <>
                                <span className="text-xs text-gray-500 dark:text-gray-400">
                                    Permanently delete all {count} version{count === 1 ? '' : 's'}?
                                </span>
                                <button
                                    onClick={handlePurge}
                                    disabled={purging || count === 0}
                                    className="rounded-lg bg-red-500 px-2.5 py-1.5 text-xs font-semibold text-white hover:bg-red-600 disabled:opacity-50"
                                >
                                    {purging ? 'Deleting…' : 'Yes, purge'}
                                </button>
                                <button
                                    onClick={() => setConfirmPurge(false)}
                                    className="rounded-lg border border-gray-300 px-2.5 py-1.5 text-xs font-medium text-gray-600 hover:bg-gray-100 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-700"
                                >
                                    Cancel
                                </button>
                            </>
                        ) : (
                            <button
                                onClick={() => setConfirmPurge(true)}
                                disabled={count === 0}
                                className="flex items-center gap-1 rounded-lg border border-red-200 px-2.5 py-1.5 text-xs font-medium text-red-600 hover:bg-red-50 disabled:opacity-50 dark:border-red-900 dark:text-red-400 dark:hover:bg-red-900/30"
                            >
                                <Trash2 size={13} />
                                Purge history
                            </button>
                        )}
                        <button
                            onClick={handleRestore}
                            disabled={selectedVersion == null || restoring || count === 0}
                            className="flex items-center gap-1 rounded-lg bg-indigo-500 px-3 py-1.5 text-xs font-semibold text-white hover:bg-indigo-600 disabled:opacity-50"
                        >
                            <RefreshCw size={13} className={restoring ? 'animate-spin' : ''} />
                            {restoring ? 'Restoring…' : `Restore v${selectedVersion ?? '—'}`}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    )
}

function ModeButton({
    active,
    onClick,
    icon,
    label,
}: {
    active: boolean
    onClick: () => void
    icon: ReactNode
    label: string
}) {
    return (
        <button
            onClick={onClick}
            className={`flex items-center gap-1 rounded-md px-2 py-1 text-[11px] font-medium transition-colors ${
                active
                    ? 'bg-white text-gray-900 shadow-sm dark:bg-gray-600 dark:text-white'
                    : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
            }`}
        >
            {icon}
            {label}
        </button>
    )
}

function VersionRow({
    version,
    selected,
    compareSelected,
    canCompare,
    onSelect,
    onToggleCompare,
}: {
    version: ItemVersion
    selected: boolean
    compareSelected: boolean
    canCompare: boolean
    onSelect: () => void
    onToggleCompare: () => void
}) {
    return (
        <div
            onClick={onSelect}
            className={`mb-1 flex cursor-pointer items-start gap-2 rounded-lg border p-2 transition-colors ${
                selected
                    ? 'border-indigo-300 bg-indigo-50 dark:border-indigo-700 dark:bg-indigo-900/30'
                    : 'border-transparent hover:bg-gray-100 dark:hover:bg-gray-700'
            }`}
        >
            <input
                type="checkbox"
                checked={compareSelected}
                disabled={!canCompare}
                onChange={onToggleCompare}
                onClick={(event) => event.stopPropagation()}
                title="Mark for comparison"
                className="mt-0.5 h-3.5 w-3.5 shrink-0 accent-indigo-500"
            />
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                    <span className="text-xs font-semibold text-gray-900 dark:text-white">v{version.version}</span>
                    <span
                        className={`rounded-full px-1.5 py-0.5 text-[10px] font-medium ${
                            OPERATION_CLASSES[version.operation] ??
                            'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300'
                        }`}
                    >
                        {OPERATION_LABELS[version.operation] ?? version.operation}
                    </span>
                </div>
                <div className="mt-0.5 truncate text-[11px] text-gray-500 dark:text-gray-400" title={version.title ?? ''}>
                    {version.title || '(no title)'}
                </div>
                <div className="text-[10px] text-gray-400" title={formatTimestamp(version.created_at)}>
                    {formatTimestamp(version.created_at)}
                </div>
            </div>
        </div>
    )
}

function SnapshotView({ version }: { version: ItemVersion | null }) {
    if (!version) {
        return <EmptyState text="Select a version to view its snapshot." />
    }
    const entries = Object.entries(version.data ?? {}).filter(
        ([field]) => field !== 'id' && field !== 'user_id',
    )
    return (
        <div>
            <div className="flex flex-wrap items-center gap-2">
                <span className="text-sm font-semibold text-gray-900 dark:text-white">
                    Version {version.version}
                </span>
                <span
                    className={`rounded-full px-2 py-0.5 text-[10px] font-medium ${
                        OPERATION_CLASSES[version.operation] ?? ''
                    }`}
                >
                    {OPERATION_LABELS[version.operation] ?? version.operation}
                </span>
                <span className="text-xs text-gray-500 dark:text-gray-400">
                    {formatTimestamp(version.created_at)}
                </span>
            </div>
            <dl className="mt-3 space-y-2">
                {entries.map(([field, value]) => (
                    <div key={field} className="rounded-lg border border-gray-200 p-2.5 dark:border-gray-700">
                        <dt className="text-[11px] font-medium uppercase tracking-wide text-gray-400">
                            {prettyFieldName(field)}
                        </dt>
                        <dd className="mt-1 whitespace-pre-wrap break-words text-sm text-gray-800 dark:text-gray-200">
                            {formatVersionValue(field, value)}
                        </dd>
                    </div>
                ))}
            </dl>
        </div>
    )
}

function DiffPane({
    diff,
    loading,
    error,
    selected,
}: {
    diff: VersionDiff | null
    loading: boolean
    error: string | null
    selected: ItemVersion | null
}) {
    if (loading) {
        return <EmptyState text="Computing diff…" />
    }
    if (error) {
        return (
            <div className="rounded-lg bg-red-50 p-3 text-xs text-red-700 dark:bg-red-900/30 dark:text-red-300">
                {error}
            </div>
        )
    }
    if (!diff) {
        if (selected) {
            return (
                <div>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                        This is the first recorded version — there is nothing earlier to compare against.
                    </p>
                    <div className="mt-3">
                        <SnapshotView version={selected} />
                    </div>
                </div>
            )
        }
        return <EmptyState text="Select a version (or check two boxes to compare them)." />
    }
    return (
        <div>
            <div className="flex flex-wrap items-center gap-2 text-xs">
                <VersionPill
                    version={diff.version_a}
                    operation={diff.operation_a}
                    createdAt={diff.created_at_a}
                    tone="old"
                />
                <ArrowLeftRight size={12} className="text-gray-400" />
                <VersionPill
                    version={diff.version_b}
                    operation={diff.operation_b}
                    createdAt={diff.created_at_b}
                    tone="new"
                />
            </div>
            {diff.fields.length === 0 ? (
                <p className="mt-4 text-xs text-gray-500 dark:text-gray-400">
                    No differences between these versions.
                </p>
            ) : (
                <div className="mt-3 space-y-3">
                    {diff.fields.map((field) => (
                        <FieldDiffRow
                            key={field.field}
                            field={field.field}
                            oldValue={field.old_value}
                            newValue={field.new_value}
                        />
                    ))}
                </div>
            )}
        </div>
    )
}

function VersionPill({
    version,
    operation,
    createdAt,
    tone,
}: {
    version: number
    operation: string
    createdAt: number
    tone: 'old' | 'new'
}) {
    return (
        <span
            className={`flex items-center gap-1.5 rounded-full px-2.5 py-1 ${
                tone === 'old'
                    ? 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300'
                    : 'bg-indigo-50 text-indigo-700 dark:bg-indigo-900/40 dark:text-indigo-300'
            }`}
        >
            <span className="font-semibold">v{version}</span>
            <span className="text-[10px]">{OPERATION_LABELS[operation] ?? operation}</span>
            <span className="text-[10px] opacity-70">{formatTimestamp(createdAt)}</span>
        </span>
    )
}

function FieldDiffRow({ field, oldValue, newValue }: { field: string; oldValue: unknown; newValue: unknown }) {
    const oldText = typeof oldValue === 'string' ? oldValue : null
    const newText = typeof newValue === 'string' ? newValue : null
    const oldIsText = oldText != null && wantsLineDiff(field, oldText)
    const newIsText = newText != null && wantsLineDiff(field, newText)

    if (oldIsText && newIsText && oldText != null && newText != null) {
        return (
            <div className="rounded-lg border border-gray-200 p-2.5 dark:border-gray-700">
                <div className="text-[11px] font-medium uppercase tracking-wide text-gray-400">
                    {prettyFieldName(field)}
                </div>
                <LineDiffView oldText={oldText} newText={newText} />
            </div>
        )
    }
    if (oldIsText && oldText != null && newText == null) {
        return (
            <div className="rounded-lg border border-gray-200 p-2.5 dark:border-gray-700">
                <div className="text-[11px] font-medium uppercase tracking-wide text-gray-400">
                    {prettyFieldName(field)}
                </div>
                <LineDiffView oldText={oldText} newText={''} />
            </div>
        )
    }
    if (newIsText && newText != null && oldText == null) {
        return (
            <div className="rounded-lg border border-gray-200 p-2.5 dark:border-gray-700">
                <div className="text-[11px] font-medium uppercase tracking-wide text-gray-400">
                    {prettyFieldName(field)}
                </div>
                <LineDiffView oldText={''} newText={newText} />
            </div>
        )
    }
    return (
        <div className="rounded-lg border border-gray-200 p-2.5 dark:border-gray-700">
            <div className="text-[11px] font-medium uppercase tracking-wide text-gray-400">
                {prettyFieldName(field)}
            </div>
            <div className="mt-1 flex flex-wrap items-center gap-2 text-sm">
                <span className="max-w-full break-all rounded bg-red-50 px-1.5 py-0.5 text-red-700 dark:bg-red-900/40 dark:text-red-300">
                    {oldValue === null || oldValue === undefined ? (
                        <em className="not-italic opacity-60">(none)</em>
                    ) : (
                        formatVersionValue(field, oldValue)
                    )}
                </span>
                <span className="text-gray-400">→</span>
                <span className="max-w-full break-all rounded bg-emerald-50 px-1.5 py-0.5 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-300">
                    {newValue === null || newValue === undefined ? (
                        <em className="not-italic opacity-60">(none)</em>
                    ) : (
                        formatVersionValue(field, newValue)
                    )}
                </span>
            </div>
        </div>
    )
}

function LineDiffView({ oldText, newText }: { oldText: string; newText: string }) {
    const lines = useMemo(() => diffLines(oldText, newText), [oldText, newText])
    return (
        <div className="mt-1 overflow-hidden rounded-md border border-gray-200 dark:border-gray-700">
            {lines.map((line, index) => (
                <div
                    key={index}
                    className={`flex gap-2 px-2 py-0.5 font-mono text-[11px] leading-5 ${
                        line.type === 'add'
                            ? 'bg-emerald-50 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-200'
                            : line.type === 'remove'
                              ? 'bg-red-50 text-red-800 dark:bg-red-900/40 dark:text-red-200'
                              : 'text-gray-500 dark:text-gray-400'
                    }`}
                >
                    <span className="w-8 shrink-0 select-none text-right text-gray-400">
                        {line.type === 'add' ? line.newNo : line.oldNo || ''}
                    </span>
                    <span className="w-3 shrink-0 select-none">
                        {line.type === 'add' ? '+' : line.type === 'remove' ? '−' : ' '}
                    </span>
                    <span className="min-w-0 flex-1 whitespace-pre-wrap break-all" dir="auto">
                        {line.text || ' '}
                    </span>
                </div>
            ))}
        </div>
    )
}

function EmptyState({ text }: { text: string }) {
    return <p className="py-8 text-center text-xs text-gray-500 dark:text-gray-400">{text}</p>
}
