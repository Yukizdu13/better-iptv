import { describe, it, expect } from 'vitest';
import { truncateAddress } from '../../lib/truncateAddress';

describe('truncateAddress', () => {
  it('returns first 6 chars + ... + last 4 chars for long addresses', () => {
    expect(truncateAddress('bc1qth40h9t8r7hvp4czqvf20f3w72jdg4epd5mjq8')).toBe('bc1qth...mjq8');
  });

  it('returns address unchanged if 13 chars or fewer', () => {
    expect(truncateAddress('shortaddr123')).toBe('shortaddr123');
  });

  it('handles empty string', () => {
    expect(truncateAddress('')).toBe('');
  });
});
