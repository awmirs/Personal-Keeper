// frontend/src/components/InfiniteScrollSentinel.tsx
// End-of-list sentinel that triggers `onLoadMore` when it scrolls into
// view. Re-observes whenever the loading state flips so that a short
// list keeps fetching pages until either the viewport fills or the
// backend runs out of items (no "Load more" click required).

import { useEffect, useRef } from 'react'
import { Loader2 } from 'lucide-react'

interface InfiniteScrollSentinelProps {
    hasMore: boolean
    loading: boolean
    error: string | null
    onLoadMore: () => void
}

export default function InfiniteScrollSentinel({
    hasMore,
    loading,
    error,
    onLoadMore,
}: InfiniteScrollSentinelProps) {
    const ref = useRef<HTMLDivElement>(null)

    useEffect(() => {
        const node = ref.current
        if (!node || !hasMore || loading || error) return
        const observer = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    if (entry.isIntersecting) onLoadMore()
                }
            },
            { rootMargin: '300px' },
        )
        observer.observe(node)
        return () => observer.disconnect()
    }, [hasMore, loading, error, onLoadMore])

    if (!hasMore && !loading && !error) return null

    return (
        <div ref={ref} className="flex flex-col items-center gap-2 py-4">
            {loading && (
                <span className="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400">
                    <Loader2 size={16} className="animate-spin" />
                    Loading more…
                </span>
            )}
            {error && (
                <button
                    type="button"
                    onClick={onLoadMore}
                    className="rounded-lg border border-red-200 px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 dark:border-red-900 dark:text-red-400 dark:hover:bg-red-900/30"
                >
                    {error} — tap to retry
                </button>
            )}
        </div>
    )
}
