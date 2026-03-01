/**
 * Truncates a crypto wallet address for display.
 * Addresses longer than 13 chars are shown as `first6...last4`.
 * Shorter addresses are returned unchanged.
 */
export function truncateAddress(address: string): string {
  if (address.length <= 13) return address;
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}
