---
kind: pr_review
status: completed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-01
---

# PR #6558 검토 — 병합 셀 경계 드래그 리사이즈

## 결론

**승인.** 최신 head `6ae289ae102b688086125c0960e31e213173eb30`은 이전 검토에서
요청한 E2E canonical 등록과 npm 실행 배선을 보완했고, 그 사이 발견한 가로 병합 셀의
행 경계 보상 누락도 같은 “병합 셀 시작 좌표만 보는 맹점”으로 좁혀 별도 회귀 E2E로
고정했다.

최신 `devel`과의 merge simulation, #6557 Rust focused test, fresh Docker WASM,
Studio typecheck·unit·build, 두 browser E2E와 생성 PNG의 직접 판정이 모두 통과했다.
이 문서의 `승인`은 검토 판정이며 GitHub approve 또는 merge 실행 승인이 아니다.

## 병합 결과

- PR #6558은 2026-09-01T08:10:05Z에 maintainer `edwardkim`이 일반 merge commit 방식으로 병합했다.
- contributor head: `6ae289ae102b688086125c0960e31e213173eb30`
- source merge commit: `d776fdb28796f1c561e3b7e9c027b0fad8ae4bcc`
- squash 및 `--admin` 우회는 사용하지 않았다.
- 이 archive와 두 PNG는 source merge 뒤 option M 문서 전용 기록으로 반영한다.

## 라우팅

- 기본 경로: `maintainer_general.md`
- 보조 경로: `intake_and_review.md`, `local_validation.md`,
  `visual_fixture_evidence.md`, `rework_and_exceptions.md`, `post_merge.md`
- 함께 확인: `pr_review_workflow.md`, `pr_review/README.md`, `CONTRIBUTING.md`,
  `dev_environment_guide.md`, `e2e-cdp.md`
- 작성자는 기존 기여자이므로 `first_time_contributor.md`는 적용하지 않았다.

