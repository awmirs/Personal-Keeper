export interface ItemMetadata {
    id: string
    created_at: number
    updated_at: number
    tags: Tag[]
    color: ColorLabel | null
    is_favorite: boolean
    trash_status: 'Active' | 'Trashed' | 'Deleted'
    position: number
}

export interface Tag {
    id: string
    name: string
}

export interface ColorLabel {
    name: string
    hex: string
}

// Note – the meta fields are flattened, so no separate meta object
export interface Note {
    id: string
    created_at: number
    updated_at: number
    tags: Tag[]
    color: ColorLabel | null
    is_favorite: boolean
    trash_status: 'Active' | 'Trashed' | 'Deleted'
    position: number
    title: string
    content: string
    is_pinned: boolean
}

// You’ll extend these for the other vaults later (same pattern)
export interface ClipboardItem {
    id: string
    created_at: number
    updated_at: number
    tags: Tag[]
    color: ColorLabel | null
    is_favorite: boolean
    trash_status: 'Active' | 'Trashed' | 'Deleted'
    position: number
    content: string
    persist_to_disk: boolean
}

export interface Todo {
    id: string
    created_at: number
    updated_at: number
    tags: Tag[]
    color: ColorLabel | null
    is_favorite: boolean
    trash_status: 'Active' | 'Trashed' | 'Deleted'
    position: number
    title: string
    description: string
    completed: boolean
    due_date: number | null
}

export interface Bookmark {
    id: string
    created_at: number
    updated_at: number
    tags: Tag[]
    color: ColorLabel | null
    is_favorite: boolean
    trash_status: 'Active' | 'Trashed' | 'Deleted'
    position: number
    url: string
    title: string
    description: string
    favicon: number[] | null
    thumbnail: number[] | null
}

export interface Contact {
    id: string
    created_at: number
    updated_at: number
    tags: Tag[]
    color: ColorLabel | null
    is_favorite: boolean
    trash_status: 'Active' | 'Trashed' | 'Deleted'
    position: number
    name: string
    phones: string[]
    emails: string[]
    addresses: string[]
    notes: string
}