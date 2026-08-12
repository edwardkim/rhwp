/**
 * UI chrome 프로파일 리졸버 (#4564).
 *
 * `?chrome=embed` — iframe 임베드처럼 문서 수명주기(열기/저장)를 호스트가 소유하는
 * 구성용 opt-in 프로파일. `?renderer=`와 같은 패턴을 따른다: 순수 resolve 함수,
 * URL 파라미터만 읽고(저장소 지속 없음), 미지원 값은 기본(full)으로 폴백하며
 * unsupportedReason으로 보고한다.
 */

export type ChromeMode = 'full' | 'embed';
export type ChromeModeRequestSource = 'default' | 'url';
export type ChromeModeUnsupportedReason = 'unsupportedChromeMode';

export interface ChromeModeRequest {
  mode: ChromeMode;
  source: ChromeModeRequestSource;
  requested?: string;
  unsupportedReason?: ChromeModeUnsupportedReason;
}

/**
 * embed 프로파일에서 등록하지 않는 파일 수명주기 커맨드.
 *
 * 문서 수명주기를 호스트가 소유하는 구성에서 로컬 저장류는 "저장됐다"는 오인을
 * 만들고(다운로드 폴더로 떨어질 뿐 호스트 저장소에는 반영되지 않는다), 열기/새
 * 문서는 호스트가 감지할 수 없는 문서 교체 경로를 연다. `file:page-setup`과
 * `file:about`은 수명주기가 아니라 편집·정보 표면이므로 유지한다.
 */
export const EMBED_HIDDEN_FILE_COMMAND_IDS: readonly string[] = [
  'file:new-doc',
  'file:open',
  'file:open-recent',
  'file:clear-recent',
  'file:save',
  'file:save-as',
  'file:save-as-hwp',
  'file:save-as-hwpx',
  'file:print-to-pdf',
  'file:print',
];

/**
 * embed 프로파일에서 등록하지 않는 편집 커맨드.
 *
 * 문서 비교는 비교 실행 시 오른쪽 문서를 현재 에디터에 로드하므로, 파일 열기와
 * 같은 급의 문서 교체 진입점이다 — 호스트는 문서 A를 열었다고 알고 있는데 Studio
 * 내부 문서가 B로 바뀔 수 있다.
 */
export const EMBED_HIDDEN_EDIT_COMMAND_IDS: readonly string[] = [
  'edit:compare-documents',
];

/** KeyboardEvent에서 단축키 판정에 쓰는 부분 — 순수 함수 테스트용 구조적 타입. */
export interface EmbedShortcutKeyEventLike {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

/**
 * embed에서 브라우저 기본 동작으로 새면 안 되는 파일 수명주기 단축키 판정.
 *
 * InputHandler가 활성이면 shortcut-map 매칭이 Ctrl+S/Ctrl+Shift+S/Ctrl+P를
 * preventDefault로 삼키지만, 문서 로드 전에는 그 경로 자체가 없어 브라우저
 * 저장/인쇄 대화상자로 빠진다. Alt+N/Ctrl+O는 전역 단축키 핸들러가 문서 유무와
 * 무관하게 이미 삼키므로 제외한다. 한글 IME 키(ㄴ/ㅔ)는 전역 핸들러의 ㅜ/ㅐ
 * 처리와 같은 이유로 함께 받는다.
 */
export function isEmbedSwallowedFileShortcut(e: EmbedShortcutKeyEventLike): boolean {
  if (!(e.ctrlKey || e.metaKey) || e.altKey) return false;
  const key = e.key.toLowerCase();
  // Ctrl+S 저장, Ctrl+Shift+S 다른 이름으로 저장
  if (key === 's' || key === 'ㄴ') return true;
  // Ctrl+P 인쇄, Ctrl+Shift+P 크롬 시스템 인쇄 대화상자 — 후자의 문서 로드 후
  // 매핑(table:block-product)은 InputHandler가 어차피 preventDefault하므로
  // 전역 흡수가 그 경로를 해치지 않는다.
  return key === 'p' || key === 'ㅔ';
}

export function resolveChromeMode(search = ''): ChromeMode {
  return resolveChromeModeRequest(search).mode;
}

export function resolveChromeModeRequest(search = ''): ChromeModeRequest {
  const explicit = new URLSearchParams(search).get('chrome');
  const normalized = explicit?.trim().toLowerCase();
  if (!normalized) return { mode: 'full', source: 'default' };
  if (normalized === 'embed') {
    return { mode: 'embed', source: 'url', requested: normalized };
  }
  if (normalized === 'full') {
    return { mode: 'full', source: 'url', requested: normalized };
  }
  return {
    mode: 'full',
    source: 'url',
    requested: explicit ?? normalized,
    unsupportedReason: 'unsupportedChromeMode',
  };
}
