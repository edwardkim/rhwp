import { detectPlatformKind, type PlatformKind } from '../engine/navigation-keymap.ts';

/** 키보드 단축키 정의 */
export interface ShortcutDef {
  /** 키 문자 (소문자). 예: 'z', 'b', '=', '-' */
  key: string;
  /** 물리 키 코드. IME 입력 중 key가 Process일 때 사용한다. 예: 'KeyJ' */
  code?: string;
  /** Ctrl (Windows) 또는 Meta (Mac) */
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  /** 특정 플랫폼에서만 활성화해야 하는 단축키 */
  platform?: PlatformKind;
}

/** 기본 단축키 → 커맨드 ID 매핑 */
export const defaultShortcuts: [ShortcutDef, string][] = [
  // 편집
  [{ key: 'z', ctrl: true }, 'edit:undo'],
  [{ key: 'z', ctrl: true, shift: true }, 'edit:redo'],
  [{ key: 'y', ctrl: true }, 'edit:redo'],
  [{ key: 'a', ctrl: true }, 'edit:select-all'],

  [{ key: 'e', ctrl: true }, 'edit:delete'],
  [{ key: 'ㄷ', ctrl: true }, 'edit:delete'],
  // macOS Option+C가 문자 입력으로 해석되어도 물리 C 키를 한컴 호환 모양 복사로 처리한다.
  [{ key: 'c', code: 'KeyC', alt: true }, 'edit:format-copy'],
  [{ key: 'ㅊ', alt: true }, 'edit:format-copy'],

  // 파일
  [{ key: 'n', alt: true }, 'file:new-doc'],
  [{ key: 'ㅜ', alt: true }, 'file:new-doc'],
  [{ key: 'o', ctrl: true }, 'file:open'],
  [{ key: 'ㅐ', ctrl: true }, 'file:open'],
  [{ key: 's', ctrl: true }, 'file:save'],
  // [Task #833] Ctrl+Shift+S → 다른 이름으로 저장 (한글 IME 'ㄴ' 도 함께).
  [{ key: 's', ctrl: true, shift: true }, 'file:save-as'],
  [{ key: 'ㄴ', ctrl: true, shift: true }, 'file:save-as'],
  [{ key: 'p', ctrl: true }, 'file:print'],

  // 서식
  [{ key: 'b', ctrl: true }, 'format:bold'],
  [{ key: 'i', ctrl: true }, 'format:italic'],
  [{ key: 'u', ctrl: true }, 'format:underline'],
  [{ key: 'l', alt: true }, 'format:char-shape'],
  [{ key: 'ㄹ', alt: true }, 'format:char-shape'],
  [{ key: 't', alt: true }, 'format:para-shape'],
  [{ key: 'ㅅ', alt: true }, 'format:para-shape'],

  // 서식 – 스타일
  [{ key: 'f6' }, 'format:style-dialog'],

  // 쪽
  [{ key: 'f7' }, 'file:page-setup'],

  // 줌
  [{ key: '=', ctrl: true }, 'view:zoom-in'],
  [{ key: '+', ctrl: true }, 'view:zoom-in'],
  [{ key: '-', ctrl: true }, 'view:zoom-out'],
  [{ key: '0', ctrl: true }, 'view:zoom-100'],

  // 검색
  [{ key: 'f', ctrl: true }, 'edit:find'],
  [{ key: 'f2', ctrl: true }, 'edit:find-replace'],
  [{ key: 'l', ctrl: true }, 'edit:find-again'],
  [{ key: 'v', alt: true, shift: true }, 'edit:compare-documents'],
  [{ key: 'h', ctrl: true, shift: true }, 'edit:document-history'],
  [{ key: 'g', alt: true }, 'edit:goto'],
  [{ key: 'ㅎ', alt: true }, 'edit:goto'],

  // 입력
  [{ key: 'f10', alt: true }, 'insert:symbols'],

  // 쪽
  [{ key: 'enter', ctrl: true }, 'page:break'],
  [{ key: 'enter', ctrl: true, shift: true }, 'page:column-break'],
  [{ key: 'enter', ctrl: true, alt: true }, 'page:col-settings'],

  // 줄간격
  [{ key: 'a', alt: true, shift: true }, 'format:line-spacing-decrease'],
  [{ key: 'ㅁ', alt: true, shift: true }, 'format:line-spacing-decrease'],
  [{ key: 'z', alt: true, shift: true }, 'format:line-spacing-increase'],
  [{ key: 'ㅋ', alt: true, shift: true }, 'format:line-spacing-increase'],

  // 글꼴 크기
  [{ key: 'e', alt: true, shift: true }, 'format:font-size-increase'],
  [{ key: 'ㄷ', alt: true, shift: true }, 'format:font-size-increase'],
  [{ key: 'r', alt: true, shift: true }, 'format:font-size-decrease'],
  [{ key: 'ㄱ', alt: true, shift: true }, 'format:font-size-decrease'],
  // 글꼴 크기 — Ctrl+]/[ (한컴 호환, 브라우저 충돌 없음)
  [{ key: ']', ctrl: true }, 'format:font-size-increase'],
  [{ key: '[', ctrl: true }, 'format:font-size-decrease'],

  // 장평/자간 (한컴 호환)
  [{ key: 'j', code: 'KeyJ', alt: true, shift: true }, 'format:char-ratio-decrease'],
  [{ key: 'ㅓ', alt: true, shift: true }, 'format:char-ratio-decrease'],
  [{ key: 'k', code: 'KeyK', alt: true, shift: true }, 'format:char-ratio-increase'],
  [{ key: 'ㅏ', alt: true, shift: true }, 'format:char-ratio-increase'],
  [{ key: 'n', code: 'KeyN', alt: true, shift: true }, 'format:char-spacing-decrease'],
  [{ key: 'ㅜ', alt: true, shift: true }, 'format:char-spacing-decrease'],
  [{ key: 'w', code: 'KeyW', alt: true, shift: true }, 'format:char-spacing-increase'],
  [{ key: 'ㅈ', alt: true, shift: true }, 'format:char-spacing-increase'],

  // 문단 정렬
  // Ctrl+Shift+L: 왼쪽 정렬 (브라우저 주소창 포커스이나 편집 영역에서 양보)
  [{ key: 'l', ctrl: true, shift: true }, 'format:align-left'],
  // Ctrl+Shift+M: 양쪽 정렬 (브라우저 충돌 없음)
  [{ key: 'm', ctrl: true, shift: true }, 'format:align-justify'],
  // Ctrl+Shift+R: 브라우저 강제새로고침 충돌 → Alt+Shift+H로 재매핑 (Alt+Shift+R은 글꼴크기축소)
  // Ctrl+Shift+C: 브라우저 요소검사 충돌 → Alt+Shift+C로 재매핑
  // Ctrl+Shift+T: 브라우저 탭복원 충돌 → Alt+Shift+T로 재매핑
  [{ key: 'h', alt: true, shift: true }, 'format:align-right'],   // 오른쪽 정렬 (재매핑, H=rigHt)
  [{ key: 'ㅗ', alt: true, shift: true }, 'format:align-right'],
  [{ key: 'c', alt: true, shift: true }, 'format:align-center'],  // 가운데 정렬 (재매핑)
  [{ key: 'ㅊ', alt: true, shift: true }, 'format:align-center'],
  [{ key: 'd', alt: true, shift: true }, 'format:align-distribute'], // 배분 정렬 (재매핑)
  [{ key: 'ㅇ', alt: true, shift: true }, 'format:align-distribute'],

  // 표
  [{ key: 'enter', alt: true }, 'table:insert-row-col'],
  [{ key: 'delete', alt: true }, 'table:delete-row-col'],
  [{ key: 's', ctrl: true, shift: true }, 'table:block-sum'],
  [{ key: 'a', ctrl: true, shift: true }, 'table:block-avg'],
  [{ key: 'p', ctrl: true, shift: true }, 'table:block-product'],
];

/**
 * KeyboardEvent에 매칭되는 단축키가 있으면 커맨드 ID를 반환한다.
 * 없으면 null.
 */
export function matchShortcut(
  e: KeyboardEvent,
  shortcuts: [ShortcutDef, string][],
  platform: PlatformKind = detectPlatformKind(),
): string | null {
  const ctrlOrMeta = e.ctrlKey || e.metaKey;
  const eventKey = e.key.toLowerCase();
  const eventCode = (e.code ?? '').toLowerCase();

  for (const [def, cmdId] of shortcuts) {
    if (def.platform && def.platform !== platform) continue;
    if (def.ctrl && !ctrlOrMeta) continue;
    if (!def.ctrl && ctrlOrMeta) continue;
    if ((def.shift ?? false) !== e.shiftKey) continue;
    if ((def.alt ?? false) !== e.altKey) continue;
    if (eventKey === def.key) return cmdId;
    if (def.code && eventCode === def.code.toLowerCase()) return cmdId;
  }
  return null;
}
