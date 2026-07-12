# PR #2194 처리 계획

## 목적

#2185의 한글 줄 나눔 bit7 소비 의미 반전 수정 PR을 검토하고, merge 전 필요한 보정 범위를 정한다.

## 확인한 커밋

| SHA | 제목 | 비고 |
|---|---|---|
| `a58d828f34cdb4bb53a02c42e381137c6de4e31d` | `Task #2185: 한글 줄 나눔 단위 의미 복구` | 핵심 코드 수정 |
| `7ae9744d965c7f609d3b9e1d5b50f565b140d589` | `Task #2185: 실제 편집 저장 회귀 테스트 추가` | HWP/HWPX 통합 테스트 |
| `a07f663a39cb03b13f5a9f2ef70395b9f495157c` | `Task #2185: WASM 및 Studio 회귀 검증` | 문서/검증 |
| `8e39cf4906efd0a57385c3ad39270ae78b2335e5` | `Task #2185: 한글 줄 나눔 의미 반전 수정 완료` | 최종 보고 |
| `ccbfe5e93bd9a0094d961de116f95d9040a78e51` | `Merge branch 'devel' into issue-2185-korean-break-unit` | 최신 devel 반영 |
| `cb51cdfccb1f3f3d2d33958c247766660d5a3e96` | PR #2194 merge commit | merge 완료 |

## Stage

1. reviewer assign 완료
2. PR 메타, 관련 이슈 #2185/#2193 확인
3. diff 검토
4. focused 로컬 검증
5. visual sweep 대표 페이지 확인
6. 문서 보존 finding 및 후속 후보 기록
7. 작업지시자 결정에 따라 #2194 먼저 merge, 옵션 2 문서 PR로 검토 기록/증적 보존

## 검증 요약

- GitHub Actions: 최신 head 기준 pass
- 로컬:
  - `git diff --check upstream/devel...HEAD`
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2185_korean_break_unit -- --nocapture`
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test korean_break_unit -- --nocapture`
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test test_tokenize_korean -- --nocapture`
  - `CARGO_INCREMENTAL=0 cargo build`
- Visual:
  - `samples/issue1949_giant_cell_nested_tables_perf.hwp`
  - `pdf/issue1949_giant_cell_nested_tables_perf-2020.pdf`
  - p1 flagged 0/1, SVG/PDF pages 115/115

## 처리 결정

작업지시자 검증 결과 PR 본래 목적은 달성했다. 따라서 #2194는 최신 CI 통과 상태에서
`cb51cdfccb1f3f3d2d33958c247766660d5a3e96`로 먼저 merge했고, 다음 항목은 옵션 2 문서 PR로 처리한다.

1. `mydocs/orders/20260711.md`
   - #2196 항목을 복구한다.
   - #2185/#2194 항목은 별도 섹션 또는 같은 M100 섹션으로 추가한다.
2. `mydocs/tech/hwp_spec_errata.md`
   - HWPX `breakNonLatinWord` 설명과 한컴 실동작 불일치 항목을 추가한다.
3. `mydocs/tech/한글문서파일형식_5.0_revision1.3.md`
   - 표 44의 bit 7 계약 근처에 HWPX 명칭과 내부 bit 의미를 섞지 말라는 보완 주석을 남긴다.
4. PR review 문서와 visual sweep asset을 archive로 보존한다.
5. 추가 repaint/표시 지연 후보 증적을 보존한다.
6. PR 본문이 `Refs #2185`이므로 merge 후 #2185를 수동 close한다.

## 추가 보완 후보 증적

사용자가 #2194 본래 수정과 별개로 다음 repaint 후보를 확인했다.

- 줄 끝까지 `1`을 입력한 뒤 캐럿이 마지막 줄 끝에 있음
- 이어서 `1`을 계속 입력하면 캐럿과 추가 입력분이 즉시 표시되지 않음
- 시간이 지나면 입력분이 사라진 것처럼 보임
- `1.1.1` 마지막 문단에서 Enter를 치면 다시 표시됨

증적 파일:

- `mydocs/pr/assets/pr_2194/pr_2194_followup_01_line_end_cursor.png`
- `mydocs/pr/assets/pr_2194/pr_2194_followup_02_extra_input_not_painted.png`
- `mydocs/pr/assets/pr_2194/pr_2194_followup_03_delayed_text_disappears.png`
- `mydocs/pr/assets/pr_2194/pr_2194_followup_04_enter_repaints.png`

## 후속 처리 메모

- #2185는 정확성 이슈이므로 PR #2194 merge 후 close 대상이다.
- #2193은 성능 후속 이슈이므로 open 유지한다.
- visual asset과 추가 repaint 증적은 `mydocs/pr/assets/pr_2194/`에 준비했다.
- 작업지시자 결정에 따라 merge 후 별도 docs-only PR, 즉 옵션 2로 보존한다.