## 메타데이터와 범위

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#6558](https://github.com/edwardkim/rhwp/pull/6558) / @jeong-sik |
| 관련 이슈 | [#6557](https://github.com/edwardkim/rhwp/issues/6557) (`closes #6557`) |
| base / draft | `devel` / 아님 |
| 검토 code candidate | `6ae289ae102b688086125c0960e31e213173eb30` |
| 검토 기준 devel | `8b17a07737e6b72910a0b7c422cdf22ac628d759` |
| 변경 규모 | 9 files, `+647/-79`, 5 commits |
| merge simulation | 충돌 없음, 결과 tree `a395da6e3200332318bdedfee5c068b2201b4a20` |
| GitHub 참고 상태 | `MERGEABLE`, `CLEAN`; merge 직전 재확인 필요 |
| reviewer | `edwardkim` 요청 상태 확인 |

변경은 Studio의 표 선택·일반 경계 resize update 구성, Rust 표 resize 마킹 조건,
Studio 단위·브라우저 E2E, Rust integration 회귀 원본, E2E manifest와 npm script에
한정된다. dependency 버전과 package lock은 바꾸지 않았다.

## 검토 경과와 원인 계층

첫 head `6f540c743`은 선택 범위와 병합 셀의 실제 span이 겹치는지를 공용 함수로
판정하게 했다. 후속 `05baacf3f`는 세로 병합 셀이 걸친 모든 행의 오른쪽 이웃을
보상하고, Rust 엔진이 실제 base grid와 갈라진 행·열만 `local_resize`로 마킹하도록
고쳤다. `e8cad78e9`는 새 Rust 회귀를 `src/** #[cfg(test)]`가 아니라
`tests/cases/` integration 원본으로 옮겼다.

추가 head `3b3b03fe8`은 가로 병합 셀의 일반 행 경계 드래그에서도 시작 열의 이웃만
찾던 같은 결함을 모든 겹친 열의 이웃 탐색으로 확장했다. 마지막 `6ae289ae1`은 두
브라우저 E2E를 `rhwp-studio/e2e/MANIFEST.md`와 `package.json`에 각각 독립 실행
명령으로 등록했다.

코드 검토에서 확인한 보호 불변식은 다음과 같다.

- 병합 셀 선택은 bbox 시작 좌표가 아니라 row/column span의 실제 겹침으로 판정한다.
- 열 경계는 병합 셀이 걸친 모든 행, 행 경계는 병합 셀이 걸친 모든 열의 반대편
  이웃에 보상 delta를 적용한다.
- 결과가 전 행·열에서 균일하면 거짓 `local_resize` 마킹을 만들지 않는다.
- 결과가 실제로 갈라진 행·열은 기존처럼 `local_resize`로 남는다.
- 병합 없는 표의 단일 이웃 보상 계약은 유지한다.

## 로컬 검증 결과

### GitHub Full CI 재사용

code candidate와 GitHub 녹색 head는 모두 `6ae289ae1`이며, 메인터너가 source·test·fixture·
workflow를 보정하지 않았다. 최신 `devel` current-base merge도 충돌 없이 통과했다.
따라서 `pr_review_workflow.md` 3.2.2에 따라 전체 release-test nextest와 광범위
Native Skia의 로컬 중복 실행은 생략하고, 핵심 focused·WASM·browser 검증을 수행했다.

- [CI run 33480330964](https://github.com/edwardkim/rhwp/actions/runs/33480330964):
  Lint, 4 archive build/shard, Build & Test, Frontend package gates 성공
- [CodeQL run 33480330937](https://github.com/edwardkim/rhwp/actions/runs/33480330937):
  Rust·Python·JavaScript/TypeScript 성공
- [Render Diff run 33480330762](https://github.com/edwardkim/rhwp/actions/runs/33480330762): 성공
- [Proptest run 33480331029](https://github.com/edwardkim/rhwp/actions/runs/33480331029): 성공
- [Adapter inter-diff run 33480331024](https://github.com/edwardkim/rhwp/actions/runs/33480331024): 성공

### Rust와 정책

| 검증 | 결과 |
| --- | --- |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `rust-test-suite-manifest.mjs --prepare/--check` | 통과: 1,098 sources, 4,762 static attrs, 48 integration targets |
| `rust-unit-test-tiers.mjs --check` | 통과: 4,221 tests / 299 modules |
| `cargo fmt --all -- --check` | 통과 |
| #6557 focused nextest | 통과: 2/2, 162 skipped, cold build 2분 42초 |

focused test는 “세로 병합 결과가 균일하면 base grid 유지”와 “실제로 갈라진 행은
계속 local resize로 마킹”하는 대조군을 함께 통과했다. 로컬 nextest는 0.9.137로
프로젝트 권장 0.9.140보다 낮다는 경고와 skipped-report 설정 경고를 냈으나 테스트
실패는 아니었다.

### WASM과 Studio

| 검증 | 결과 |
| --- | --- |
| `docker compose --env-file ... run --rm wasm` | 통과, 6분 45초 |
| fresh `pkg/rhwp.js` / `pkg/rhwp_bg.wasm` | 423 KiB / 9.5 MiB |
| `(cd rhwp-studio && npx tsc --noEmit)` | 통과 |
| `npm --prefix rhwp-studio test` | 통과: 1,337 pass / 1 skip / 0 fail |
| `npm --prefix rhwp-studio run build` | 통과, Vite 245 modules |
| `python3 scripts/check_e2e_manifest.py` | 통과: tracked 124 / manifest 124 |
| `e2e:issue-6557-merged-col` | 통과: 열 경계 440.7 == 440.7, 두 대상 모두 +47.6px |
| `e2e:issue-6557-merged-row` | 통과: 표 높이 194.2 → 194.2, row1 50.0/50.0/50.0 |

`npm ci`는 3 vulnerabilities(낮음 1, 높음 2)를 보고했지만 이 PR은 dependency나
lockfile을 바꾸지 않고 두 npm script만 추가한다. Vite의 기존 CanvasKit `fs`·`path`
externalization과 대형 chunk 경고도 build 실패가 아니며 이 PR의 resize 변경과 무관하다.

## 직접 브라우저 판정과 증적

두 E2E는 fresh WASM을 로드한 headless Chrome에서 실제 mousedown·mousemove·mouseup을
발생시키고 최종 Canvas 화면을 1280×900 PNG로 저장했다. 첫 실행 테마 선택은 E2E가
`시작하기` 버튼을 닫도록 명시해 이전 timeout 원인을 제거한다.

| 축 | 임시 E2E PNG | SHA-256 | 사람 판정 |
| --- | --- | --- | --- |
| 열 | `rhwp-studio/e2e/screenshots/merged-cell-boundary-drag-after.png` | `bd8bca1838a73f13ca5f4ecb17ac55b9c214a76de24836da141f81506bfe5d24` | 세로 병합 셀과 하단 일반 셀의 오른쪽 경계가 한 직선이며 선택 음영·괘선에 겹침·붕괴 없음 |
| 행 | `rhwp-studio/e2e/screenshots/merged-cell-row-boundary-drag-after.png` | `9bde59760616dee187221875126638d4a161815fb3df578f71d0d069ac835366` | 가로 병합 셀 아래 세 칸의 y·높이가 같고 표 외곽 높이 보존, 겹침 없음 |

merge 후 보존할 안정 경로는 다음과 같다. source merge 뒤 option M archive commit에 같은 SHA-256으로 반영하며 원 contributor
source head에는 섞지 않는다.

- `mydocs/pr/assets/pr_6558_merged_col_boundary_review.png`
- `mydocs/pr/assets/pr_6558_merged_row_boundary_review.png`

이 PR은 HWP/HWPX fixture나 페이지 렌더링을 바꾸지 않으므로 PDF visual sweep과 한컴
오라클 대조는 적용하지 않았다. 판정 범위는 합성 표에서의 Studio 사용자 상호작용과
최종 Canvas geometry다.

## 잔여 위험과 범위 밖 사항

- 이웃 탐색은 resize 완료 시 bbox를 선형 순회한다. drag 1회당 실행되는 제한된 경로이며
  dependency·렌더 loop를 늘리지 않아 별도 성능 blocker로 보지 않는다.
- E2E는 빈 합성 표를 사용하므로 복잡한 실문서의 모든 병합 topology를 증명하지 않는다.
  다만 세로·가로 병합, 선택 경로·일반 경로, 균일·갈라진 결과와 병합 없는 대조군을
  함께 고정해 이번 issue의 네 결함 계층을 직접 덮는다.
- 비공개 10k 코퍼스는 사용하지 않았다. 이번 수정은 공개 가능한 합성 표와 결정적
  자동 테스트로 재현 가능하다.

## 최종 판정

- 판정: **승인**
- 대상: contributor code candidate `6ae289ae102b688086125c0960e31e213173eb30`
- merge 전 조건: 최신 PR head가 위 SHA와 같고, GitHub required checks가 실패·대기 없이
  성공하며, `MERGEABLE`·`CLEAN`을 재확인하고 작업지시자가 merge를 승인할 것
- merge 방식: 작업지시자 승인 뒤 maintainer 일반 경로의 정상 merge commit 방식
- merge 후: merge SHA 확인 → `devel` 동기화 → review 문서 archive와 두 asset 반영 →
  #6557 상태 확인·필요 시 close/comment → contributor comment → worktree·branch 정리
- 원격 조치: 이 문서 갱신만으로 approve, comment, push 또는 merge를 수행하지 않는다.

## Merge 후 contributor PR comment 계획

source merge SHA와 asset record commit SHA가 확정되고 위 두 asset이 `devel`에 반영된 뒤에만 다음 내용을 PR comment로
게시한다.

- 원 head `6ae289ae1`과 merge commit 연결
- Rust focused 2/2, Studio 1,337 pass, Docker WASM, 두 E2E의 실제 수치
- 열 경계 `440.7 == 440.7`, 행 높이 `194.2 → 194.2`와 사람 판정의 범위
- 아래 merge-SHA 고정 raw image URL 두 개
- UTF-8 without BOM `--body-file` 게시 후 API로 한글·BOM·`??` 치환 검증

```text
https://raw.githubusercontent.com/edwardkim/rhwp/<asset-record-commit-sha>/mydocs/pr/assets/pr_6558_merged_col_boundary_review.png
https://raw.githubusercontent.com/edwardkim/rhwp/<asset-record-commit-sha>/mydocs/pr/assets/pr_6558_merged_row_boundary_review.png
```
