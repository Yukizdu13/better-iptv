import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock('../../lib/logger', () => ({
  logger: { debug: vi.fn(), info: vi.fn(), warn: vi.fn(), error: vi.fn() },
}));
vi.mock('../../lib/platform', () => ({ isIOS: vi.fn(() => false) }));

import Setup from '../../components/Setup';
import { isIOS } from '../../lib/platform';

describe('Setup — onglet Compte', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockResolvedValue('/home/test/Documents/better-iptv-compte.biptv');
    vi.mocked(isIOS).mockReturnValue(false);
  });

  it('propose trois onglets sur la page d’accueil', () => {
    render(<Setup />);
    expect(screen.getByRole('button', { name: 'M3U URL' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Xtream Codes' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Compte' })).toBeInTheDocument();
  });

  it('affiche uniquement l’import dans l’onglet Compte', () => {
    render(<Setup />);
    fireEvent.click(screen.getByRole('button', { name: 'Compte' }));
    expect(screen.getByRole('button', { name: 'Importer le compte' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Exporter le compte' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Add Playlist' })).not.toBeInTheDocument();
  });

  it('masque l’onglet Compte en mode modale et sur iOS', () => {
    const { unmount } = render(<Setup onCancel={() => {}} />);
    expect(screen.queryByRole('button', { name: 'Compte' })).not.toBeInTheDocument();
    unmount();
    vi.mocked(isIOS).mockReturnValue(true);
    render(<Setup />);
    expect(screen.queryByRole('button', { name: 'Compte' })).not.toBeInTheDocument();
  });
});
