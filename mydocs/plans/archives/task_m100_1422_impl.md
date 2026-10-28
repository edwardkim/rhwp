# 구현 계획서 — Task M100-1422: rhwp-studio 다크모드 잔여 UI 대비 정리

- 이슈: https://github.com/edwardkim/rhwp/issues/1422
- 수행 계획서: `mydocs/plans/task_m100_1422.md`
- 작성일: 2026-06-17
- 브랜치: `local/task1422`
- 기준 커밋: `ab1879c94328cf49b569e2d687ae723b75f3acaa`

## 1. 설계 요약

#1422는 PR #1420의 테마 기반을 재설계하지 않고, 남은 개별 UI의 색상 처리를 정리하는 후속 작업이다.
구현의 핵심은 다음 세 가지다.

1. UI chrome과 문서 색상 preview를 분리한다.
2. inline light 색상을 semantic token 또는 명시적 문서 preview 색상으로 분류한다.
3. Chrome Auto Dark Mode는 마지막 단계에서 앱의 명시적 테마 선택을 보존하는 방향으로 점검한다.

문서 preview와 색상 견본은 실제 문서 의미를 유지해야 하므로 `--doc-paper` 또는 실제 색상값을 사용한다.
반대로 버튼, fieldset/legend, input/select/textarea, popup surface, 보조 테두리는 UI token을 사용한다.

## 2. 예상 수정 파일

주요 수정 후보:

- `rhwp-studio/src/styles/base.css`
- `rhwp-studio/src/styles/dialogs.css`
- `rhwp-studio/src/core/theme.ts`
- `rhwp-studio/index.html`
- `rhwp-studio/src/ui/table-cell-props-dialog.ts`
- `rhwp-studio/src/ui/cell-border-bg-dialog.ts`
- `rhwp-studio/src/ui/equation-editor-dialog.ts`
- `rhwp-studio/src/ui/page-border-dialog.ts`
- `rhwp-studio/src/ui/table-create-dialog.ts`
- `rhwp-studio/src/ui/endnote-shape-dialog.ts`
- `rhwp-studio/src/ui/para-shape-dialog.ts`
- `rhwp-studio/src/ui/toolbar.ts`
- `rhwp-studio/src/ui/validation-modal.ts`
- `rhwp-studio/src/ui/grid-settings-dialog.ts`
- `rhwp-studio/e2e/theme-mode.test.mjs`
- 필요 시 `rhwp-studio/e2e/dialog-theme.test.mjs` 신규

문서 산출물:

- `mydocs/working/task_m100_1422_stage1.md`
- `mydocs/working/task_m100_1422_stage2.md`
- `mydocs/working/task_m100_1422_stage3.md`
- `mydocs/working/task_m100_1422_stage4.md`
- `mydocs/working/task_m100_1422_stage5.md`
- `mydocs/working/task_m100_1422_stage6.md`
- `mydocs/report/task_m100_1422_report.md`

## 3. Stage 1 — 공통 dialog/control 토큰 정리

목표:

- 공통 폼 컨트롤이 다크모드에서 토큰 기반 배경/글자/테두리를 갖도록 한다.
- 이후 개별 다이얼로그 수정이 공통 스타일을 재사용할 수 있게 한다.

작업:

1. `dialogs.css`의 `.dialog-input`, `.dialog-select`, textarea 계열에 background/color/border 토큰을 명시한다.
2. `disabled`, `readOnly`, `:focus`, placeholder 상태를 다크모드에서 충분한 대비로 정리한다.
3. fieldset/legend, popup surface, preview 주변 버튼에 재사용 가능한 class 또는 token 사용 패턴을 정리한다.
4. 문서 preview용 색상은 UI surface와 분리해 `--doc-paper`를 우선 사용한다.

검증:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

완료 기준:

- 공통 `.dialog-input`이 다크모드에서 흰 배경으로 남지 않는다.
- light 테마에서 기존 입력 필드 시각이 크게 변하지 않는다.
- Stage 1 완료보고서를 작성하고 승인 요청한다.

## 4. Stage 2 — 표/셀 속성 및 셀 테두리/배경 정리

목표:

- 첨부 화면에서 확인된 표/셀 속성의 읽기 전용 필드와 테두리 preview 주변 UI를 우선 수정한다.
- 표/셀 속성과 셀 테두리/배경의 중복된 preview 처리 방식을 맞춘다.

작업:

