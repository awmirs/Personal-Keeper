import { createContext, useContext, useState, ReactNode } from 'react'
import ConfirmationDialog from "../components/ConfirmationDialog.tsx";

interface ConfirmationContextType {
    confirm: (message: string, title?: string) => Promise<boolean>
}

const ConfirmationContext = createContext<ConfirmationContextType | null>(null)

export function ConfirmationProvider({ children }: { children: ReactNode }) {
    const [dialogState, setDialogState] = useState<{
        isOpen: boolean
        message: string
        title?: string
        resolve?: (value: boolean) => void
    }>({ isOpen: false, message: '' })

    const confirm = (message: string, title?: string): Promise<boolean> => {
        return new Promise((resolve) => {
            setDialogState({ isOpen: true, message, title, resolve })
        })
    }

    const handleConfirm = () => {
        dialogState.resolve?.(true)
        setDialogState({ isOpen: false, message: '' })
    }

    const handleCancel = () => {
        dialogState.resolve?.(false)
        setDialogState({ isOpen: false, message: '' })
    }

    return (
        <ConfirmationContext.Provider value={{ confirm }}>
            {children}
            <ConfirmationDialog
                isOpen={dialogState.isOpen}
                title={dialogState.title || 'Confirm'}
                message={dialogState.message}
                onConfirm={handleConfirm}
                onCancel={handleCancel}
            />
        </ConfirmationContext.Provider>
    )
}

export function useConfirmation() {
    const context = useContext(ConfirmationContext)
    if (!context) {
        throw new Error('useConfirmation must be used within a ConfirmationProvider')
    }
    return context
}