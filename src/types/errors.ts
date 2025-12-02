/**
 * Application error types matching the Rust backend AppError enum
 */

/**
 * Error codes matching the Rust AppError variants
 */
export type AppErrorCode =
  | 'Database'
  | 'Http'
  | 'InvalidInput'
  | 'PlaylistNotFound'
  | 'ChannelNotFound'
  | 'Mpv'
  | 'Parse'
  | 'Epg'
  | 'Io'
  | 'Config';

/**
 * Application error structure from Tauri backend
 * This matches the serialized format of the Rust AppError enum
 */
export interface AppError {
  code: AppErrorCode;
  details: string;
}

/**
 * Type guard to check if an error is an AppError
 */
export function isAppError(e: unknown): e is AppError {
  return (
    typeof e === 'object' &&
    e !== null &&
    'code' in e &&
    'details' in e &&
    typeof (e as AppError).code === 'string' &&
    typeof (e as AppError).details === 'string'
  );
}

/**
 * Get a user-friendly error message based on error code
 */
export function getErrorMessage(error: AppError): string {
  switch (error.code) {
    case 'Database':
      return 'Databasfel. Försök igen senare.';
    case 'Http':
      return 'Nätverksfel. Kontrollera din anslutning.';
    case 'InvalidInput':
      return error.details || 'Ogiltig indata.';
    case 'PlaylistNotFound':
      return 'Spellistan kunde inte hittas.';
    case 'ChannelNotFound':
      return 'Kanalen kunde inte hittas.';
    case 'Mpv':
      return `Uppspelningsfel: ${error.details}`;
    case 'Parse':
      return 'Kunde inte läsa filen. Kontrollera formatet.';
    case 'Epg':
      return 'Kunde inte hämta programguiden.';
    case 'Io':
      return 'Filfel. Kontrollera behörigheter.';
    case 'Config':
      return 'Konfigurationsfel.';
    default:
      return 'Ett oväntat fel inträffade.';
  }
}

/**
 * Get error severity level for UI styling
 */
export function getErrorSeverity(error: AppError): 'error' | 'warning' | 'info' {
  switch (error.code) {
    case 'Database':
    case 'Io':
    case 'Config':
      return 'error';
    case 'Http':
    case 'Epg':
    case 'Parse':
      return 'warning';
    case 'InvalidInput':
    case 'PlaylistNotFound':
    case 'ChannelNotFound':
    case 'Mpv':
      return 'info';
    default:
      return 'error';
  }
}

/**
 * Parse an unknown error into a user-friendly message
 */
export function parseError(error: unknown): string {
  if (isAppError(error)) {
    return getErrorMessage(error);
  }

  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'string') {
    return error;
  }

  return 'Ett oväntat fel inträffade.';
}
