import { describe, it, expect } from 'vitest';
import {
  MIN_PASSPHRASE_CHARS,
  describeError,
  isPassphraseLongEnough,
  passphraseLength,
} from './accountPassphrase';

describe('accountPassphrase', () => {
  it('requires at least 25 characters', () => {
    expect(MIN_PASSPHRASE_CHARS).toBe(25);
    expect(isPassphraseLongEnough('a'.repeat(24))).toBe(false);
    expect(isPassphraseLongEnough('a'.repeat(25))).toBe(true);
  });

  it('counts code points like the backend (accents, emoji)', () => {
    expect(passphraseLength('é'.repeat(25))).toBe(25);
    expect(passphraseLength('😀'.repeat(25))).toBe(25);
    expect(isPassphraseLongEnough('😀'.repeat(24))).toBe(false);
  });

  it('describes Tauri AppError values', () => {
    expect(describeError({ code: 'InvalidInput', details: 'Phrase incorrecte' })).toBe(
      'Phrase incorrecte'
    );
    expect(describeError(new Error('boom'))).toBe('boom');
    expect(describeError('texte')).toBe('texte');
    expect(describeError(null)).toBe('Erreur inconnue');
  });
});
