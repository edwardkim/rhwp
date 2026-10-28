# Task #1280 3단계 완료보고서 — e2e 회귀 테스트 + 통합 검증

## 목표

`enterTextboxPlacementMode()` + 마우스 드래그(=실제 프런트 삽입 경로)로 글상자를 만든 뒤
텍스트 입력이 되는지 end-to-end로 검증하는 e2e 회귀 테스트를 추가하고, WASM 빌드로 통합 검증한다.

## 변경 내용

**신규 파일**: `rhwp-studio/e2e/issue-1280-textbox-text-input.test.mjs` (puppeteer-core, `helpers.mjs` 패턴)

`window.__inputHandler.enterTextboxPlacementMode()` 호출 → `#scroll-container canvas`에 `page.mouse`
드래그 → `finishTextboxPlacement` 경로 구동. WASM `createShapeControl` 직접 호출이 아니라 **실제 프런트
경로**를 타므로 #1280(프런트 shapePlacementType) 회귀를 실제로 잡는다.

검증 단계 4종:
1. `enterTextboxPlacementMode()` 직후 `shapePlacementType === 'textbox'` (정확한 회귀 가드 — 수정 전 'rectangle')
2. 마우스 드래그 → 글상자 도형 생성·선택 (`cursor.getSelectedPictureRef()` → `type === 'shape'`)
3. `insertTextInCell(...)`가 throw하지 않음 (= text_box 존재의 결정적 증거; 수정 전엔 "텍스트 박스가 없습니다"로 실패)
4. `getTextInCellByPath(...)`로 글상자 내부 첫 문단 텍스트 보존 확인

## 검증

WASM 빌드(Docker):
```
[INFO]: Optimizing wasm binaries with `wasm-opt`...
[INFO]: :-) Your wasm pkg is ready to publish at /app/pkg.   # exit 0
```

e2e 실행 (Vite :7700 + headless Chrome):
```
=== E2E: #1280: 삽입한 글상자에 커서 진입·텍스트 입력 ===
  PASS: 글상자 배치 모드의 shapePlacementType === 'textbox' (실제 'textbox') — #1280 핵심 회귀
  PASS: 드래그로 글상자 도형이 생성·선택됨 (ref={"sec":0,"para":0,"ctrl":2,"type":"shape"})
  PASS: 글상자에 텍스트 입력 성공 (#1280 — 수정 전엔 "텍스트 박스가 없습니다"로 실패)
  PASS: 글상자 내부 첫 문단 텍스트 보존 (readBack="글상자 텍스트")
```

**4개 단언 전부 통과** ✓ — 삽입한 글상자가 text_box를 가진 채 생성되고 텍스트 입력이 정상 동작한다.

## 환경 메모 (이 윈도우 컨트리뷰터 머신 한정 — 본 수정과 무관)

이 머신은 rhwp-studio 프런트 개발 환경이 완전히 갖춰져 있지 않아, e2e 실행 위해 다음을 보강했다:

1. **MSVC C++ 빌드 도구 + Windows SDK 설치** (네이티브 cargo 빌드용, 2단계에서 수행).
2. **rolldown 네이티브 바인딩 누락** — npm optional-deps 버그(npm/cli#4828)로 `@rolldown/binding-win32-x64-msvc`
   가 빠져 Vite가 기동 실패. `npm install --no-save @rolldown/binding-win32-x64-msvc@1.0.3`로 보강(package-lock 무변경).
3. **Node 20.16.0** — Vite 8은 20.19+ 권장(경고만, 기동에는 문제 없음).
4. **TestReporter HTML 생성 크래시(Windows 전용, 하네스 버그)** — `helpers.mjs`의 `getReportFilename()`이
   `process.argv[1]`(Node가 절대경로·역슬래시로 해석)에 `split('/')`(POSIX 가정)을 적용해 보고서 경로가
   깨진다. **4개 단언이 모두 통과한 이후** 발생하므로 검증 결과와 무관하며, WSL2/CI(POSIX)에서는 정상.
   본 #1280 범위 밖이라 하네스 코드는 수정하지 않았다(별도 처리 권장).

## 산출물

- 신규 e2e: `rhwp-studio/e2e/issue-1280-textbox-text-input.test.mjs`
- 스크린샷: `e2e/screenshots/issue-1280-01-textbox-created.png`, `issue-1280-02-text-inserted.png` (gitignore)

## 다음 단계

최종 결과보고서(`report/task_m100_1280_report.md`) 작성 → 승인 → 컨트리뷰터 PR(upstream devel).

## 승인 대기

본 보고서와 e2e 테스트 커밋 후 승인을 요청한다.
