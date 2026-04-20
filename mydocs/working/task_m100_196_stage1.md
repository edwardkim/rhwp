---
타스크: #196 rhwp-studio HWPX 저장 비활성화 + 사용자 안내
단계: Stage 1 — EditorContext 확장 + 가드 + 안내 UI + 토스트 컴포넌트
브랜치: local/task196
작성일: 2026-04-19
선행: 구현계획서 task_m100_196_impl.md (승인됨)
---

# Stage 1 단계별 완료 보고서

## 1. 목표

EditorContext 확장 + 진입점 가드 + 다층 안내 UI + 토스트 컴포넌트.

## 2. 산출물

### 2.1 신규 파일

[`rhwp-studio/src/ui/toast.ts`](rhwp-studio/src/ui/toast.ts) — 우상단 슬라이드 토스트 컴포넌트.

API:
```typescript
export function showToast(options: {
  message: string;
  durationMs?: number;  // 기본 8000ms, 0 = 자동 페이드 없음
  action?: { label: string; onClick: () => void };
}): void;
```

특징:
- 우상단 고정 슬라이드 인 (200ms ease-out)
- 자동 페이드 (350ms) — duration 후 자동 제거
- 사용자 닫기 버튼 (×) — 즉시 제거
- 선택적 액션 버튼 (텍스트 링크 스타일)
- 다중 토스트 지원 (container 자동 생성)
- ARIA `role="status"` + `aria-live="polite"`

### 2.2 수정 파일

| 파일 | 변경 |
|---|---|
| [`command/types.ts`](rhwp-studio/src/command/types.ts) | `EditorContext.sourceFormat?: 'hwp' \| 'hwpx'` 추가 |
| [`main.ts`](rhwp-studio/src/main.ts) | `getContext()` 에 `sourceFormat` 채움 + `showToast` import + `notifyHwpxBetaIfNeeded()` 함수 + `loadBytes` 에서 호출 |
| [`commands/file.ts`](rhwp-studio/src/command/commands/file.ts) | `file:save.canExecute` 갱신: `ctx.hasDocument && ctx.sourceFormat !== 'hwpx'` |
| [`hwpctl/index.ts`](rhwp-studio/src/hwpctl/index.ts) | `SaveAs` 진입 직후 HWPX 가드 + 콘솔 경고 + return false |
| [`ui/menu-bar.ts`](rhwp-studio/src/ui/menu-bar.ts) | `updateMenuStates`: `file:save` 비활성 시 `title` 속성으로 툴팁 표시 |

## 3. 핵심 동작

### 3.1 file:save 진입점 차단

```typescript
canExecute: (ctx) => ctx.hasDocument && ctx.sourceFormat !== 'hwpx',
```

→ HWPX 출처 시:
- 메뉴 항목 disabled (회색)
- 단축키 Ctrl+S 무반응
- dispatcher 가 자동 처리

### 3.2 hwpctl/SaveAs 진입점 차단

```typescript
if (sourceFormat === 'hwpx' && format !== 'hwp') {
  console.warn('[hwpctl] SaveAs: HWPX 출처 저장은 현재 베타 단계로 비활성화되어 있습니다 (#196)');
  return false;
}
```

(향후 #197 완성 시 `format === 'hwp'` 옵션은 어댑터 호출 분기로 활용 가능 — 현재는 사용 안 함)

### 3.3 다층 안내

| 위치 | 내용 |
|---|---|
| 우상단 토스트 (HWPX 로드 직후 1회, 8초) | "HWPX 형식은 현재 베타 단계라 직접 저장이 비활성화되어 있습니다.\n다음 업데이트에서 지원 예정입니다." + [자세히] 액션 (#197 링크) |
| 상태 표시줄 | "HWPX 베타 모드 — 저장은 다음 업데이트에서 지원됩니다" |
| 저장 메뉴 hover 툴팁 | "HWPX 직접 저장은 현재 베타 단계로 비활성화되어 있습니다. 다음 업데이트에서 지원 예정입니다." |

## 4. 검증 결과

### 4.1 TypeScript 컴파일

```
$ cd rhwp-studio && npx tsc --noEmit
(에러 0)
```

### 4.2 단위 테스트

토스트 컴포넌트는 DOM 의존이 강해 별도 단위 테스트 미작성. 작업지시자 시각 검증 (Stage 3) 으로 대체.

## 5. 정체성 셀프 체크

- [x] 최소 침습 — 5개 파일 수정 + 1개 파일 신규
- [x] 기존 메커니즘 재사용 — `canExecute`, dispatcher disabled, 상태바 sb-message
- [x] 토스트는 일반 재사용 가능한 형태 — 다른 안내에도 활용 가능
- [x] 회귀 우선 — `sourceFormat !== 'hwpx'` 분기로 HWP 출처 보호
- [x] PR #189 새 흐름과 충돌 없음 — file:save 진입 자체를 막아 새 흐름 미진입
- [x] #197 완료 시 제거 용이 — `notifyHwpxBetaIfNeeded`, `canExecute`, hwpctl 가드, menu-bar title 처리 모두 명확한 위치

## 6. 다음 단계

Stage 2: 빌드 검증.

- rhwp-studio 빌드 (`npm run build`)
- rhwp-chrome dist 빌드 (`npm run build`)

## 7. 승인 요청

본 단계 완료 보고서 승인 후 Stage 2 착수.
