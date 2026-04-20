---
타스크: #196 rhwp-studio HWPX 저장 비활성화 + 사용자 안내
단계: Stage 3 — 작업지시자 수동 검증 + 부수 fix + README 갱신
브랜치: local/task196
작성일: 2026-04-19
선행: Stage 2 완료
---

# Stage 3 단계별 완료 보고서

## 1. 1차 검증 결과 (Stage 2 dist 기준)

| 케이스 | 결과 |
|---|---|
| HWPX 출처 → 저장 메뉴 비활성 | ✅ |
| 저장 메뉴 hover 툴팁 | ✅ |
| 상태 표시줄 베타 메시지 | ✅ |
| 우상단 토스트 표시 | ✅ |
| Ctrl+S 무반응 | ✅ |
| HWP 출처 정상 저장 (회귀 0) | ✅ |
| 옵션 페이지 동작 (#166 fix 회귀 검증) | ✅ |

## 2. 1차 검증 중 발견 + 즉시 처리한 부수 fix

검증 사이클에서 작업지시자 보고로 발견된 사항들:

### 2.1 토스트 표시 타이밍 — 너무 느림

**보고**: HWPX 로드 후 토스트가 너무 늦게 표시.
**원인**: `notifyHwpxBetaIfNeeded` 호출이 `initializeDocument` (폰트 로딩 + #177 validation 모달) 뒤에 있어 사용자 입력 대기까지 토스트가 안 뜸.
**수정**: 호출 위치를 `loadDocument` 직후로 이동 (1차 시도).

### 2.2 토스트와 #177 validation 모달 충돌

**보고**: 토스트가 모달과 동시에 떠서 마우스 클릭 이벤트 충돌.
**원인**: 토스트와 모달의 z-index 모두 10000. 우상단 토스트의 본체가 모달 닫기(×) 영역과 겹칠 수 있음.
**수정**:
- 토스트 호출 위치를 `initializeDocument` **후**로 다시 이동 (모달 닫힌 후 토스트)
- 토스트 z-index 를 `10000` → `21000` 으로 강화 (모달보다 명확히 위)
- 두 안내가 시간적으로 분리되어 충돌 0

### 2.3 토스트 자동 페이드 → 사용자 명시 닫기

**보고**: 자동 페이드 말고 사용자가 [확인] 누르면 닫히도록.
**원인**: 단순 안내 보다는 사용자가 인지했음을 보장하기 위해.
**수정**:
- `Toast` 컴포넌트에 `confirmLabel` 옵션 추가
- 호출부: `durationMs: 0` (자동 페이드 OFF) + `confirmLabel: '확인'`
- 확인 버튼은 강조 스타일 (파란 배경)
- [자세히] 액션은 클릭 시 새 탭 열고 토스트는 유지 (사용자가 [확인] 으로 닫음)

### 2.4 about-dialog `__APP_VERSION__` ReferenceError (#196 무관 발견)

**보고**: 메뉴 → 도움말 → 제품 정보 클릭 시 콘솔 에러.
**원인**: `rhwp-chrome/vite.config.ts` 가 `__APP_VERSION__` 의 `define` 옵션을 누락. rhwp-studio/vite.config.ts 는 정상.
**수정**: `rhwp-chrome/vite.config.ts` 에 `define: { __APP_VERSION__: JSON.stringify(studioPkg.version) }` 추가.
**부수 효과**: 작업지시자 결정으로 라이브러리/확장 버전 이원화 — rhwp-studio 0.7.2→0.7.3, rhwp-chrome/safari 확장 0.1.x→0.2.1.

### 2.5 about-dialog 한글 제품명 + 카피라이트 변경

**작업지시자 요청**:
- 한글 제품명: "한글 문서 호환 저장 도구" → **"HWP 오픈소스 편집"**
- 카피라이트: "© 2026" → **"© 2026 rhwp: Edward Kim"**

### 2.6 인쇄 미리보기 창 스타일 깨짐

**보고**: 메뉴 → 파일 → 인쇄 시 새 창의 모든 요소가 거대하게 표시. Ctrl+0 으로 정상화.
**원인**: about:blank 의 사이트별 줌 메모리. Chrome 이 이전 about:blank 줌을 기억해 새 인쇄 창에 적용.
**처리**: 본 #196 범위 외로 분리, 신규 이슈 [#199](https://github.com/edwardkim/rhwp/issues/199) 등록.

## 3. 2차 검증 결과 (부수 fix 후)

| 케이스 | 결과 |
|---|---|
| 토스트 표시 타이밍 (모달 닫힌 후) | ✅ |
| 토스트 [확인] 버튼으로만 닫힘 | ✅ |
| about-dialog 정상 동작 + 신규 한글 제품명/카피라이트 | ✅ |
| 옵션 페이지 버전 표시 (manifest 자동 반영) | ✅ |

작업지시자 보고: "rhwp 확장은 정상 동작 확인했습니다."

## 4. README 갱신

| 파일 | 변경 |
|---|---|
| [`rhwp-chrome/README.md`](rhwp-chrome/README.md) | 저장 섹션 갱신 (HWP/HWPX 분리) + 변경 이력 v0.2.1 + 향후 예정 |
| [`README.md`](README.md) | 이정표 v0.5.0 → v0.5.0~v0.7.x 확장 + 최근 변경 (v0.7.3 / 확장 v0.2.1) |
| [`README_EN.md`](README_EN.md) | Recent Changes + Coming Soon 신규 섹션 |

## 5. 산출물 변경 요약

| 영역 | 변경 |
|---|---|
| `rhwp-studio/src/command/types.ts` | `EditorContext.sourceFormat` 추가 |
| `rhwp-studio/src/main.ts` | `getContext()` 에 sourceFormat + `notifyHwpxBetaIfNeeded` |
| `rhwp-studio/src/command/commands/file.ts` | `file:save.canExecute` 갱신 |
| `rhwp-studio/src/hwpctl/index.ts` | `SaveAs` HWPX 가드 |
| `rhwp-studio/src/ui/menu-bar.ts` | 비활성 file:save 메뉴 hover 툴팁 |
| `rhwp-studio/src/ui/toast.ts` | 신규 — 우상단 토스트 + confirmLabel 옵션 |
| `rhwp-studio/src/ui/about-dialog.ts` | 한글 제품명 + 카피라이트 변경 |
| `rhwp-chrome/vite.config.ts` | `__APP_VERSION__` define 주입 |
| `rhwp-studio/package.json` | 0.7.2 → 0.7.3 |
| `rhwp-chrome/manifest.json` + `package.json` | 0.1.1 → 0.2.1 |
| `rhwp-safari/src/manifest.json` | 0.1.0 → 0.2.1 |
| README 3개 | 변경 이력 + 향후 예정 섹션 |

## 6. 검증 사이클 요약

1차 검증 → 부수 fix 6건 → 2차 검증 → README 갱신 → 마무리.

작업지시자 보고: "rhwp 확장은 정상 동작 확인했습니다."

## 7. 본 단계의 가치

원래 단순 작업지시자 검증 단계로 계획됐으나, 실제 검증 사이클에서 **6건의 사용자 인지 가치 있는 부수 fix** 가 발견·즉시 처리됨:
- UX 개선: 토스트 타이밍/모달 충돌/명시 닫기
- 기존 버그픽스: about-dialog `__APP_VERSION__`
- 브랜딩 개선: 한글 제품명/카피라이트
- 별도 이슈 분리: 인쇄 미리보기 줌 (#199)

수동 검증의 가치를 단순 회귀 확인 이상으로 활용한 사이클.

## 8. 승인 요청

본 단계 완료 보고서 승인 후 최종 결과 보고서 + 커밋 + merge + push + 이슈 close 진행.
