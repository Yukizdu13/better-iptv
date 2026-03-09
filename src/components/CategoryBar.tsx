import { memo } from 'react';
import { usePlayerStore } from '../stores/player-store';

/**
 * Horizontal scrollable bar showing provider categories (Sweden, Norway, F1, etc.)
 * Allows quick filtering of channels by category.
 */
export const CategoryBar = memo(function CategoryBar() {
  const categories = usePlayerStore((s) => s.categories);
  const categoryFilter = usePlayerStore((s) => s.categoryFilter);
  const setCategoryFilter = usePlayerStore((s) => s.setCategoryFilter);

  // Don't render if no categories available
  if (categories.length === 0) return null;

  return (
    <div
      className="scrollbar-hide flex gap-2 overflow-x-auto bg-gray-800/50 px-4 py-3 pb-6"
      role="tablist"
      aria-label="Channel categories"
    >
      {/* "All" chip - shows all channels in current content type */}
      <button
        onClick={() => setCategoryFilter(null)}
        role="tab"
        aria-selected={categoryFilter === null}
        className={`shrink-0 rounded-full px-3 py-1.5 text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900 ${
          categoryFilter === null
            ? 'bg-blue-600 text-white'
            : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
        } `}
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
          className={`shrink-0 whitespace-nowrap rounded-full px-3 py-1.5 text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900 ${
            categoryFilter === category
              ? 'bg-blue-600 text-white'
              : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
          } `}
        >
          {category}
        </button>
      ))}
    </div>
  );
});
