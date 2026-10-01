// frontend/src/lib/usePaginatedList.ts
// Cursor-driven list hook backing the vault views. Wraps the paginated
// list endpoints (`{items, next_cursor}`) so views only deal with an
// append-only array, a "load more" callback, and simple loading / error
// flags. Optimistic updates go through `setItems`, matching the shape
// the vault views used before pagination.

import { useCallback, useEffect, useRef, useState } from 'react'
import type { Dispatch, SetStateAction } from 'react'
import api from './api'

interface Page<T> {
    items: T[]
    next_cursor: string | null
}

/**
 * Per-vault page size. Kept small where items tend to be content-heavy
 * (notes) and larger where they are compact (clipboard / todos).
 */
export const VAULT_PAGE_SIZE: Record<string, number> = {
    notes: 25,
    clipboard: 50,
    todos: 50,
    bookmarks: 50,
    contacts: 50,
    credentials: 50,
}

export interface PaginatedList<T> {
    items: T[]
    setItems: Dispatch<SetStateAction<T[]>>
    nextCursor: string | null
    loading: boolean
    loadingMore: boolean
    error: string | null
    loadMoreError: string | null
    reload: () => Promise<void>
    loadMore: () => Promise<void>
}

function errorMessage(err: unknown, fallback: string): string {
    if (err instanceof Error && err.message) return err.message
    return fallback
}

/**
 * @param url  List endpoint (e.g. `/notes`), or `null` to disable
 *             fetching entirely (used by the credentials vault while
 *             the vault is locked).
 * @param limit Page size for this list.
 */
export function usePaginatedList<T>(url: string | null, limit: number): PaginatedList<T> {
    const [items, setItems] = useState<T[]>([])
    const [nextCursor, setNextCursor] = useState<string | null>(null)
    const [loading, setLoading] = useState(url !== null)
    const [loadingMore, setLoadingMore] = useState(false)
    const [error, setError] = useState<string | null>(null)
    const [loadMoreError, setLoadMoreError] = useState<string | null>(null)

    // Guard against a stale response landing after `url` changed.
    const urlRef = useRef(url)
    urlRef.current = url

    const fetchPage = useCallback(
        async (cursor: string | null): Promise<Page<T>> => {
            if (!url) throw new Error('no url')
            const params: Record<string, string | number> = { limit }
            if (cursor) params.cursor = cursor
            const res = await api.get(url, { params })
            const data = res.data as Page<T> | T[]
            if (Array.isArray(data)) return { items: data, next_cursor: null }
            return {
                items: Array.isArray(data.items) ? data.items : [],
                next_cursor: data.next_cursor ?? null,
            }
        },
        [url, limit],
    )

    const reload = useCallback(async () => {
        if (!url) {
            setItems([])
            setNextCursor(null)
            setLoading(false)
            setError(null)
            setLoadMoreError(null)
            return
        }
        const target = url
        setLoading(true)
        setError(null)
        setLoadMoreError(null)
        try {
            const page = await fetchPage(null)
            if (urlRef.current !== target) return
            setItems(page.items)
            setNextCursor(page.next_cursor)
        } catch (err) {
            if (urlRef.current !== target) return
            setError(errorMessage(err, 'Failed to load'))
        } finally {
            if (urlRef.current === target) setLoading(false)
        }
    }, [url, fetchPage])

    const loadMore = useCallback(async () => {
        if (!url || !nextCursor || loadingMore || loading) return
        const target = url
        const cursor = nextCursor
        setLoadingMore(true)
        setLoadMoreError(null)
        try {
            const page = await fetchPage(cursor)
            if (urlRef.current !== target) return
            setItems((prev) => [...prev, ...page.items])
            setNextCursor(page.next_cursor)
        } catch (err) {
            if (urlRef.current !== target) return
            setLoadMoreError(errorMessage(err, 'Failed to load more'))
        } finally {
            if (urlRef.current === target) setLoadingMore(false)
        }
    }, [url, nextCursor, loadingMore, loading, fetchPage])

    // Reset the list whenever `url` changes (e.g. credentials unlock/lock).
    useEffect(() => {
        setItems([])
        setNextCursor(null)
        setLoadMoreError(null)
        if (url) {
            void reload()
        } else {
            setLoading(false)
            setError(null)
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [url])

    return {
        items,
        setItems,
        nextCursor,
        loading,
        loadingMore,
        error,
        loadMoreError,
        reload,
        loadMore,
    }
}
