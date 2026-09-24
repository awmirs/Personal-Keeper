// src/components/ViewSwitcher.tsx
import { LayoutList, LayoutGrid, AlignJustify } from 'lucide-react'
import { useViewStore, ViewType } from '../stores/viewStore'

const icons: Record<string, React.ReactNode> = {
    list: <LayoutList size={18} />,
    grid: <LayoutGrid size={18} />,
    compact: <AlignJustify size={18} />,
}

const allowedViews: ViewType[] = ['list', 'grid', 'compact'] // we'll expand later

export default function ViewSwitcher({ vaultKey }: { vaultKey: string }) {
    const view = useViewStore((s) => s.views[vaultKey] || 'list')
    const setView = useViewStore((s) => s.setView)

    return (
        <div className="flex items-center gap-1 bg-gray-100 dark:bg-gray-700 rounded p-1">
            {allowedViews.map((key) => (
                <button
                    key={key}
                    type="button"
                    aria-label={`Switch to ${key} view`}
                    aria-pressed={view === key}
                    onClick={() => setView(vaultKey, key)}
                    className={`p-1 rounded ${
                        view === key
                            ? 'bg-white dark:bg-gray-600 shadow'
                            : 'hover:bg-gray-200 dark:hover:bg-gray-600'
                    }`}
                    title={key.charAt(0).toUpperCase() + key.slice(1)}
                >
                    {icons[key]}
                </button>
            ))}
        </div>
    )
}