1. `table-cell-props-dialog.ts`의 읽기 전용 width/height inline `#f5f5f5` 배경을 제거하거나 read-only token으로 대체한다.
2. `table-cell-props-dialog.ts`와 `cell-border-bg-dialog.ts`의 선 샘플/보조선/preview 주변 버튼 색상을 분류한다.
3. 실제 셀 배경 없음 또는 종이 배경을 뜻하는 흰색 preview는 `--doc-paper` 또는 명시적 문서 preview 값으로 유지한다.
4. 방향 버튼과 라벨, 보조 테두리는 UI token을 사용한다.

검증:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

시각 확인:

- 다크모드 표/셀 속성의 기본 탭 크기 필드
- 표/셀 속성 테두리/배경 미리보기
- 셀 테두리/배경 다이얼로그 미리보기

완료 기준:

- 읽기 전용 크기 입력 필드가 다크 UI와 일관된다.
- 중앙 셀/문서 preview는 문서 색상 의미를 유지한다.
- Stage 2 완료보고서를 작성하고 승인 요청한다.

## 5. Stage 3 — 수식 편집 및 쪽 테두리/배경 정책 반영

목표:

- 수식 편집 preview 가독성을 저장 색상 변경 없이 해결한다.
- 쪽 테두리/배경은 중앙 문서 preview를 흰 종이로 유지하고 주변 UI만 다크모드 처리한다.

작업:

1. `.eq-preview` 배경을 UI surface가 아니라 문서 종이 preview 성격으로 조정한다.
2. `equation-editor-dialog.ts`에서 preview-only 색상 반전 없이 기본 검은 수식이 읽히는지 확인한다.
3. `page-border-dialog.ts`의 중앙 SVG preview 배경과 내부 종이 fill은 흰 종이 표현으로 유지한다.
4. `page-border-dialog.ts`의 fieldset/legend, 그룹 제목, 미리보기 사방 버튼, 비활성 컨트롤은 UI token으로 전환한다.

검증:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

시각 확인:

- 다크모드 수식 편집 preview의 기본 검은 수식 가독성
- 쪽 테두리/배경 중앙 문서 preview 흰색 유지
- 쪽 테두리/배경 주변 fieldset/버튼 대비 개선

완료 기준:

- 수식 저장 색상 의미를 바꾸지 않는다.
- 쪽 테두리/배경 중앙 문서 preview를 다크 UI surface로 바꾸지 않는다.
- Stage 3 완료보고서를 작성하고 승인 요청한다.

## 6. Stage 4 — 추가 라이트 하드코딩 sweep

목표:

- 이슈 본문에 추가 후보로 등록된 popup/dialog의 동일 패턴을 최소 수정한다.
- 문서 preview와 UI chrome을 다시 분류해 불필요한 전면 리팩터링을 피한다.

작업:

1. `table-create-dialog.ts` quick grid popup의 surface, border, hover, label 색상을 token화한다.
2. `endnote-shape-dialog.ts`의 preview button/menu surface와 option hover 색상을 token화한다.
3. `para-shape-dialog.ts`의 preview 텍스트/표면을 문서 preview 정책에 맞게 분리한다.
4. `toolbar.ts`의 글머리표 popup surface/cell hover 색상을 token화한다.
5. `validation-modal.ts`, `grid-settings-dialog.ts`의 fieldset/legend 및 입력 필드 잔여 색상을 점검한다.

검증:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

시각 확인:

- 표 만들기 popup
- 미주 모양
- 문단 모양
- 글머리표 popup
- validation/grid 관련 dialog

완료 기준:

- 이슈 본문에 등록된 추가 후보의 라이트 UI surface 하드코딩이 정리된다.
- 실제 색상 견본은 테마에 의해 의미가 바뀌지 않는다.
- Stage 4 완료보고서를 작성하고 승인 요청한다.

## 7. Stage 5 — focused 회귀 가드 보강

목표:

- 수동 시각 확인에만 의존하지 않도록 주요 다이얼로그의 색상 정책을 DOM/e2e로 고정한다.
- 기존 `theme-mode.test.mjs`는 유지하고 필요한 범위에서 focused test를 추가한다.

작업:

1. 기존 `theme-mode.test.mjs`의 light/dark/theme-color 검증을 유지한다.
2. 필요 시 `dialog-theme.test.mjs`를 추가해 주요 computed style을 확인한다.
3. 최소 검증 후보:
   - dark에서 `.dialog-input` 배경이 흰색으로 남지 않음
   - `.eq-preview` 배경이 문서 종이 계열임
   - 쪽 테두리/배경 중앙 preview는 흰색 유지
   - UI 버튼/fieldset/legend는 dark token 계열
4. 테스트가 특정 다이얼로그 열기 절차에 과도하게 의존하면 DOM 기반 helper를 우선 사용한다.

검증:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

