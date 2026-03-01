import { useState, useEffect } from 'react';
import { getVersion } from '@tauri-apps/api/app';
import { appLogDir } from '@tauri-apps/api/path';
import { openPath, openUrl } from '@tauri-apps/plugin-opener';
import { Copy, Check, FolderOpen } from 'lucide-react';
import { truncateAddress } from '../../lib/truncateAddress';
import { logger } from '../../lib/logger';
import logoImage from '../../assets/logo/logo-256.webp';

const CRYPTO_ADDRESSES = [
  { currency: 'BTC', address: 'bc1qth40h9t8r7hvp4czqvf20f3w72jdg4epd5mjq8' },
  { currency: 'ETH', address: '0x47183F4e4FEAeE4BF52d95E68893e950125b1B44' },
  { currency: 'SOL', address: '3waxf6r2tmaaADuBGYoVD5qz4z8VnFNEGGafbXZ6Jf2j' },
] as const;

export default function AboutTab() {
  const [version, setVersion] = useState('');
  const [cryptoOpen, setCryptoOpen] = useState(false);
  const [copiedCurrency, setCopiedCurrency] = useState<string | null>(null);

  useEffect(() => {
    getVersion().then(setVersion).catch((err) => {
      logger.warn('Failed to get app version:', err);
    });
  }, []);

  const handleCopy = async (currency: string, address: string) => {
    try {
      await navigator.clipboard.writeText(address);
      setCopiedCurrency(currency);
      setTimeout(() => setCopiedCurrency(null), 2000);
    } catch {
      // Clipboard write failed silently
    }
  };

  const handleOpenLogs = async () => {
    try {
      const dir = await appLogDir();
      await openPath(dir);
    } catch (err) {
      logger.error('Failed to open logs folder:', err);
    }
  };

  return (
    <div className="space-y-6 py-4">
      {/* App identity */}
      <div className="flex items-center gap-4">
        <img src={logoImage} alt="Better IPTV" className="h-12 w-12 rounded-xl" />
        <div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Better IPTV</h3>
          <div className="flex items-center gap-2 mt-0.5">
            {version && (
              <>
                <span className="text-sm text-gray-500 dark:text-gray-400">v{version}</span>
                <span className="text-xs text-gray-400 dark:text-gray-500">·</span>
              </>
            )}
            <span className="text-xs text-gray-400 dark:text-gray-500">GPL-2.0</span>
          </div>
        </div>
      </div>

      {/* Support copy */}
      <p className="text-sm text-gray-600 dark:text-gray-400 text-pretty leading-relaxed">
        Better IPTV is built and maintained by a single developer in their spare time — no ads, no
        subscriptions, no commercial backing. If you find it useful, consider supporting its
        development.
      </p>

      {/* Donation buttons */}
      <div className="flex flex-wrap gap-3">
        <button
          onClick={() => { openUrl('https://ko-fi.com/R6R21I53PD').catch((err) => logger.error('Failed to open Ko-fi URL:', err)); }}
          className="inline-flex items-center gap-2 rounded-lg bg-[#FF5E5B] px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-[#e54f4d]"
        >
          <svg className="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M23.881 8.948c-.773-4.085-4.859-4.593-4.859-4.593H.723c-.604 0-.679.798-.679.798s-.082 7.324-.022 11.822c.164 2.424 2.586 2.672 2.586 2.672s8.267-.023 11.966-.049c2.438-.426 2.683-2.566 2.658-3.734 4.352.24 7.422-2.831 6.649-6.916zm-11.062 3.511c-1.246 1.453-4.011 3.976-4.011 3.976s-.121.119-.31.023c-.076-.057-.108-.09-.108-.09-.443-.441-3.368-3.049-4.034-3.954-.709-.965-1.041-2.7-.091-3.71.951-1.01 3.005-1.086 4.363.407 0 0 1.565-1.494 3.468-.947 1.904.547 1.832 2.51.723 4.295zm6.173.478c-.928.116-1.682.028-1.682.028V7.284h1.77s1.971.551 1.971 2.638c0 1.913-.985 2.667-2.059 3.015z" />
          </svg>
          Support on Ko-fi
        </button>
        <button
          onClick={() => { openUrl('https://github.com/sponsors/mewset').catch((err) => logger.error('Failed to open GitHub Sponsors URL:', err)); }}
          className="inline-flex items-center gap-2 rounded-lg bg-gray-900 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-gray-700 dark:bg-gray-700 dark:hover:bg-gray-600"
        >
          <svg className="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
            <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
          </svg>
          GitHub Sponsors
        </button>
      </div>

      {/* Crypto accordion */}
      <div className="rounded-lg border border-gray-200 overflow-hidden dark:border-gray-700">
        <button
          onClick={() => setCryptoOpen((o) => !o)}
          aria-expanded={cryptoOpen}
          aria-controls="crypto-addresses"
          className="flex w-full items-center justify-between px-4 py-3 text-sm font-medium text-gray-600 transition-colors hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-200"
        >
          Donate with crypto
          <svg
            className={`h-4 w-4 transition-transform ${cryptoOpen ? 'rotate-180' : ''}`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </button>
        {cryptoOpen && (
          <div id="crypto-addresses" className="border-t border-gray-200 px-4 pb-4 pt-3 space-y-3 dark:border-gray-700">
            {CRYPTO_ADDRESSES.map(({ currency, address }) => (
              <div key={currency}>
                <p className="mb-1 text-xs font-semibold uppercase text-gray-500 dark:text-gray-400">
                  {currency}
                </p>
                <div className="flex items-center gap-2 rounded-lg border border-gray-100 bg-gray-50 px-3 py-2 dark:border-gray-600 dark:bg-gray-700">
                  <span className="flex-1 font-mono text-xs text-gray-700 dark:text-gray-300">
                    {truncateAddress(address)}
                  </span>
                  <button
                    onClick={() => handleCopy(currency, address)}
                    aria-label={`Copy ${currency} address`}
                    className="flex-shrink-0 text-gray-400 transition-colors hover:text-blue-600 dark:hover:text-blue-400"
                  >
                    {copiedCurrency === currency ? (
                      <Check className="h-4 w-4 text-green-500" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Open logs folder */}
      <div className="border-t border-gray-200 pt-4 dark:border-gray-700">
        <button
          onClick={handleOpenLogs}
          className="inline-flex items-center gap-2 rounded-lg px-3 py-2 text-sm text-gray-600 transition-colors hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700"
        >
          <FolderOpen className="h-4 w-4" />
          Open logs folder
        </button>
      </div>
    </div>
  );
}
