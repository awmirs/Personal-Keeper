import { Loader2 } from 'lucide-react'

export default function LoadingSpinner({ message }: { message?: string }) {
    return (
        <div className="flex flex-col items-center justify-center py-12">
            <Loader2 className="animate-spin text-blue-500 dark:text-blue-400" size={36} />
            {message && (
                <p className="mt-3 text-sm text-gray-500 dark:text-gray-400">{message}</p>
            )}
        </div>
    )
}