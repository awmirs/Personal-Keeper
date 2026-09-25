/**
 * Calculates a new fractional position when moving an item within a sorted list.
 * Avoids position collision and requires mutating only the target item.
 */
export function calculateFractionalPosition<T extends { position: number }>(
    items: T[],
    currentIndex: number,
    direction: 'up' | 'down'
): number | null {
    const targetIndex = direction === 'up' ? currentIndex - 1 : currentIndex + 1
    if (targetIndex < 0 || targetIndex >= items.length) return null

    if (direction === 'up') {
        if (targetIndex === 0) {
            return items[0].position - 1000.0
        }
        const prev = items[targetIndex - 1].position
        const next = items[targetIndex].position
        if (Math.abs(next - prev) < 1e-9) {
            return prev - 1.0
        }
        return (prev + next) / 2.0
    } else {
        if (targetIndex === items.length - 1) {
            return items[items.length - 1].position + 1000.0
        }
        const prev = items[targetIndex].position
        const next = items[targetIndex + 1].position
        if (Math.abs(next - prev) < 1e-9) {
            return next + 1.0
        }
        return (prev + next) / 2.0
    }
}
