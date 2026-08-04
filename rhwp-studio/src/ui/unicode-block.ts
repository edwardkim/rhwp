/**
 * 유니코드 블록 판정 헬퍼
 *
 * symbols-dialog.ts 에서 분리한 순수 함수 모듈. `@/engine/command` 등 런타임
 * 의존성이 없어 node 네이티브 테스트 러너로 직접 import 해도 안전하다
 * (symbols-dialog.ts 는 InsertTextCommand 를 가져오며, 해당 모듈은 node의
 * TypeScript strip-only 모드가 지원하지 않는 parameter property 문법을 쓴다).
 */

export interface UnicodeBlock {
  name: string;
  start: number;
  end: number;
}

/** 문자가 해당 유니코드 블록에 속하는지 판정(최근 사용 문자는 다른 블록일 수 있음). */
export function isCodePointInBlock(codePoint: number, block: UnicodeBlock): boolean {
  return codePoint >= block.start && codePoint <= block.end;
}
