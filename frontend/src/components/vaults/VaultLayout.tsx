import React from 'react'
import { Plus, Search } from 'lucide-react'
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
                    {extraActions}
                </div>
            </div>

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
        </div>
    )
}
