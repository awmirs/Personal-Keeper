// frontend/src/components/history/ItemHistoryButton.tsx
// Reusable per-item "Version history" trigger: opens the
// VersionHistoryModal for a single vault item. Drop it next to any
// edit/delete affordance and pass the item's type + id.

import { useState } from 'react'
import { History } from 'lucide-react'
import VersionHistoryModal from './VersionHistoryModal'

interface ItemHistoryButtonProps {
    /** One of: note, clipboard, todo, bookmark, contact, credential */
    itemType: string
    /** Id of the item whose history should be shown */
    itemId: string
    /** Optional display title shown in the modal header */
    title?: string
    /** Optional override for the button classes */
    className?: string
    /** Icon size (defaults to 14, matching sibling action buttons) */
    size?: number
    /** Called after a successful restore; defaults to reloading the page */
    onRestored?: () => void
}

export default function ItemHistoryButton({
    itemType,
    itemId,
    title,
    className = '',
    size = 14,
    onRestored,
}: ItemHistoryButtonProps) {
    const [open, setOpen] = useState(false)

    return (
        <>
            <button
                type="button"
                onClick={(e) => { e.stopPropagation(); setOpen(true) }}
                title="Version history"
                className={
                    className ||
                    'rounded p-1 text-gray-500 hover:bg-black/10 dark:text-gray-400 dark:hover:bg-white/10'
                }
            >
                <History size={size} />
            </button>
            {open && (
                <VersionHistoryModal
                    itemType={itemType}
                    itemId={itemId}
                    title={title}
                    onRestored={onRestored ?? (() => window.location.reload())}
                    onClose={() => setOpen(false)}
                />
            )}
        </>
    )
}
