import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Lock, X, Eye, EyeOff, ShieldAlert } from 'lucide-react';
import { invoke } from '../../lib/invoke';
import { useAuth } from '../../contexts/AuthContext';

interface ChangePasswordModalProps {
  onClose: () => void;
}

export function ChangePasswordModal({ onClose }: ChangePasswordModalProps) {
  const { t } = useTranslation();
  const { user } = useAuth();
  const [newPassword, setNewPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const [showNew, setShowNew] = useState(false);
  const [error, setError] = useState('');
  const [saving, setSaving] = useState(false);
  const [done, setDone] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (newPassword.length < 8) { setError('Hasło musi mieć co najmniej 8 znaków.'); return; }
    if (newPassword !== confirm) { setError('Hasła nie są zgodne.'); return; }
    if (!user) return;
    setSaving(true);
    try {
      await invoke('reset_password', { user_id: user.id, new_password: newPassword });
      setDone(true);
      setTimeout(onClose, 1500);
    } catch (e) {
      setError('Błąd zapisu. Spróbuj ponownie.');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl w-full max-w-md">
        <div className="p-6 border-b border-gray-100 dark:border-gray-700">
          <div className="flex items-center gap-3 mb-1">
            <div className="w-10 h-10 bg-amber-100 dark:bg-amber-900/30 rounded-lg flex items-center justify-center">
              <ShieldAlert className="w-5 h-5 text-amber-600 dark:text-amber-400" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">Zmień hasło domyślne</h2>
              <p className="text-xs text-gray-500 dark:text-gray-400">Wymagane przy pierwszym logowaniu</p>
            </div>
          </div>
        </div>

        {done ? (
          <div className="p-6 text-center text-emerald-600 dark:text-emerald-400 font-medium">
            ✓ Hasło zostało zmienione.
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="p-6 space-y-4">
            <p className="text-sm text-amber-700 dark:text-amber-400 bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg px-4 py-3">
              Korzystasz z domyślnego hasła administratora. Zmień je teraz, aby zabezpieczyć system.
            </p>

            <div>
              <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Nowe hasło</label>
              <div className="relative">
                <input
                  type={showNew ? 'text' : 'password'}
                  required
                  value={newPassword}
                  onChange={e => setNewPassword(e.target.value)}
                  className="w-full px-3 py-2 pr-10 text-sm border border-gray-200 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <button type="button" onClick={() => setShowNew(v => !v)} className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400">
                  {showNew ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                </button>
              </div>
            </div>

            <div>
              <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Potwierdź hasło</label>
              <input
                type="password"
                required
                value={confirm}
                onChange={e => setConfirm(e.target.value)}
                className="w-full px-3 py-2 text-sm border border-gray-200 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {error && <p className="text-xs text-red-600 dark:text-red-400">{error}</p>}

            <div className="flex gap-3 pt-1">
              <button type="button" onClick={onClose} className="flex-1 py-2 text-sm border border-gray-200 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700">
                Pomiń teraz
              </button>
              <button type="submit" disabled={saving} className="flex-1 py-2 text-sm bg-blue-600 hover:bg-blue-700 disabled:opacity-60 text-white rounded-lg transition-colors flex items-center justify-center gap-2">
                <Lock className="w-4 h-4" />
                {saving ? 'Zapisuję...' : 'Zmień hasło'}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}
