// src/stores/viewStore.ts
import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type ViewType = 'list' | 'grid' | 'compact' | 'card' | 'table'

interface ViewState {
    views: Record<string, ViewType>
    setView: (vault: string, view: ViewType) => void
    getView: (vault: string) => ViewType
}

export const useViewStore = create<ViewState>()(
    persist(
        (set, get) => ({
            views: {
                notes: 'list',
                clipboard: 'list',
                todos: 'list',
                bookmarks: 'list',
                contacts: 'list',
                credentials: 'list',
            },
            setView: (vault, view) =>
                set((state) => ({ views: { ...state.views, [vault]: view } })),
            getView: (vault) => get().views[vault] || 'list',
        }),
        { name: 'view-preferences' }
    )
)