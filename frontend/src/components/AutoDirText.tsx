/**
 * Renders plain text with per‑line direction auto‑detection.
 * Each line becomes a <div dir="auto"> – perfect for Clipboard, Todos, Contacts, etc.
 */
export default function AutoDirText({ text, className = '', as: Tag = 'div' }: {
    text: string
    className?: string
    as?: 'div' | 'span' | 'p'
}) {
    const lines = text.split('\n')
    return (
        <>
            {lines.map((line, i) => (
                <Tag key={i} dir="auto" className={className}>
                    {line}
                </Tag>
            ))}
        </>
    )
}