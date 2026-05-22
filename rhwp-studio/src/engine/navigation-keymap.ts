export type PlatformKind = 'mac' | 'other';

export type NavigationAction =
  | 'wordBackward'
  | 'wordForward'
  | 'lineStart'
  | 'lineEnd'
  | 'paragraphBackward'
  | 'paragraphForward';

export interface NavigationKeyInput {
  key?: string;
  code?: string;
  shiftKey: boolean;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
}

interface PlatformInfo {
  platform?: string;
  userAgent?: string;
  maxTouchPoints?: number;
  userAgentData?: {
    platform?: string;
  };
}

type NavigationTestGlobal = typeof globalThis & {
  __rhwpTestPlatformKind?: PlatformKind;
};

function testPlatformOverride(): PlatformKind | null {
  const value = (globalThis as NavigationTestGlobal).__rhwpTestPlatformKind;
  return value === 'mac' || value === 'other' ? value : null;
}

export function detectPlatformKind(nav?: PlatformInfo): PlatformKind {
  const override = testPlatformOverride();
  if (override) return override;

  const source = (nav ?? (typeof navigator === 'undefined' ? undefined : navigator)) as PlatformInfo | undefined;
  const platform = source?.userAgentData?.platform || source?.platform || '';
  const userAgent = source?.userAgent || '';

  if (/mac|iphone|ipad|ipod/i.test(platform)) return 'mac';
  if (/mac os x|iphone|ipad|ipod/i.test(userAgent)) return 'mac';
  return 'other';
}

function navigationKey(input: NavigationKeyInput): string {
  const key = input.key || '';
  if (key === 'ArrowLeft' || key === 'ArrowRight' || key === 'ArrowUp' || key === 'ArrowDown' ||
      key === 'Home' || key === 'End') {
    return key;
  }
  return input.code || key;
}

export function getNavigationAction(
  input: NavigationKeyInput,
  platform: PlatformKind = detectPlatformKind(),
): NavigationAction | null {
  const key = navigationKey(input);
  const hasOnlyShift = !input.ctrlKey && !input.metaKey && !input.altKey;

  if (hasOnlyShift) {
    if (key === 'Home') return 'lineStart';
    if (key === 'End') return 'lineEnd';
    return null;
  }

  if (platform === 'mac') {
    if (input.altKey && !input.ctrlKey && !input.metaKey) {
      if (key === 'ArrowLeft') return 'wordBackward';
      if (key === 'ArrowRight') return 'wordForward';
    }
    if (input.metaKey && !input.ctrlKey && !input.altKey) {
      if (key === 'ArrowLeft') return 'lineStart';
      if (key === 'ArrowRight') return 'lineEnd';
    }
    return null;
  }

  if (input.ctrlKey && !input.metaKey && !input.altKey) {
    if (key === 'ArrowLeft') return 'wordBackward';
    if (key === 'ArrowRight') return 'wordForward';
    if (key === 'ArrowUp') return 'paragraphBackward';
    if (key === 'ArrowDown') return 'paragraphForward';
  }

  return null;
}

export function shouldSuppressUnmappedNavigation(
  input: NavigationKeyInput,
  platform: PlatformKind = detectPlatformKind(),
): boolean {
  const key = navigationKey(input);
  return platform === 'other' &&
    input.altKey &&
    !input.ctrlKey &&
    !input.metaKey &&
    (key === 'ArrowLeft' || key === 'ArrowRight');
}

export function formatShortcutLabel(
  label: string,
  platform: PlatformKind = detectPlatformKind(),
): string {
  if (platform !== 'mac') return label;
  return label.replace(/\bCtrl\b/g, 'Command');
}
