// src/components/views/CompactListView.tsx
import React from 'react'

interface CompactListViewProps<T> {
    items: T[]
    renderItem: (item: T, index: number) => React.ReactNode
    className?: string
}

export default function CompactListView<T>({ items, renderItem, className = '' }: CompactListViewProps<T>) {
    return (
        <div className={`space-y-2 ${className}`}>
            {items.map((item, idx) => (
                <div key={idx}>{renderItem(item, idx)}</div>
            ))}
        </div>
    )
}