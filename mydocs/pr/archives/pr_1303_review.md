# PR #1303 리뷰 - 미주 다줄 문단 다음 같은 미주 연속 문단 줄간격 과소 수정

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1303 |
| 제목 | fix(#1302): 미주 다줄 문단 다음 같은 미주 연속 문단 줄간격 과소 수정 |
| 작성자 | planet6897 |
| 대상 이슈 | #1302 |
| 대상 브랜치 | `devel` |
| 검토 기준 | `local/pr1303-upstream` |

## 2. 변경 범위

| 파일 | 판단 |
|---|---|
| `src/renderer/height_cursor.rs` | 실제 렌더링 vpos 보정 |
| `tests/issue_1139_inline_picture_duplicate.rs` | #1302 회귀 핀 테스트 |
| `mydocs/plans/task_m100_1302.md` | 기여자 작업 문서 |
| `mydocs/plans/task_m100_1302_impl.md` | 기여자 작업 문서 |
| `mydocs/working/task_m100_1302_stage1.md` | 기여자 작업 문서 |
| `mydocs/working/task_m100_1302_stage2.md` | 기여자 작업 문서 |
| `mydocs/working/task_m100_1302_stage3.md` | 기여자 작업 문서 |
| `mydocs/report/task_m100_1302_report.md` | 기여자 작업 문서 |

실제 수용 대상은 코드 1개와 테스트 1개로 충분하다. 기여자 작업 문서는 현재 저장소의 `mydocs` 운영 흐름과 맞지 않으므로 통합에서 제외하는 것이 안전하다.

## 3. 문제 구조

문제 샘플은 `samples/3-11월_실전_통합_2022.hwp` 18쪽 좌측 단 미주 영역이다.

다줄 미주 문단 `pi=852`의 마지막 줄 다음에 같은 문제의 연속 문단 `pi=853`이 이어질 때, 저장된 vpos delta는 `prev_lh + prev_ls` 수준의 정상 한 줄 전진을 가리킨다. 하지만 `compact_endnote_page_tail_backtrack`이 컬럼 하단 page-path 조건에서 이를 overlap tail로 오인해 trailing 줄간격을 깎으면서 문단간 간격이 좁아진다.

PR의 수정은 다음 조건을 추가한다.

- 현재 문단이 수식-only tail이면 기존 frame-fit backtrack을 유지한다.
- 그 외 breakable text에서 `curr_first_vpos - prev_vpos >= prev_lh + prev_ls`이면 정상 한 줄 전진으로 보고 page-tail backtrack을 비활성화한다.

이 방향은 타당하다. 기존 #1274 계열 atomic 수식 tail을 제외한 점도 중요하며, 이 PR이 단순히 backtrack 전체를 끄지 않는 점이 수용 가능한 범위다.

## 4. 검증 결과

PR head(`local/pr1303-upstream`)에서 직접 실행:

```text
cargo fmt --all -- --check
통과

cargo test --test issue_1139_inline_picture_duplicate issue_1302_2022_nov_page18_multiline_endnote_continuation_keeps_line_spacing -- --nocapture
1 passed

cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
52 passed

cargo clippy --lib -- -D warnings
통과

cargo test
통과
```

전체 테스트 중 확인된 주요 수치:

- lib tests: 1576 passed, 0 failed, 6 ignored
- `issue_1139_inline_picture_duplicate`: 52 passed
- 이후 integration/doc tests 포함 전체 명령 종료 코드 0

## 5. 시각 판정용 산출물

PR head 기준으로 생성:

| 파일 | 용도 |
|---|---|
| `output/poc/pr1303-endnote-spacing/hwp-page18/3-11월_실전_통합_2022_018.svg` | 일반 렌더 시각 판정 |
| `output/poc/pr1303-endnote-spacing/hwp-page18-debug/3-11월_실전_통합_2022_018.svg` | 문단/표 디버그 오버레이 확인 |

## 6. 검토 의견

수정은 좁은 조건 게이트이며, 기존 page-tail backtrack의 목적 자체를 제거하지 않는다. 특히 수식-only tail을 제외한 덕분에 #1274류 frame-fit 회귀 위험을 낮춘다.

잠재 리스크는 조건식이 `seg.vertical_pos`와 `curr_first_vpos`의 차이를 직접 비교한다는 점이다. PR head 자체에서는 주변 테스트 파일 전체와 전체 `cargo test`가 통과했다.

통합 중 최신 `local/devel`의 #1284 회귀 핀(`issue_1284_2024_between20_page13_question_flow_matches_pdf`)이 한 차례 실패했다. 원인은 PR의 full-advance gate가 `문...` 문항 제목에도 적용되어 기존 title-bottom 보정 경로를 우회한 것이다. #1302 대상은 비제목 연속 문단이므로, 통합본에서는 `!current_is_endnote_title` 조건을 추가해 gate 범위를 breakable 비제목 텍스트 연속 문단으로 좁혔다. 보강 후 #1284, #1302, 미주 주변 테스트 68개와 전체 `cargo test`가 통과했다.

## 7. 권장 처리

권장안: **수용**

- #1302 증상을 직접 고정하는 회귀 테스트가 추가되었다.
- 미주/페이지 tail 주변 테스트 52개가 통과했다.
- 전체 `cargo test`와 clippy가 통과했다.
- 실제 반영은 `src/renderer/height_cursor.rs`, `tests/issue_1139_inline_picture_duplicate.rs`만 가져오는 방식이 좋다.

권장 절차:

1. 메인테이너 시각 판정
2. `local/devel` 기준 통합 브랜치 생성
3. PR의 코드/테스트 변경만 적용
4. 통합 시 `current_is_endnote_title` guard를 포함해 #1284 title-bottom 보정 회귀 방지
5. `cargo fmt --all -- --check`
6. `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
7. `cargo clippy --lib -- -D warnings`
8. `cargo test`
9. 결과 보고서 작성 후 승인 게이트

## 8. PR 코멘트 초안

```markdown
검토했습니다. 다줄 미주 문단 다음의 같은 미주 연속 문단에서 stored vpos가 정상 한 줄 전진을 나타내는데도 page-tail backtrack이 trailing을 깎는 문제를 잘 좁혀 처리한 것으로 확인했습니다.

로컬에서 다음 검증을 통과했습니다.

- `cargo fmt --all -- --check`
- `cargo test --test issue_1139_inline_picture_duplicate issue_1302_2022_nov_page18_multiline_endnote_continuation_keeps_line_spacing -- --nocapture` (1 passed)
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` (52 passed)
- `cargo clippy --lib -- -D warnings`
- `cargo test`

메인테이너 시각 판정 후 코드/테스트 중심으로 반영하겠습니다. 감사합니다.
```
