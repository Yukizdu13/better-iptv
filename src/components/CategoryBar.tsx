import { usePlayerStore } from '../stores/player-store';

/**
 * Horizontal scrollable bar showing provider categories (Sweden, Norway, F1, etc.)
 * Allows quick filtering of channels by category.
 */
export function CategoryBar() {
  const { categories, categoryFilter, setCategoryFilter } = usePlayerStore();

  // Don't render if no categories available
  if (categories.length === 0) return null;

  return (
    <div
      className="flex gap-2 overflow-x-auto px-4 py-3 pb-6 bg-gray-800/50 scrollbar-hide"
      role="tablist"
      aria-label="Channel categories"
    >
      {/* "All" chip - shows all channels in current content type */}
      <button
        onClick={() => setCategoryFilter(null)}
        role="tab"
        aria-selected={categoryFilter === null}
        className={`
          shrink-0 px-3 py-1.5 rounded-full text-sm font-medium transition-colors
          focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900
          ${
            categoryFilter === null
              ? 'bg-blue-600 text-white'
              : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
          }
        `}
      >
        All
      </button>

      {/* Category chips from provider */}
      {categories.map((category) => (
        <button
          key={category}
          onClick={() => setCategoryFilter(category)}
          role="tab"
          aria-selected={categoryFilter === category}
          className={`
            shrink-0 px-3 py-1.5 rounded-full text-sm font-medium transition-colors
            whitespace-nowrap
            focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900
            ${
              categoryFilter === category
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }
          `}
        >
          {category}
        </button>
      ))}
    </div>
  );
}
