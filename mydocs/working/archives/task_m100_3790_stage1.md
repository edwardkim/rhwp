# task_m100_3790 Stage 1 결과 — CI 영향축 shadow classifier

- **Issue**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **연계 이슈**: [#3684](https://github.com/edwardkim/rhwp/issues/3684), [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **브랜치**: `codex/issue-3790-ci-impact-shadow`
- **기준**: `upstream/devel` `3d4863a0d531`
- **기록 시각**: 2026-08-02 KST
- **상태**: Stage 1 구현·focused 검증과 PR full CI 완료, review 보정 진행 중

## 1. 구현 결과

- 독립 영향축 `rust_required`, `frontend_mode`, `render_required`, `native_skia_required`,
  `codeql_languages`와 판정 상태·version·reason을 순수 Node 함수로 구현했다.
- #3785, #3656, #3670, #3672, #3690의 GitHub 실제 파일 목록을 JSON fixture로 고정했다.
- `CI preflight`가 PR/push 파일 목록을 별도 JSON 입력으로 만들고 classifier 결과를 `shadow_*` output과
  Job Summary에 기록하게 했다.
- shadow checkout은 classifier 한 파일만 sparse checkout하고 credential을 유지하지 않으며, classifier
  실행 step에는 GitHub token을 전달하지 않는다.
- 기존 review-only fast-pass에서는 shadow checkout·수집·분류를 생략해 빠른 경로의 비용을 유지한다.
- checkout·수집·classifier·summary 실패는 기존 CI를 실패시키지 않으며 출력 기본값은 모든 검증을 요구하는
  `full`이다.
- 기존 `frontend_required`, heavy worker 조건과 `Build & Test` aggregate 판정은 변경하지 않았다.
- pull request의 classifier authority는 실제 checkout ref에 맞춰 `pr-merge-advisory`로 기록한다.

## 1.1 활성화 전 확인된 축 계약

- `frontend_mode=unit`은 Node unit test만 뜻하지 않는다. Studio `tsconfig.json`이 전체 `src`를 포함하므로
  전역 `tsc --noEmit`과 전체 Studio unit test를 함께 실행해야 한다.
- Render Diff는 WASM Canvas visual diff뿐 아니라 Vite 앱 전체를 실행하는 CanvasKit readiness도 포함한다.
  현재 `render_required` 경로 규칙은 readiness boot 의존성을 완전히 표현하지 못하므로 advisory 상태로
  유지하고, 활성화 전 두 gate를 분리하거나 실제 의존성의 보수적 합집합으로 보정한다.
- `canvaskit` 문자열을 포함한 테스트는 render로 보면서 피시험 코드
  `src/core/canvaskit-document-preflight.ts`는 package로만 보는 이름 기반 비대칭도 활성화 전에 제거한다.

## 2. historical fixture 결과

| PR | 분류 | Rust | Frontend | Canvas | Native Skia | CodeQL |
| --- | --- | --- | --- | --- | --- | --- |
| #3785 | Studio shortcut | false | unit | false | false | JavaScript/TypeScript |
| #3656 | Studio command | false | unit | false | false | JavaScript/TypeScript |
| #3670 | hwpctl/public API | false | package | false | false | JavaScript/TypeScript |
| #3672 | Studio page renderer | false | unit | true | false | JavaScript/TypeScript |
| #3690 | Cargo·Rust·WASM mixed | full | package | true | true | JavaScript/TypeScript, Python, Rust |

## 3. fail-closed 경계

다음 입력은 `classification_status=full`과 모든 CodeQL 언어로 고정했다.

- `.github/**`, `Cargo.toml`, `Cargo.lock`, Rust toolchain
- classifier·fixture·workflow 계약 테스트 자체
- `src/main.rs`와 `src/wasm_api.rs`/`src/wasm_api/**`
- rename과 `previous_filename` 존재
- 빈·잘못된 파일 목록, PR 3,000개 및 push 300개 경계, 수집 오류
- 알려진 lane으로 분류하지 못한 경로

`src/main.rs`는 #3789의 render/export 진입점 모듈화가 완료될 때까지 full이다.

## 4. 검증

| 검증 | 결과 |
| --- | --- |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 19 passed / 0 failed |
| `python3 -m unittest scripts/tests/test_ci_impact_workflow.py scripts/tests/test_render_diff_workflow.py` | 6 passed |
| `actionlint .github/workflows/ci.yml` | 통과, 경고 없음 |
| `git diff --check` | 통과 |

검증 환경은 Node `v24.15.0`, actionlint `1.7.12`다. workflow 자체가 변경되므로 원격 PR에서는 기존 full
CI가 그대로 실행되어야 하며, shadow summary가 worker 실행 여부를 바꾸지 않는지 추가로 확인한다.

## 5. 다음 단계와 동기화 지점

1. Stage 1 draft PR에서 shadow summary와 현재 full CI 결과를 관찰한다.
2. false negative 없이 안정되면 frontend `unit|package|render`를 먼저 활성화한다.
3. 다음 PR에서 Rust·Native Skia 조건화를 활성화한다.
4. #3684는 #3810으로 완료됐으며 정리 후 cache 기준선은 4.73GB다. Stage 4 뒤 이 기준선의 회귀 여부를
   확인한다.
5. cache 총량 회귀가 없으면 별도 메인터너 대기 없이 CodeQL 언어 분리 단계로 진행한다.

각 활성화 PR의 push·review·merge는 별도 승인 게이트를 따른다.
