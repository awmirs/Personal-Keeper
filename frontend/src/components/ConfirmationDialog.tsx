import { useEffect } from 'react'
import { X } from 'lucide-react'

interface ConfirmationDialogProps {
    isOpen: boolean
    title?: string
    message: string
    confirmText?: string
    cancelText?: string
    onConfirm: () => void
    onCancel: () => void
}

export default function ConfirmationDialog({
                                               isOpen,
                                               title = 'Confirm',
                                               message,
                                               confirmText = 'Delete',
                                               cancelText = 'Cancel',
                                               onConfirm,
                                               onCancel,
                                           }: ConfirmationDialogProps) {
    // Close on Escape key
    useEffect(() => {
        const handleEscape = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && isOpen) onCancel()
        }
        document.addEventListener('keydown', handleEscape)
        return () => document.removeEventListener('keydown', handleEscape)
    }, [isOpen, onCancel])

    if (!isOpen) return null

    return (
        <div
            className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
            onClick={onCancel}
        >
            <div
                className="w-full max-w-md rounded-lg bg-white dark:bg-gray-800 shadow-xl"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between border-b border-gray-200 dark:border-gray-700 p-4">
                    <h3 className="text-lg font-semibold dark:text-white">{title}</h3>
                    <button
                        onClick={onCancel}
                        className="rounded p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    >
                        <X size={20} />
                    </button>
                </div>
                <div className="p-6">
                    <p className="text-gray-700 dark:text-gray-300">{message}</p>
                </div>
                <div className="flex justify-end gap-3 border-t border-gray-200 dark:border-gray-700 p-4">
                    <button
                        onClick={onCancel}
                        className="rounded border border-gray-300 bg-white px-4 py-2 text-gray-700 hover:bg-gray-50 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600"
                    >
                        {cancelText}
                    </button>
                    <button
                        onClick={onConfirm}
                        className="rounded bg-red-600 px-4 py-2 text-white hover:bg-red-700"
                    >
                        {confirmText}
                    </button>
                </div>
            </div>
        </div>
    )
}