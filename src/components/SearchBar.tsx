import { forwardRef } from 'react';
import { Search } from 'lucide-react';

interface SearchBarProps {
  /** Current search query */
  value: string;
  /** Callback when search query changes */
  onChange: (value: string) => void;
  /** Placeholder text */
  placeholder?: string;
}

/**
 * Search bar component for filtering channels
 *
 * Features:
 * - Search icon prefix
 * - Responsive width
 * - Dark mode support
 * - Supports ref forwarding for keyboard shortcut focus
 */
export const SearchBar = forwardRef<HTMLInputElement, SearchBarProps>(
  function SearchBar({ value, onChange, placeholder = 'Search channels...' }, ref) {
    return (
      <div className="border-b border-gray-200 bg-white p-4 dark:border-gray-700 dark:bg-gray-800">
        <div className="mx-auto px-2">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-5 w-5 -translate-y-1/2 transform text-gray-400" />
            <input
              ref={ref}
              type="text"
              value={value}
              onChange={(e) => onChange(e.target.value)}
              placeholder={placeholder}
              className="w-full rounded-lg border border-gray-300 py-2 pl-10 pr-4 focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
            />
          </div>
        </div>
      </div>
    );
  }
);

export default SearchBar;
