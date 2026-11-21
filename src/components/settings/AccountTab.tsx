import { useEffect, useState } from 'react';
import {
  defaultAccountFilePath,
  exportAccount,
  importAccount,
  type AccountExportReport,
  type AccountMergeReport,
} from '../../lib/tauri';
import {
  MIN_PASSPHRASE_CHARS,
  describeError,
  isPassphraseLongEnough,
  passphraseLength,
} from '../../lib/accountPassphrase';
import { logger } from '../../lib/logger';

const INPUT_CLASS =
  'w-full rounded-md border border-gray-300 bg-white px-2 py-1 text-sm focus:border-transparent focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white';
const BUTTON_CLASS =
  'rounded-md bg-blue-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50';

export default function AccountTab() {
  const [path, setPath] = useState('');
  const [passphrase, setPassphrase] = useState('');
  const [confirm, setConfirm] = useState('');
  const [importPassphrase, setImportPassphrase] = useState('');
  const [busy, setBusy] = useState<'export' | 'import' | null>(null);
  const [exportResult, setExportResult] = useState<AccountExportReport | null>(null);
  const [importResult, setImportResult] = useState<AccountMergeReport | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    defaultAccountFilePath()
      .then(setPath)
      .catch((err) => logger.error('Failed to get default account path:', err));
  }, []);

  const length = passphraseLength(passphrase);
  const canExport =
    busy === null &&
    path.trim() !== '' &&
    isPassphraseLongEnough(passphrase) &&
    passphrase === confirm;
  const canImport = busy === null && path.trim() !== '' && importPassphrase !== '';

  async function handleExport() {
    setBusy('export');
    setError('');
    setExportResult(null);
    setImportResult(null);
    try {
      const report = await exportAccount(path.trim(), passphrase);
      setExportResult(report);
      setPassphrase('');
      setConfirm('');
    } catch (err) {
      logger.error('Account export failed');
      setError(describeError(err));
    } finally {
      setBusy(null);
    }
  }

  async function handleImport() {
    setBusy('import');
    setError('');
    setExportResult(null);
    setImportResult(null);
    try {
      const report = await importAccount(path.trim(), importPassphrase);
      setImportResult(report);
      setImportPassphrase('');
    } catch (err) {
      logger.error('Account import failed');
      setError(describeError(err));
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="space-y-8">
      <section className="space-y-3">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Fichier de compte</h3>
        <p className="text-xs text-gray-500 dark:text-gray-400">
          Exporte tes profils (avec leurs identifiants), favoris, séries en cours, presets de play
          aléatoire et réglages dans un fichier chiffré, pour les retrouver sur une autre machine
          (Linux, Windows). Le fichier est protégé par ta phrase secrète : sans elle, il est
          illisible et irrécupérable.
        </p>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Chemin du fichier
          <input
            type="text"
            value={path}
            onChange={(e) => setPath(e.target.value)}
            className={`${INPUT_CLASS} mt-1`}
            spellCheck={false}
          />
        </label>
      </section>

      <section className="space-y-3">
        <h4 className="text-base font-semibold text-gray-900 dark:text-white">Exporter</h4>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Phrase secrète (au moins {MIN_PASSPHRASE_CHARS} caractères)
          <input
            type="password"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            className={`${INPUT_CLASS} mt-1`}
            autoComplete="new-password"
          />
        </label>
        <p
          className={`text-xs ${
            isPassphraseLongEnough(passphrase) ? 'text-green-600' : 'text-gray-500'
          } dark:text-gray-400`}
        >
          {length} / {MIN_PASSPHRASE_CHARS} caractères
        </p>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Confirmer la phrase secrète
          <input
            type="password"
            value={confirm}
            onChange={(e) => setConfirm(e.target.value)}
            className={`${INPUT_CLASS} mt-1`}
            autoComplete="new-password"
          />
        </label>
        {confirm !== '' && passphrase !== confirm && (
          <p className="text-xs text-red-600">Les deux phrases ne correspondent pas.</p>
        )}
        <button type="button" className={BUTTON_CLASS} disabled={!canExport} onClick={handleExport}>
          {busy === 'export' ? 'Export en cours…' : 'Exporter le compte'}
        </button>
        {exportResult && (
          <p className="text-sm text-green-700 dark:text-green-400">
            Compte exporté dans {exportResult.path} : {exportResult.profiles} profil(s),{' '}
            {exportResult.favorites} favori(s), {exportResult.series_progress} série(s) en cours,{' '}
            {exportResult.presets} preset(s), {exportResult.settings} réglage(s).
          </p>
        )}
      </section>

      <section className="space-y-3">
        <h4 className="text-base font-semibold text-gray-900 dark:text-white">Importer</h4>
        <p className="text-xs text-gray-500 dark:text-gray-400">
          L&apos;import fusionne le fichier avec cet appareil : pour chaque élément, la version la
          plus récente gagne. Rien n&apos;est supprimé, sauf les profils supprimés depuis sur
          l&apos;autre machine. Les chaînes ne sont pas dans le fichier : recharge ensuite chaque
          liste (favoris et séries en cours se rattachent alors automatiquement).
        </p>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Phrase secrète du fichier
          <input
            type="password"
            value={importPassphrase}
            onChange={(e) => setImportPassphrase(e.target.value)}
            className={`${INPUT_CLASS} mt-1`}
            autoComplete="off"
          />
        </label>
        <button type="button" className={BUTTON_CLASS} disabled={!canImport} onClick={handleImport}>
          {busy === 'import' ? 'Import en cours…' : 'Importer le compte'}
        </button>
        {importResult && (
          <div className="space-y-2 text-sm text-green-700 dark:text-green-400">
            <p>
              Import terminé : {importResult.profiles_added} profil(s) ajouté(s),{' '}
              {importResult.profiles_updated} mis à jour, {importResult.profiles_deleted}{' '}
              supprimé(s) ; {importResult.favorites_applied} favori(s),{' '}
              {importResult.progress_applied} série(s) en cours, {importResult.presets_applied}{' '}
              preset(s), {importResult.settings_applied} réglage(s) appliqués.
              {importResult.pending > 0 &&
                ` ${importResult.pending} élément(s) en attente du chargement des listes.`}
            </p>
            <button type="button" className={BUTTON_CLASS} onClick={() => window.location.reload()}>
              Recharger l&apos;application
            </button>
          </div>
        )}
      </section>

      {error && (
        <p role="alert" className="text-sm text-red-600">
          {error}
        </p>
      )}
    </div>
  );
}
