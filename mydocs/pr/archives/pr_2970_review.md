# PR #2970 개별 검토 기록

## PR 정보

| 항목 | 내용 |
|---|---|
| PR 번호 | [#2970](https://github.com/edwardkim/rhwp/pull/2970) |
| 작성자 | [@kevin9327](https://github.com/kevin9327) (기존 기여자) |
| 제목 | fix(hwpx): borderFill threeD/shadow 속성이 저장 시 0으로 고정되던 문제 수정 (#2965) |
| base ← head | `devel ← task/m100-2965-borderfill-3d-shadow` |
| 변경 규모 | +103 / -2 (2 파일) |
| 관련 이슈 | [#2965](https://github.com/edwardkim/rhwp/issues/2965) |
| 문서 작성 시점 참고값 | merge state: `BEHIND`, maintainer_can_modify: `true` |

## 변경 범위

- 분류 축: **HWPX parser/serializer**
- PR 제목·변경 규모·첨부된 task report를 기준으로 범위를 확인했다.
- `mydocs/report/task_m100_*.md`가 포함된 경우 파일명과 본문의 이슈 번호 대조는 최종 merge 전 다시 확인한다.

## 렌더 영향·시각 검증

현 단계에서는 필수 판정 없음. merge simulation 중 layout/paint 영향이 확인되면 visual sweep으로 재분류합니다.

## 사전 검증

- 대량 PR 사전 분류: `scripts/pr_triage.sh kevin9327` 실행 완료.
- CI: 문서 작성 시점 참고값: GitHub required checks 성공. 최종 merge 전 최신 head 기준으로 재확인해야 합니다.
- 실제 merge simulation (2026-07-23, upstream/devel `cbddc1cd8708`): **CLEAN**. focused test와 최신 CI 확인은 이 문서의 권고 조건에 따라 별도 수행한다.
 - 누적 통합 검증: `review/kevin9327-candidates-integration-20260723`에서 오래된 PR 순서 36/54로 실제 병합 완료 (검토용 merge commit `99b909081798`). 이 브랜치는 merge 대상이 아니라 상호작용·테스트 확인용 임시 브랜치다.
 - 누적 공통 게이트: `cargo test --lib --quiet` 2551 passed/7 ignored, `cargo test --profile release-test --tests --quiet` 완료, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `git diff --check`, `cargo test --doc --quiet`(0 passed/1 ignored), `wasm-pack build --target web --out-dir pkg`, `npx tsc --noEmit`, `npm test`(535 passed)을 통과했다.
 - 누적 Rust/Studio 검증 결과는 이 PR 하나의 승인 근거가 아니며, 최신 head·개별 CI·의존 PR 상태를 merge 직전에 다시 확인해야 한다.
 - volatile 값(merge state·CI·head)은 문서 작성 시점 참고값이며, 최종 판단 근거로 고정하지 않는다.

## 검토 결과·리스크

사전 diff 분류에서 별도 차단 사유는 발견하지 못했습니다. 단, 개별 로컬 merge simulation과 focused test는 아직 완료되지 않았습니다.

## 처리 권고

**검토 계속 — 현 문서는 사전 개별 기록입니다. 최신 `devel` merge simulation, focused test 및 최신 CI 확인 전에는 승인·merge를 권고하지 않습니다.**

최종 merge 조건은 최신 PR head의 required checks 통과, 최신 `devel` 기준 병합 가능 상태, 필요한 focused/visual 검증 완료 및 작업지시자 승인이다.
