export const SITE_URL = 'https://playmorris.vercel.app/';

const HANDLE = '@shramanb113';

const HINT_CREDITS_KEY = 'morris-bench:hint-credits:v1';
const STARTING_HINT_CREDITS = 3;
export const HINT_CREDITS_PER_SHARE = 2;

export function hintCredits(): number {
  try {
    const raw = localStorage.getItem(HINT_CREDITS_KEY);
    if (raw === null) return STARTING_HINT_CREDITS;
    const n = Number.parseInt(raw, 10);
    return Number.isFinite(n) ? n : STARTING_HINT_CREDITS;
  } catch {
    return STARTING_HINT_CREDITS;
  }
}

function setHintCredits(n: number): void {
  try {
    localStorage.setItem(HINT_CREDITS_KEY, String(n));
  } catch {
    // Storage unavailable — credits just won't persist across visits.
  }
}

/** Spends one credit if available. Returns whether it succeeded. */
export function consumeHintCredit(): boolean {
  const n = hintCredits();
  if (n <= 0) return false;
  setHintCredits(n - 1);
  return true;
}

export function grantHintCredits(n: number): void {
  setHintCredits(hintCredits() + n);
}

// Written to invite a reply, not just a glance — per X's own open-sourced
// ranking weights (xai-org/x-algorithm, home-mixer/params/param.rs),
// ReplyWeight (5.0) and ShareViaCopyLinkWeight (20.0) outweigh
// FavoriteWeight (0.5) by an order of magnitude, and hashtags carry no
// ranking weight at all in that code — so this leans on a genuine
// challenge/question over hashtag-stuffing, with exactly one topical tag
// for search discoverability, not for reach.
export const SHARE_TEXT =
  `Nine Men's Morris, but the bot doesn't take it easy on you. Betting you can't beat it on Hard — reply if you do. Made by ${HANDLE}. #BoardGames`;

const DIFFICULTY_LABEL: Record<string, string> = { easy: 'Easy', medium: 'Medium', hard: 'Hard' };

/** Win-specific post text — tags the maker and invites a reply/challenge. */
export function winShareText(difficulty: string): string {
  const level = DIFFICULTY_LABEL[difficulty] ?? difficulty;
  return `Just beat the bot at Nine Men's Morris on ${level} difficulty — reply if you think you can too. Made by ${HANDLE}. #BoardGames`;
}

export type SharePlatform = 'x' | 'linkedin' | 'whatsapp';

/**
 * Each platform's own compose/share intent — a plain URL opened in a new
 * tab, so the visitor lands on the platform's own post screen with the
 * text already filled in (where the platform's intent supports that) and
 * just hits post themselves. No native OS share sheet involved.
 *
 * LinkedIn and Facebook's share intents don't accept custom post text
 * (they only take a URL and build their own preview from the page's Open
 * Graph tags), so `text` is unused for those — the link is what carries
 * the message there.
 */
export function shareIntentUrl(platform: SharePlatform, text: string): string {
  switch (platform) {
    case 'x':
      return `https://x.com/intent/tweet?text=${encodeURIComponent(text)}&url=${encodeURIComponent(SITE_URL)}`;
    case 'linkedin':
      return `https://www.linkedin.com/sharing/share-offsite/?url=${encodeURIComponent(SITE_URL)}`;
    case 'whatsapp':
      return `https://wa.me/?text=${encodeURIComponent(`${text} ${SITE_URL}`)}`;
  }
}

export function openShareIntent(platform: SharePlatform, text: string): void {
  window.open(shareIntentUrl(platform, text), '_blank', 'noopener,noreferrer');
}