신규 테스트가 추가된 경우:

```bash
cd rhwp-studio && node e2e/dialog-theme.test.mjs --mode=headless
```

완료 기준:

- 자동화 가능한 핵심 색상 정책이 테스트로 고정된다.
- 테스트가 문서 내용 색상 반전을 요구하지 않는다.
- Stage 5 완료보고서를 작성하고 승인 요청한다.

## 8. Stage 6 — Chrome Auto Dark Mode 최종 점검

목표:

- Chrome `Auto Dark Mode for Web Contents` 환경에서 `보기 > 테마 > 밝게` 선택이 앱의 밝은 테마 의도를
  가능한 범위에서 보존하는지 확인한다.
- 앞선 UI 토큰 수정 이후에만 이 단계를 수행해 원인 분리를 명확히 한다.

작업:

1. `theme.ts`의 `color-scheme` 적용 방식이 명시적 `light` 선택을 충분히 표현하는지 점검한다.
2. 필요한 경우 `index.html`의 meta `color-scheme` 또는 root style 적용 순서를 조정한다.
3. `어둡게` 테마는 Chrome 강제 변환에 의존하지 않고 자체 dark token으로 표시되는지 확인한다.
4. `시스템` 테마는 OS 선호와 앱 dataset이 일관되는지 확인한다.
5. Chrome 실험 기능 자체를 앱이 변경하려 하지 않는다.

검증:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

수동 시각 확인:

- Chrome Auto Dark Mode 활성화
- rhwp-studio `보기 > 테마 > 밝게`
- rhwp-studio `보기 > 테마 > 어둡게`
- rhwp-studio `보기 > 테마 > 시스템 설정`

완료 기준:

- `밝게` 테마에서 앱의 light token 의도가 가능한 범위에서 유지된다.
- Chrome 전역 실험 기능 설정 변경은 요구하지 않는다.
- Stage 6 완료보고서를 작성하고 승인 요청한다.

## 9. Stage 7 — 초기 테마 bootstrap 보정

목표:

- `어둡게` 테마 저장 후 새로고침할 때 첫 paint 전에 dark token이 적용되도록 한다.
- 앱 모듈 초기화 전까지 light 기본 토큰이 잠깐 보이는 theme FOUC를 줄인다.

작업:

1. `index.html`의 stylesheet 로드 전에 작은 inline bootstrap script를 추가한다.
2. bootstrap script는 `localStorage.rhwp-settings`의 theme mode를 읽어 `data-theme-mode`, `data-theme-effective`, `color-scheme`, `theme-color`를 선반영한다.
3. `theme.ts`의 정식 테마 동기화는 기존처럼 앱 초기화 후 같은 값을 재동기화한다.
4. 초기 bootstrap 동작을 검증하는 focused e2e를 추가한다.

검증:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-bootstrap.test.mjs --mode=headless
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
```

완료 기준:

- 저장된 dark 테마가 DOMContentLoaded 시점부터 root dataset과 color-scheme에 반영된다.
- menu-bar 등 첫 paint 대상의 computed background가 dark token으로 계산된다.
- 기존 theme-mode, dialog-theme, auto-dark 회귀 테스트가 깨지지 않는다.
- Stage 7 완료보고서를 작성하고 승인 요청한다.

## 10. 최종 검증

필수:

```bash
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless
cd rhwp-studio && node e2e/theme-bootstrap.test.mjs --mode=headless
```

신규 focused e2e가 추가된 경우:

```bash
cd rhwp-studio && node e2e/dialog-theme.test.mjs --mode=headless
```

권장 시각 검증:

- 다크모드 표/셀 속성
- 다크모드 셀 테두리/배경
- 다크모드 수식 편집
- 다크모드 쪽 테두리/배경
- 다크모드 표 만들기 popup
- 다크모드 문단 모양/미주 모양
- Chrome Auto Dark Mode + `밝게` 테마

## 11. 완료 기준

- #1422 이슈 본문에 등록된 대표 화면의 다크모드 대비 문제가 해소된다.
- 문서 종이, 실제 문서 preview, 색상 견본의 의미가 유지된다.
- 수식 편집은 저장 색상 변경 없이 preview 가독성을 확보한다.
- Chrome Auto Dark Mode 환경에서 앱의 명시적 light 선택을 보존하려는 조치와 검증 기록이 남는다.
- 모든 단계 완료 후 최종 보고서 `mydocs/report/task_m100_1422_report.md`를 작성한다.

## 12. 승인 요청

위 계획대로 Stage 1부터 구현한다. 승인 전에는 rhwp-studio 소스 코드를 수정하지 않는다.
