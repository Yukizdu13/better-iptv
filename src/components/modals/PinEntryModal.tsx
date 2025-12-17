import { useState, useEffect } from 'react';
import { X } from 'lucide-react';
import { setParentalPin, verifyParentalPin } from '../../lib/tauri';
import { validatePin } from '../../lib/parentalControls';
import { logger } from '../../lib/logger';

interface PinEntryModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  mode: 'set' | 'change' | 'verify';
  title?: string;
}

export default function PinEntryModal({
  isOpen,
  onClose,
  onSuccess,
  mode,
  title,
}: PinEntryModalProps) {
  const [currentPin, setCurrentPin] = useState('');
  const [newPin, setNewPin] = useState('');
  const [confirmPin, setConfirmPin] = useState('');
  const [error, setError] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Reset form when modal opens
  useEffect(() => {
    if (isOpen) {
      setCurrentPin('');
      setNewPin('');
      setConfirmPin('');
      setError('');
      setIsSubmitting(false);
    }
  }, [isOpen]);

  const handleSubmit = async () => {
    setError('');
    setIsSubmitting(true);

    try {
      if (mode === 'verify') {
        // Verify existing PIN
        const isValid = await verifyParentalPin(currentPin);
        if (!isValid) {
          logger.warn('PIN verification failed in unlock modal');
          setError('Incorrect PIN');
          setIsSubmitting(false);
          return;
        }
        logger.info('PIN verified successfully in unlock modal');
        onSuccess();
        onClose();
      } else {
        // Set or change PIN
        if (mode === 'change') {
          // Verify current PIN first
          const isValid = await verifyParentalPin(currentPin);
          if (!isValid) {
            logger.warn('Current PIN verification failed when attempting to change PIN');
            setError('Incorrect current PIN');
            setIsSubmitting(false);
            return;
          }
          logger.info('Current PIN verified successfully for PIN change');
        }

        // Validate new PIN
        const validation = validatePin(newPin);
        if (!validation.valid) {
          setError(validation.error || 'Invalid PIN');
          setIsSubmitting(false);
          return;
        }

        // Check if PINs match
        if (newPin !== confirmPin) {
          setError('PINs do not match');
          setIsSubmitting(false);
          return;
        }

        // Set the new PIN
        await setParentalPin(newPin);
        onSuccess();
        onClose();
        resetForm();
      }
    } catch (err) {
      setError(`Failed: ${err}`);
      setIsSubmitting(false);
    }
  };

  const resetForm = () => {
    setCurrentPin('');
    setNewPin('');
    setConfirmPin('');
    setError('');
    setIsSubmitting(false);
  };

  const handleClose = () => {
    resetForm();
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl dark:bg-gray-800">
        {/* Header */}
        <div className="mb-4 flex items-center justify-between">
          <h3 className="text-xl font-bold text-gray-900 dark:text-white">
            {title || (mode === 'set' ? 'Set PIN' : mode === 'change' ? 'Change PIN' : 'Enter PIN')}
          </h3>
          <button
            onClick={handleClose}
            className="rounded-lg p-1 hover:bg-gray-100 dark:hover:bg-gray-700"
          >
            <X className="h-5 w-5 text-gray-500 dark:text-gray-400" />
          </button>
        </div>

        {/* Form */}
        <div className="space-y-4">
          {(mode === 'change' || mode === 'verify') && (
            <div>
              <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                {mode === 'change' ? 'Current PIN' : 'PIN'}
              </label>
              <input
                type="password"
                inputMode="numeric"
                value={currentPin}
                onChange={(e) => setCurrentPin(e.target.value.replace(/\D/g, '').slice(0, 6))}
                placeholder={mode === 'verify' ? 'Enter PIN' : 'Current PIN'}
                className="w-full rounded-lg border border-gray-300 px-4 py-2 text-center text-2xl tracking-widest focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                maxLength={6}
                autoFocus
              />
            </div>
          )}

          {mode !== 'verify' && (
            <>
              <div>
                <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                  {mode === 'set' ? 'PIN' : 'New PIN'} (4-6 digits)
                </label>
                <input
                  type="password"
                  inputMode="numeric"
                  value={newPin}
                  onChange={(e) => setNewPin(e.target.value.replace(/\D/g, '').slice(0, 6))}
                  placeholder="Enter new PIN"
                  className="w-full rounded-lg border border-gray-300 px-4 py-2 text-center text-2xl tracking-widest focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                  maxLength={6}
                  autoFocus={mode === 'set'}
                />
              </div>

              <div>
                <label className="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Confirm PIN
                </label>
                <input
                  type="password"
                  inputMode="numeric"
                  value={confirmPin}
                  onChange={(e) => setConfirmPin(e.target.value.replace(/\D/g, '').slice(0, 6))}
                  placeholder="Confirm PIN"
                  className="w-full rounded-lg border border-gray-300 px-4 py-2 text-center text-2xl tracking-widest focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                  maxLength={6}
                />
              </div>
            </>
          )}

          {error && (
            <div className="rounded-lg bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400">
              {error}
            </div>
          )}

          <div className="text-xs text-gray-500 dark:text-gray-400">
            {mode !== 'verify' && 'PIN must be 4-6 digits containing only numbers.'}
          </div>
        </div>

        {/* Actions */}
        <div className="mt-6 flex justify-end gap-3">
          <button
            onClick={handleClose}
            className="rounded-lg bg-gray-500 px-4 py-2 text-white hover:bg-gray-600 disabled:opacity-50"
            disabled={isSubmitting}
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            className="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 disabled:opacity-50"
            disabled={
              isSubmitting ||
              (mode === 'verify' && !currentPin) ||
              (mode !== 'verify' && (!newPin || !confirmPin)) ||
              (mode === 'change' && !currentPin)
            }
          >
            {isSubmitting
              ? 'Processing...'
              : mode === 'verify'
                ? 'Unlock'
                : mode === 'set'
                  ? 'Set PIN'
                  : 'Change PIN'}
          </button>
        </div>
      </div>
    </div>
  );
}
