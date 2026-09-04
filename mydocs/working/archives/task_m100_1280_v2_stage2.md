# Task #1280 (v2) 2단계 완료보고서 — 프런트 ControlLayoutItem 필드 확장

## 목표

Stage 1에서 Rust 레이아웃 쿼리가 노출하기 시작한 `plane/zOrder/stableIndex`(+ shape/line/group에도
확장된 `wrap`)를 프런트 타입 `ControlLayoutItem`에 정식 선언한다. **동작 변화 없음** — 기존에
런타임 동적 접근(`ctrl.wrap` 등)하던 것을 타입으로 정합시키는 단계.

## 변경 내용

**파일**: `rhwp-studio/src/core/types.ts` `ControlLayoutItem`

선택적 필드 4개 추가:

- `plane?: number` — 렌더 정렬키. BehindText=1, 어울림/기본=2, InFrontOfText=3. 클수록 위.
- `zOrder?: number` — 개체 z-order(작을수록 아래).
- `stableIndex?: number` — 같은 plane/zOrder 내 tie-breaker.
- `wrap?: string` — 텍스트 어울림 모드(이미지뿐 아니라 shape/line/group에도 노출).

Stage 3(`findPictureAtClick` 최상단 선택)에서 이 필드를 소비한다.

## 검증

```
cd rhwp-studio && npx tsc --noEmit
```

- **신규 타입 오류 0건.** 출력되는 3건은 모두 `src/view/canvaskit-renderer.ts`의
  `canvaskit-wasm` 모듈 미설치(node_modules) 관련 **기존 베이스라인 오류**로, 본 변경과 무관하다.
  (검증: `git stash`로 본 변경 제거 후 tsc 실행 시 동일 3건 그대로 출력됨을 확인.)

```
npm test  →  node --experimental-strip-types --test tests/*.test.ts
```

- **54 passed; 0 failed.** `npm test` 스크립트(`node --test tests/*.test.ts`)는 node v22.12
  환경에서 `.ts` 확장자 로더 부재로 `ERR_UNKNOWN_FILE_EXTENSION`(7개 파일 전부 동일)으로 실패하므로,
  타입 스트리핑(`--experimental-strip-types`)을 켜서 실행했다. 본 변경은 인터페이스 필드 추가(타입
  전용)라 런타임 로직 영향이 없으며, 회귀 0.

## 다음 단계

Stage 3 — `findPictureAtClick` Pass 1을 첫-적중-반환에서 `(plane, zOrder, stableIndex)` 최상단
선택으로 교체(#1171 Pass 0 / #516 Pass 2 보존) + `topmost-hittest.test.mjs` e2e.

## 승인 대기

본 보고서와 소스 커밋 후 승인 요청. 승인 후 Stage 3 진행.
