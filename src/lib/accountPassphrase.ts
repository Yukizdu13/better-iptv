/** Minimum length of an account passphrase, in characters (same rule as the Rust backend). */
export const MIN_PASSPHRASE_CHARS = 25;

/** Counts characters like Rust's `chars().count()` (code points, not UTF-16 units). */
export function passphraseLength(passphrase: string): number {
  return Array.from(passphrase).length;
}

export function isPassphraseLongEnough(passphrase: string): boolean {
  return passphraseLength(passphrase) >= MIN_PASSPHRASE_CHARS;
}

/** Readable message from a Tauri `AppError` ({ code, details }) or any other thrown value. */
export function describeError(err: unknown): string {
  if (typeof err === 'string') return err;
  if (err && typeof err === 'object') {
    const details = (err as { details?: unknown }).details;
    if (typeof details === 'string' && details) return details;
    const message = (err as { message?: unknown }).message;
    if (typeof message === 'string' && message) return message;
  }
  return 'Erreur inconnue';
}
