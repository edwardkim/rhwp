# PR #2168 통합 검토 - 80168 잔여 축 분석과 Windows 오라클 도구

- PR: https://github.com/edwardkim/rhwp/pull/2168
- 관련 이슈: #2148
- 작성자: `planet6897`
- reviewer: `jangster77` 지정 완료
- base/head: `devel` <- `pr/task2148` (원 PR), 현재는 통합 브랜치에서 검토
- 원 PR 참고 head: `4416b4057dd7d9f690eec1d703cfd21290d73876`
- 통합 기준: `8d695deef` + PR #2168 cherry-pick `ea5707554`, `fa6d9655c`, `066c9abd7`
- 문서 작성 시점 참고 상태: 원 PR `CLEAN`; CI/CodeQL 통과, merge 전 통합 PR CI 재확인 필요
- 작성일: 2026-07-10

## 결론

**merge 후보.** 이 PR은 renderer 동작을 바꾸지 않고, 80168의 잔여 페이지 불일치가
단일 원인이 아니라 em 줄높이·NO_LS 셀 중첩 표·래핑 폭의 세 축 상쇄임을 재분류한다.
보고서와 stage 문서는 모두 "래핑 폭 해소 전 랜딩 금지"라는 같은 결론을 유지한다.

## 변경 범위

- `tools/hangul_row_heights2.py`: Windows Hancom COM/`pyhwpx` 오라클에 `--col`, `--pi`와
  본문 표 판별자를 추가한다.
- `mydocs/working/task_m100_2148_stage{1,2,3}.md`, 최종 보고서: 가설 기각과 세 참값 축을 기록한다.
- production renderer, WASM API, golden fixture에는 변경이 없다. 따라서 이 PR 자체는 visual sweep 대상이 아니다.

## 검증

- 원 PR CI, CodeQL, Canvas visual diff: 통과 (문서 작성 시점 참고값).
- `python3 -m py_compile tools/hangul_row_heights2.py`: 통과.
- `python3 tools/hangul_row_heights2.py --help`: `--col`, `--pi`를 포함한 CLI 인자 확인.
- Windows Hancom COM/`pyhwpx` 실제 오라클 실행은 현재 macOS 통합 검증 환경 밖이다. 도구는
  진단용이며 production path를 변경하지 않으므로, 통합 PR의 renderer 회귀 판단 근거로 사용하지 않는다.
- 깨끗한 `target` 전체 사전 검증: `cargo build --release`, `cargo test --release --lib`
  (2191 passed), `cargo test --profile release-test --tests`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --doc`, Studio `npx tsc --noEmit`/`npm test` (183 passed), WASM build 모두 통과.

## 잔여

80168의 페이지 정합은 래핑 폭 축을 해결한 뒤에만 em/중첩 표 보정을 함께 랜딩할 수 있다.
이 PR은 그 전제와 재현 도구를 보존하며 #2148을 close하지 않는다.

## 최종 조건

1. 통합 PR의 최신 CI·CodeQL을 확인한다.
2. Windows에서 이 도구를 실제 오라클로 사용할 때는 한컴/`pyhwpx` 환경에서 대상 HWP와
   `--exe` 경로를 명시해 별도 재현 기록을 남긴다.

## 옵션 1 기록

이 문서와 오늘할일 `mydocs/orders/20260710.md`를 통합 PR에 함께 포함한다. 시각 출력 경로를
바꾸지 않는 분석/도구 PR이므로 별도 review PNG는 만들지 않는다.

## Merge 결과

- 통합 PR [#2170](https://github.com/edwardkim/rhwp/pull/2170)은 2026-07-10에 merge commit
  `c95d8fd743ae4cfcbcbb0e26444ebef4e42b84ba`로 `devel`에 반영됐다.
- 최신 head CI, CodeQL, Render Diff는 모두 성공했다. 갱신 전 head `19abd763b`의 CI/CodeQL/Render Diff는
  force-cancel 후 `completed/cancelled`를 확인했다.
- #2148은 래핑 폭 축이 남아 있으므로 auto-close하지 않고 open으로 유지한다.
