import { useState, type ReactElement } from 'react';
import { motion } from 'framer-motion';
import { SITE_URL, openShareIntent, type SharePlatform } from '../lib/share';

interface ShareModalProps {
  headline: string;
  subline: string;
  text: string;
  onShare: (platform: SharePlatform) => void;
  onClose: () => void;
}

const PLATFORMS: { id: SharePlatform; label: string; color: string; icon: ReactElement }[] = [
  {
    id: 'x',
    label: 'X',
    color: '#e8e6df',
    icon: (
      <path d="M18.3 2H21l-6.8 7.8L22 22h-6.4l-5-6.6L4.8 22H2l7.3-8.4L2 2h6.5l4.5 6L18.3 2Zm-1.1 18h1.8L7 4H5.1l12.1 16Z" />
    ),
  },
  {
    id: 'linkedin',
    label: 'LinkedIn',
    color: '#4a9be0',
    icon: (
      <path d="M4.98 3.5a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5ZM3 9h4v12H3V9Zm7 0h3.8v1.7h.05c.53-1 1.83-2.05 3.77-2.05C21.8 8.65 22 11.1 22 14.1V21h-4v-6.1c0-1.45-.03-3.32-2.03-3.32-2.03 0-2.34 1.58-2.34 3.22V21h-4V9Z" />
    ),
  },
  {
    id: 'whatsapp',
    label: 'WhatsApp',
    color: '#5cc264',
    icon: (
      <path d="M12.04 2C6.58 2 2.13 6.45 2.13 11.91c0 1.75.46 3.45 1.32 4.95L2 22l5.28-1.38a9.87 9.87 0 0 0 4.76 1.21h.01c5.46 0 9.9-4.45 9.9-9.91C21.96 6.45 17.5 2 12.04 2Zm4.5 13.88c-.25-.12-1.46-.72-1.68-.8-.23-.08-.39-.12-.56.12-.16.25-.64.8-.78.96-.14.17-.29.19-.53.06-.25-.12-1.05-.39-1.99-1.23-.74-.66-1.23-1.47-1.38-1.72-.14-.25-.02-.38.11-.5.11-.11.25-.29.37-.43.12-.14.16-.25.25-.41.08-.17.04-.31-.02-.43-.06-.12-.56-1.35-.77-1.85-.2-.48-.41-.42-.56-.43h-.48c-.16 0-.43.06-.66.31-.23.25-.86.84-.86 2.05 0 1.21.88 2.38 1 2.54.12.17 1.73 2.64 4.2 3.7.59.25 1.05.4 1.41.52.59.19 1.13.16 1.56.1.48-.07 1.46-.6 1.66-1.17.21-.58.21-1.08.14-1.18-.06-.1-.22-.16-.47-.28Z" />
    ),
  },
];

export function ShareModal({ headline, subline, text, onShare, onClose }: ShareModalProps) {
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(`${text} ${SITE_URL}`);
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    } catch {
      // Clipboard unavailable — the text is already visible to select by hand.
    }
  }

  function handlePlatform(platform: SharePlatform) {
    openShareIntent(platform, text);
    onShare(platform);
  }

  return (
    <motion.div
      className="fixed inset-0 z-50 flex items-center justify-center p-6"
      style={{ background: 'color-mix(in srgb, var(--bg) 82%, transparent)', backdropFilter: 'blur(6px)' }}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
    >
      <motion.div
        className="w-full max-w-sm rounded-2xl p-6 text-center"
        style={{ background: 'var(--panel)', border: '1px solid var(--line)' }}
        initial={{ opacity: 0, y: 16, scale: 0.96 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: 12, scale: 0.96 }}
        transition={{ type: 'spring', stiffness: 320, damping: 28 }}
      >
        <div className="text-xl mb-1" style={{ fontFamily: 'var(--font-display)', fontWeight: 600 }}>
          {headline}
        </div>
        <p className="mb-4" style={{ color: 'var(--ink-soft)' }}>
          {subline}
        </p>

        <div
          className="text-left text-sm rounded-xl p-3.5 mb-4"
          style={{ background: 'var(--panel-raised)', color: 'var(--ink)' }}
        >
          {text}
          <span style={{ color: 'var(--hint)' }}> {SITE_URL.replace(/^https?:\/\//, '').replace(/\/$/, '')}</span>
        </div>

        <div className="grid grid-cols-3 gap-2 mb-3">
          {PLATFORMS.map((p) => (
            <button
              key={p.id}
              onClick={() => handlePlatform(p.id)}
              className="flex flex-col items-center gap-1.5 py-3 rounded-xl"
              style={{ background: 'var(--panel-raised)' }}
            >
              <svg viewBox="0 0 24 24" width="19" height="19" fill={p.color}>
                {p.icon}
              </svg>
              <span className="text-xs font-medium" style={{ color: 'var(--ink-soft)' }}>
                {p.label}
              </span>
            </button>
          ))}
        </div>

        <button
          onClick={handleCopy}
          className="w-full py-2.5 rounded-full text-sm font-medium mb-2"
          style={{ border: '1px solid var(--line)', color: 'var(--ink-soft)' }}
        >
          {copied ? 'Copied' : 'Copy text'}
        </button>
        <button onClick={onClose} className="w-full py-2 text-sm" style={{ color: 'var(--ink-soft)' }}>
          Not now
        </button>
      </motion.div>
    </motion.div>
  );
}
