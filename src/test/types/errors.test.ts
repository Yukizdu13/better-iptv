import { describe, it, expect } from 'vitest';
import {
  isAppError,
  getErrorMessage,
  getErrorSeverity,
  parseError,
  type AppError,
} from '../../types/errors';

describe('Error Types', () => {
  describe('isAppError', () => {
    it('should return true for valid AppError objects', () => {
      const error: AppError = {
        code: 'Database',
        details: 'Connection failed',
      };
      expect(isAppError(error)).toBe(true);
    });

    it('should return false for regular Error objects', () => {
      const error = new Error('Something went wrong');
      expect(isAppError(error)).toBe(false);
    });

    it('should return false for strings', () => {
      expect(isAppError('error message')).toBe(false);
    });

    it('should return false for null/undefined', () => {
      expect(isAppError(null)).toBe(false);
      expect(isAppError(undefined)).toBe(false);
    });

    it('should return false for objects missing required fields', () => {
      expect(isAppError({ code: 'Database' })).toBe(false);
      expect(isAppError({ details: 'error' })).toBe(false);
    });
  });

  describe('getErrorMessage', () => {
    it('should return Swedish message for Database error', () => {
      const error: AppError = { code: 'Database', details: 'Connection failed' };
      expect(getErrorMessage(error)).toBe('Databasfel. Försök igen senare.');
    });

    it('should return Swedish message for Http error', () => {
      const error: AppError = { code: 'Http', details: 'timeout' };
      expect(getErrorMessage(error)).toBe('Nätverksfel. Kontrollera din anslutning.');
    });

    it('should return details for InvalidInput error', () => {
      const error: AppError = { code: 'InvalidInput', details: 'Name cannot be empty' };
      expect(getErrorMessage(error)).toBe('Name cannot be empty');
    });

    it('should return Swedish message for Mpv error with details', () => {
      const error: AppError = { code: 'Mpv', details: 'Failed to start' };
      expect(getErrorMessage(error)).toBe('Uppspelningsfel: Failed to start');
    });

    it('should return Swedish message for Parse error', () => {
      const error: AppError = { code: 'Parse', details: 'Invalid XML' };
      expect(getErrorMessage(error)).toBe('Kunde inte läsa filen. Kontrollera formatet.');
    });

    it('should return Swedish message for Epg error', () => {
      const error: AppError = { code: 'Epg', details: 'fetch failed' };
      expect(getErrorMessage(error)).toBe('Kunde inte hämta programguiden.');
    });

    it('should return Swedish message for PlaylistNotFound', () => {
      const error: AppError = { code: 'PlaylistNotFound', details: '42' };
      expect(getErrorMessage(error)).toBe('Spellistan kunde inte hittas.');
    });

    it('should return Swedish message for ChannelNotFound', () => {
      const error: AppError = { code: 'ChannelNotFound', details: '123' };
      expect(getErrorMessage(error)).toBe('Kanalen kunde inte hittas.');
    });
  });

  describe('getErrorSeverity', () => {
    it('should return error for Database', () => {
      const error: AppError = { code: 'Database', details: 'error' };
      expect(getErrorSeverity(error)).toBe('error');
    });

    it('should return error for Io', () => {
      const error: AppError = { code: 'Io', details: 'error' };
      expect(getErrorSeverity(error)).toBe('error');
    });

    it('should return warning for Http', () => {
      const error: AppError = { code: 'Http', details: 'error' };
      expect(getErrorSeverity(error)).toBe('warning');
    });

    it('should return warning for Epg', () => {
      const error: AppError = { code: 'Epg', details: 'error' };
      expect(getErrorSeverity(error)).toBe('warning');
    });

    it('should return info for InvalidInput', () => {
      const error: AppError = { code: 'InvalidInput', details: 'error' };
      expect(getErrorSeverity(error)).toBe('info');
    });

    it('should return info for Mpv', () => {
      const error: AppError = { code: 'Mpv', details: 'error' };
      expect(getErrorSeverity(error)).toBe('info');
    });
  });

  describe('parseError', () => {
    it('should parse AppError objects', () => {
      const error: AppError = { code: 'Database', details: 'Connection failed' };
      expect(parseError(error)).toBe('Databasfel. Försök igen senare.');
    });

    it('should parse Error objects', () => {
      const error = new Error('Something went wrong');
      expect(parseError(error)).toBe('Something went wrong');
    });

    it('should parse strings', () => {
      expect(parseError('error message')).toBe('error message');
    });

    it('should return default message for unknown types', () => {
      expect(parseError({})).toBe('Ett oväntat fel inträffade.');
      expect(parseError(123)).toBe('Ett oväntat fel inträffade.');
    });
  });
});
