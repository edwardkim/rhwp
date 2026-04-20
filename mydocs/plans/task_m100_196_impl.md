---
타스크: #196 rhwp-studio HWPX 저장 비활성화 + 사용자 안내
브랜치: local/task196
작성일: 2026-04-19
선행: mydocs/plans/task_m100_196.md (수행계획서, 승인됨)
---

# 구현계획서: rhwp-studio HWPX 저장 비활성화 + 사용자 안내

## 0. 작업지시자 결정 사항 (수행계획서 §11 + 후속)

| 질문 | 결정 |
|---|---|
| Q1. 단계 분할 | **3단계** (가드+안내UI 통합 / 빌드 / 검증) |
| Q2. 토스트 컴포넌트 | 우상단 슬라이드 + 자동 페이드 + 사용자 닫기 (× 버튼) |
| Q3. sourceFormat 접근 | **(A) `EditorContext.sourceFormat` 추가** — 정도. `canExecute(ctx)` 가 services 접근 불가하므로 ctx 확장이 정도 |
| Q4. What's New 신규 이슈 | 본 #196 완료 후 등록 |

## 1. 핵심 설계

### 1.1 EditorContext 확장 (Q3 결정)

[`rhwp-studio/src/command/types.ts:7-32`](rhwp-studio/src/command/types.ts#L7-L32) 의 `EditorContext` 에 1줄 추가:

```typescript
export interface EditorContext {
  // ... 기존 필드들 ...
  /** 원본 파일 형식 ('hwp' | 'hwpx' | 미로드 시 undefined) */
  sourceFormat?: 'hwp' | 'hwpx';
}
```

[`rhwp-studio/src/main.ts:45-60`](rhwp-studio/src/main.ts#L45-L60) `getContext()` 갱신:

```typescript
function getContext(): EditorContext {
  return {
    hasDocument: wasm.pageCount > 0,
    // ... 기존 필드들 ...
    sourceFormat: wasm.pageCount > 0 ? wasm.getSourceFormat() as 'hwp' | 'hwpx' : undefined,
  };
}
```

### 1.2 file:save 진입점 가드

[`commands/file.ts:44-48`](rhwp-studio/src/command/commands/file.ts#L44-L48):

```typescript
{
  id: 'file:save',
  label: '저장',
  icon: 'icon-save',
  shortcutLabel: 'Ctrl+S',
  canExecute: (ctx) => ctx.hasDocument && ctx.sourceFormat !== 'hwpx',
  // ...
}
```

→ HWPX 출처는 `canExecute === false` → 메뉴/버튼/단축키 자동 비활성.

### 1.3 hwpctl/SaveAs 진입점 가드

[`hwpctl/index.ts:72`](rhwp-studio/src/hwpctl/index.ts#L72) `SaveAs(filename, format?, arg?)`:

```typescript
SaveAs(filename: string, _format?: string, _arg?: string): boolean {
  try {
    const sourceFormat = this.wasmDoc.getSourceFormat();
    if (sourceFormat === 'hwpx') {
      console.warn('[hwpctl] SaveAs: HWPX 출처 저장 비활성화됨 (#196)');
      return false;
    }
    // ... 기존 HWP 저장 로직 (PR #189 이전 형태 유지)
  }
}
```

### 1.4 신규 토스트 컴포넌트

신규 파일: [`rhwp-studio/src/ui/toast.ts`](rhwp-studio/src/ui/toast.ts)

API:
```typescript
export interface ToastOptions {
  /** 메시지 본문 (필수) */
  message: string;
  /** 자동 페이드 시간 (ms). 기본 8000ms. */
  durationMs?: number;
  /** 액션 버튼 (선택) */
  action?: {
    label: string;
    onClick: () => void;
  };
}

export function showToast(options: ToastOptions): void;
```

레이아웃:
- 우상단 고정 (top: 16px, right: 16px)
- 슬라이드 인 (transform: translateX(100%) → 0, transition 200ms)
- 자동 페이드 (opacity 1 → 0, duration 후 350ms 이내)
- 닫기 버튼 (×) — 클릭 시 즉시 페이드아웃
- 액션 버튼 (선택) — 메시지 옆에 텍스트 링크 스타일
- z-index: 1000+ (모달보다 위)
- max-width: 400px

스타일은 기존 confirm-dialog 의 디자인 토큰 (테두리, 색상) 따름.

### 1.5 안내 표시 진입점

main.ts 의 `loadFile` / `loadBytes` 직후 sourceFormat 검사:

```typescript
async function loadBytes(...): Promise<void> {
  // ... 기존 로직 ...
  await initializeDocument(docInfo, ...);

  // HWPX 출처 안내 (#196)
  if (wasm.getSourceFormat() === 'hwpx') {
    showToast({
      message: 'HWPX 형식은 현재 베타 단계라 직접 저장이 비활성화되어 있습니다.\n다음 업데이트에서 지원 예정입니다.',
      durationMs: 8000,
    });
    // 상태 표시줄 메시지
    const sb = document.getElementById('sb-message');
    if (sb) sb.textContent = 'HWPX 베타 모드 — 저장은 다음 업데이트에서 지원됩니다';
  }
}
```

### 1.6 저장 버튼 disabled 시 툴팁

`canExecute === false` 시 dispatcher 가 메뉴/버튼에 자동 적용하는 disabled 상태에 더해, **툴팁 (`title` 속성)** 도 추가 필요. dispatcher 의 메뉴/버튼 렌더링 위치 확인 후 disabled 상태에서 title 속성 부여:

> "HWPX 직접 저장은 현재 베타 단계로 비활성화되어 있습니다. 다음 업데이트에서 지원 예정입니다."

위치: `command/dispatcher.ts` (또는 메뉴/도구상자 렌더 함수).

## 2. 단계 분할 (3 Stage)

### Stage 1 — EditorContext 확장 + 가드 + 안내 UI

**변경**:
- `command/types.ts`: `EditorContext.sourceFormat` 필드 추가
- `main.ts`: `getContext()` 에 sourceFormat 채움
- `commands/file.ts`: `file:save.canExecute` 갱신
- `hwpctl/index.ts`: `SaveAs` HWPX 가드 + 콘솔 경고
- `ui/toast.ts` 신규: 우상단 슬라이드 토스트 컴포넌트
- `main.ts`: `loadBytes` 직후 HWPX 출처 토스트 + 상태바 메시지
- `command/dispatcher.ts` (또는 메뉴 렌더): disabled 버튼에 툴팁 (`title` 속성)

**신규/수정 파일**:
- 신규: `ui/toast.ts`
- 신규 (선택): `ui/toast.test.ts`
- 수정: `command/types.ts`, `main.ts`, `commands/file.ts`, `hwpctl/index.ts`, `command/dispatcher.ts` (또는 메뉴 렌더 위치)

**단위 테스트**:
- `toast.test.ts`: 표시/자동 페이드/닫기 버튼/액션 클릭 (가능 범위)
- `canExecute` 분기는 단순한 식이라 별도 테스트 불요

**완료 기준**:
- TypeScript 컴파일 에러 0
- 신규 토스트 단위 테스트 그린 (있다면)
- vite dev 시각 동작 (HWPX 로드 → 비활성 + 토스트)

### Stage 2 — 빌드 검증

- rhwp-studio 빌드 (`npm run build`)
- rhwp-chrome dist 빌드 (`npm run build`)
- TypeScript 컴파일 0 에러

**완료 기준**: 양쪽 빌드 산출물 정상 생성.

### Stage 3 — 작업지시자 수동 검증 + 보고서

검증 케이스:
- HWPX 파일 열기 → 저장 메뉴/버튼/Ctrl+S 비활성 (회색)
- HWPX 파일 열기 → 저장 버튼 hover 툴팁 표시
- HWPX 파일 열기 → 상태바 베타 메시지 표시
- HWPX 파일 열기 → 우상단 토스트 1회 (자동 페이드)
- 토스트 × 버튼 클릭 → 즉시 닫힘
- HWP 파일 열기 → 위 모든 안내 없음, 정상 저장 가능

보고서:
- `mydocs/working/task_m100_196_stage[1..3].md`
- `mydocs/report/task_m100_196_report.md`
- 오늘할일 갱신
- merge → push → 이슈 close

## 3. 파일 변경 요약

| Stage | 신규 파일 | 수정 파일 |
|---|---|---|
| 1 | `ui/toast.ts`, (선택) `ui/toast.test.ts` | `command/types.ts`, `main.ts`, `commands/file.ts`, `hwpctl/index.ts`, `command/dispatcher.ts` (또는 메뉴 렌더) |
| 2 | — | `rhwp-chrome/dist/*` (빌드 산출물) |
| 3 | `mydocs/working/task_m100_196_stage[1..3].md`, `mydocs/report/task_m100_196_report.md` | `mydocs/orders/20260419.md` |

## 4. 위험 (수행계획서 §6 보강)

| 위험 | 단계 | 완화 |
|---|---|---|
| `EditorContext.sourceFormat` 추가가 다른 호출자 영향 | Stage 1 | optional 필드 (`?:`) 라 기존 코드 무영향 |
| dispatcher 의 메뉴 렌더가 `title` 속성 미설정 | Stage 1 | dispatcher 코드 확인 + `canExecute === false` 분기에 title 추가 |
| 토스트가 모달과 z-index 충돌 | Stage 1 | z-index 1000+ 로 명시적 우선순위 |
| 도구 상자 disabled 버튼 시각 표현이 OS 별 차이 | Stage 1 | 기존 disabled 패턴 (다른 버튼) 참고하여 일관성 유지 |
| HWPX 로드 후 다른 파일 (HWP) 열기 시 상태바 메시지 잔존 | Stage 1 | loadBytes 진입 시 HWP 출처면 상태바 메시지 초기화 |

## 5. 검증

- 단위 테스트: 토스트 컴포넌트 (가능 범위)
- 작업지시자 수동 (Stage 3): vite dev server 또는 chrome 확장
- 회귀: HWP 출처 무영향

## 6. 일정

- Stage 1: 0.8일
- Stage 2: 0.1일
- Stage 3: 0.5일
- **총: 1.5일**

## 7. 정체성 셀프 체크

- [x] 최소 침습 — `canExecute` 1줄 + 토스트 컴포넌트 + main.ts 안내 호출
- [x] 기존 메커니즘 재사용 — `canExecute`, dispatcher 의 disabled 처리, 상태바 메시지
- [x] 신규 토스트 컴포넌트는 일반 재사용 가능한 형태 (다른 안내에도 활용 가능)
- [x] 회귀 우선 — HWP 출처는 sourceFormat !== 'hwpx' 분기로 보호
- [x] 트러블슈팅 사전 검색 (메모리 적용) — 0건 확인
- [x] PR #189 새 흐름 (current-handle) 과 충돌 없음 — `file:save` 진입 자체를 막으므로 새 흐름 영역 미진입

## 8. 승인 요청

본 구현계획서 승인 후 Stage 1 착수.
