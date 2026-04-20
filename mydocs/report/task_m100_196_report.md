# #196 최종 보고서 — rhwp-studio HWPX 저장 비활성화 + 사용자 안내

- **타스크**: [#196](https://github.com/edwardkim/rhwp/issues/196)
- **마일스톤**: M100 (v0.2.1 — 확장) / 0.7.3 (라이브러리)
- **브랜치**: `local/task196`
- **기간**: 2026-04-19 (단일일)
- **상태**: **완료** ✅

## 1. 요약

#178 두 번째 시도가 한컴 호환 실패로 마무리되면서 후속 이슈로 분리됐고, 본 #196 은 그 동안 사용자 데이터 손상을 막기 위해 **HWPX 출처 문서의 저장을 UI 차원에서 비활성화** + **다층 안내** 제공.

추가로 검증 사이클에서 발견된 부수 fix 6건 (토스트 UX, about-dialog ReferenceError, 브랜딩, 별도 이슈 분리) 도 함께 처리. 본 v0.2.1 배포의 사용자 인지 가치 향상.

## 2. 진척 측정

| 검증 | 결과 |
|---|---|
| HWPX 출처 → 저장 메뉴/툴팁/토스트/상태바/Ctrl+S | ✅ 모두 통과 |
| HWP 출처 회귀 (정상 저장) | ✅ |
| 옵션 페이지 (#166 fix 회귀) | ✅ |
| about-dialog (`__APP_VERSION__` 주입) | ✅ |
| TypeScript 컴파일 + rhwp-chrome 빌드 | ✅ |
| 작업지시자 수동 검증 | ✅ |

## 3. 수행한 작업

### 3.1 단계별 산출물

| Stage | 산출물 | 보고서 |
|---|---|---|
| 1 | EditorContext + 가드 + 안내 UI + Toast 컴포넌트 | [stage1](mydocs/working/task_m100_196_stage1.md) |
| 2 | rhwp-studio + rhwp-chrome dist 빌드 | [stage2](mydocs/working/task_m100_196_stage2.md) |
| 3 | 작업지시자 수동 검증 + 부수 fix 6건 + README 갱신 | [stage3](mydocs/working/task_m100_196_stage3.md) |

### 3.2 #196 본 작업

| 영역 | 변경 |
|---|---|
| `EditorContext.sourceFormat` | optional 필드 추가 |
| `file:save.canExecute` | `ctx.sourceFormat !== 'hwpx'` 가드 |
| `hwpctl/SaveAs` | HWPX 가드 + 콘솔 경고 |
| `notifyHwpxBetaIfNeeded` | HWPX 로드 후 토스트 + 상태바 메시지 |
| 메뉴 hover 툴팁 | 비활성 file:save 항목 |
| `ui/toast.ts` 신규 | 우상단 슬라이드 + 확인 버튼 + 액션 옵션 |

### 3.3 부수 fix (검증 사이클 발견)

| 영역 | 변경 |
|---|---|
| 토스트 타이밍 | 모달 충돌 회피 (모달 닫힌 후 토스트) + z-index 21000 |
| 토스트 동작 | 자동 페이드 제거 + 명시 [확인] 버튼 |
| `about-dialog` ReferenceError | rhwp-chrome/vite.config.ts 에 `__APP_VERSION__` define 추가 |
| 버전 이원화 | rhwp-studio 0.7.3 / 확장 0.2.1 (동기화 정책 분리) |
| about-dialog 브랜딩 | 한글 제품명 "HWP 오픈소스 편집" + 카피라이트 "© 2026 rhwp: Edward Kim" |
| 인쇄 미리보기 줌 | 별도 이슈 [#199](https://github.com/edwardkim/rhwp/issues/199) 분리 |

### 3.4 README 갱신

3개 파일에 변경 이력 + 향후 예정 섹션 신규 추가:
- `rhwp-chrome/README.md` (Chrome Web Store 페이지)
- `README.md` (메인, 한국어)
- `README_EN.md` (영문)

## 4. 핵심 설계 결정

### 4.1 UI 차원 비활성화 (모달 차단보다 정도)

작업지시자 결정: "그냥 hwpx 의 경우 저장을 disable 시키는 건 어떤가요? 그리고 사용자에게 모던한 방법으로 알려주면 되요"

→ `canExecute` 가드로 메뉴/단축키/버튼 자동 비활성. 저장 시도 자체가 일어나지 않음. 데이터 보호 + UX 단순.

### 4.2 다층 안내

- **즉시**: 우상단 토스트 (HWPX 로드 직후, [확인] 버튼)
- **상시**: 상태 표시줄 베타 메시지
- **요청 시**: 저장 메뉴 hover 툴팁

### 4.3 버전 이원화 (작업지시자 결정)

- **rhwp-studio (라이브러리)**: 0.7.x — Rust crate 와 동기, 변화량 큼
- **rhwp-chrome / safari (확장)**: 0.x.x — Chrome Web Store 배포 주기

→ 라이브러리 변화량과 확장 배포 주기가 다를 수 있어 분리.

### 4.4 토스트 컴포넌트 — 일반 재사용 가능

`showToast({message, durationMs, action, confirmLabel})` 형태로 일반화. 향후 다른 안내에도 활용 가능.

## 5. 한계 및 후속

- **#197 완료 시 본 차단 제거** — 코드 위치 명확 (`notifyHwpxBetaIfNeeded`, `file:save.canExecute`, hwpctl 가드, menu-bar title)
- **인쇄 미리보기 줌** ([#199](https://github.com/edwardkim/rhwp/issues/199)) — 별도 이슈 후속
- **What's New 알림** — 다음 배포 또는 별도 이슈 (사용자가 업데이트 인지)
- **"다시 표시 안 함" 토스트 옵션** — 필요 시 후속

## 6. 정체성 셀프 체크

- [x] 최소 침습 — `canExecute` 1줄 + 토스트 컴포넌트 + main.ts 안내
- [x] 기존 메커니즘 재사용 — `canExecute`, dispatcher disabled, 상태바
- [x] 토스트 일반 재사용 — 다른 안내에도 활용 가능
- [x] 회귀 우선 — `sourceFormat !== 'hwpx'` 분기로 HWP 보호
- [x] PR #189 새 흐름과 충돌 0
- [x] 트러블슈팅 사전 검색 (메모리 적용)
- [x] 부수 fix 도 책임감 있게 처리 (별도 이슈 분리 또는 본 작업 포함)

## 7. 본 작업의 폭넓은 가치

원래 단순 "저장 비활성화 + 안내" 작업이었으나 검증 사이클에서 발견된 부수 fix 6건이 본 v0.2.1 배포의 사용자 인지 가치를 크게 향상:

- HWPX 데이터 손상 방지 (본 #196)
- 일반 다운로드 위치 기억 복원 ([#198](https://github.com/edwardkim/rhwp/issues/198))
- HWP Ctrl+S UX 개선 (PR [#189](https://github.com/edwardkim/rhwp/pull/189) by [@ahnbu](https://github.com/ahnbu))
- 옵션 페이지 동작 ([#166](https://github.com/edwardkim/rhwp/issues/166))
- about-dialog ReferenceError fix (본 작업 부수)
- 브랜딩 개선 (제품명/카피라이트)

## 8. 관련 자료

- 이슈 #196
- 사용자 보고: `mydocs/feedback/chrome-fd-001.md`, `mydocs/feedback/chrome-ext-update-01.md` (관련 — What's New 후속)
- 단계별 보고서: `mydocs/working/task_m100_196_stage[1..3].md`
- 수행계획서: `mydocs/plans/task_m100_196.md`
- 구현계획서: `mydocs/plans/task_m100_196_impl.md`
- 후속 이슈: [#197](https://github.com/edwardkim/rhwp/issues/197) (HWPX→HWP 완전 변환기), [#199](https://github.com/edwardkim/rhwp/issues/199) (인쇄 미리보기 줌)
- 동시 close: [#166](https://github.com/edwardkim/rhwp/issues/166) (옵션 페이지 CSP — 본 배포 포함 확인)

## 9. 다음 작업

- v0.2.1 배포 (Chrome Web Store + Microsoft Edge Add-ons)
- chrome-ext-update-01 (What's New) 별도 이슈 등록 후 진행
- #197 (HWPX→HWP 완전 변환기) — M101 또는 다음 패치
