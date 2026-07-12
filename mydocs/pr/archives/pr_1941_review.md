# PR #1941 Review

## 메타

| 항목 | 값 |
|------|----|
| PR | #1941 |
| 제목 | Task #1939: HWP5-origin HWPX render-diff 구조 불일치 수정 |
| 작성자 | jangster77 |
| base | devel |
| head | task/m100-1939-76076-renderdiff |
| 관련 이슈 | #1939 |
| 규모 | 작성 시점 참고값: 3 files, +122/-1 |
| mergeable | 작성 시점 참고값: MERGEABLE / BLOCKED |

최종 merge 조건은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인이다.

## 변경 범위

- `DocumentCore::reflow_zero_height_paragraphs()`의 TAC 표 높이 보정에서
  `LineSeg::missing_lineseg_placeholder()` 단일 문단을 제외한다.
- `tests/issue_1939.rs`를 추가해 76076 HWP5-in-.hwpx 샘플의
  `roundtrip_geom(..., Via::Hwpx)` strict 구조 안정성을 고정한다.
- `mydocs/working/task_m100_1939_stage1.md`에 원인 분석과 검증 결과를 기록한다.

## 원인 검토

#1939의 증상은 #1936 이후 페이지 수는 82쪽으로 맞지만, `render-diff --via hwpx` strict 기준에서
page 38/39 구조가 갈라지는 것이었다.

원본 HWP5-in-.hwpx의 해당 구간에는 TAC RowBreak 표 host 문단이 있고, HWP5 원본 문단에는
`line_segs`가 없다. HWP5-origin HWPX export는 이 의미를 보존하려고
`missing_lineseg_placeholder`를 출력한다. 이 marker는 HWPX 재파스 후 reflow gate에서만 쓰이고,
이후 제거되어 HWP5 원본과 같은 `line_segs.is_empty()` 경로를 타야 한다.

하지만 HWPX 로드 중 TAC 표 높이 보정이 marker의 `line_height`를 표 높이로 바꾸면서
`clear_missing_lineseg_placeholders()` 제거 조건을 깨뜨렸다. 그 결과 roundtrip 문서만 실제
LineSeg가 있는 것처럼 처리되어 page 38/39 경계가 밀렸다.

이번 수정은 샘플명, 페이지 번호, 임의 계수가 아니라 `LineSeg` marker의 문서 IR 의미에 근거한다.

## 렌더 영향과 시각 검증 판정

렌더/pagination 영향이 있는 변경이다. 다만 이번 PR의 merge 판단 근거는 기준 PDF와의 사람이 보는
visual sweep이 아니라, 같은 입력의 HWPX roundtrip 기하 구조가 보존되는지 확인하는 strict
`render-diff --via hwpx`이다.

실측 결과:

- 수정 전: page 38/39 구조 불일치, 최대 변위 642.53px, `STRUCT_MISMATCH`
- 수정 후: 페이지 수 A=82 B=82, 구조 불일치 페이지 0, `status: PASS`

따라서 `mydocs/pr/assets/`에 별도 visual PNG asset은 남기지 않는다. 검증 근거는
`tests/issue_1939.rs`와 stage 문서에 기록된 render-diff 결과다.

## 로컬 검증

- `env CARGO_INCREMENTAL=0 cargo build --bin rhwp`: 통과
- `target/debug/rhwp render-diff samples/issue1891/76076_regulatory_analysis.hwpx --via hwpx`: PASS
- `target/debug/rhwp export-hwpx samples/issue1891/76076_regulatory_analysis.hwpx tmp/issue1939/after/76076-roundtrip.hwpx --verify-pages`: 82쪽 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1939`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1891`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --test issue_1939 -- -D warnings`: 통과
- `git diff --check`: 통과
- 작업지시자 검증: `cargo test --profile release-test --tests` 통과

## 리스크

- TAC 표 host 문단의 LineSeg 높이 보정 경로를 건드리지만, 제외 대상은
  `missing_lineseg_placeholder` 단일 문단으로 한정된다.
- HWPX 일반 문서의 누락 LineSeg reflow 및 HWPX RowBreak 합성 보정 경로는 유지된다.
- #1891 HWP5-origin 샘플군 페이지 수 보존 테스트가 통과했다.

## 결론

merge 후보로 판단한다.

merge 전 조건:

- PR head 최신 커밋 기준 GitHub Actions 통과
- PR head에 review 문서와 오늘할일 기록 포함
- 작업지시자 최종 승인
