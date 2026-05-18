// src/components/views/ListView.tsx
import React from 'react'

interface ListViewProps<T> {
    items: T[]
    renderItem: (item: T, index: number) => React.ReactNode
    className?: string
}

export default function ListView<T>({ items, renderItem, className = '' }: ListViewProps<T>) {
    return (
        <div className={`space-y-4 ${className}`}>
            {items.map((item, idx) => (
                <div key={idx}>{renderItem(item, idx)}</div>
            ))}
        </div>
    )
}