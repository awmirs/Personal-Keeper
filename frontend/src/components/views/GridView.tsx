// src/components/views/GridView.tsx
import React from 'react'

interface GridViewProps<T> {
    items: T[]
    renderItem: (item: T, index: number) => React.ReactNode
    className?: string
}

export default function GridView<T>({ items, renderItem, className = '' }: GridViewProps<T>) {
    return (
        <div className={`grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 ${className}`}>
            {items.map((item, idx) => (
                <div key={idx}>{renderItem(item, idx)}</div>
            ))}
        </div>
    )
}