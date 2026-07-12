# PR #2194 리뷰 - 한글 줄 나눔 단위 의미 반전 정정

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/2194 |
| 제목 | `fix(renderer): 한글 줄 나눔 단위 의미 반전 정정 (#2185)` |
| 작성자 | `postmelee` |
| base | `devel` |
| head | `postmelee:issue-2185-korean-break-unit` |
| 관련 이슈 | #2185, 후속 #2193 |
| 규모 | 문서 작성 시점 참고값: 12 files, +983/-12 |
| mergeable | 문서 작성 시점 참고값: `MERGEABLE` |
| maintainer modify | `true` |
| merge 결과 | `cb51cdfccb1f3f3d2d33958c247766660d5a3e96`로 merge 완료 |

최종 판단은 PR head 최신 커밋 기준 GitHub Actions와 로컬 focused 검증이 통과했고,
작업지시자가 PR 본래 목적 달성을 확인했으므로 merge 가능이다. 아래 문서 보존 항목과
추가 화면 갱신 후보는 merge 후 옵션 2 문서 PR로 처리한다.

## 변경 범위 분석

- `src/renderer/composer/line_breaking.rs`
  - 한글 어절 토큰화 조건을 `korean_break_unit == 0`으로 정정했다.
  - 단일 한글 글자 줄바꿈 허용 조건을 `korean_break_unit == 1`로 정정했다.
- `src/renderer/style_resolver.rs`
  - 내부 의미 주석을 `0=어절, 1=글자`로 바로잡았다.
- `src/renderer/composer/tests.rs`
  - 토크나이저 테스트의 bit 값을 실제 계약에 맞췄다.
  - `"가나 다라"` 60px 폭에서 bit0 `[0, 3]`, bit1 `[0, 4]` 회귀를 추가했다.
- `tests/issue_2185_korean_break_unit.rs`
  - `issue1949_giant_cell_nested_tables_perf` HWP/HWPX에서 실제 셀 입력, pagination flush,
    원본 형식 저장, 재로드까지 고정한다.
- 문서
  - #2185 계획서, 단계 보고서, 최종 보고서, 오늘할일을 포함한다.

## Findings / 후속 처리 항목

### P1. `mydocs/orders/20260711.md`가 이미 merge된 #2196 기록을 덮어쓴다

PR head의 `mydocs/orders/20260711.md`는 `upstream/devel`에 존재하는 `Self PR #2196 — CanvasKit 글머리
기호 폰트 보정` 항목을 삭제하고 `M100 — v1.0.0 조판 엔진 체계화` 항목으로 대체한다. 이 상태로 merge하면
오늘할일 기록에서 #2196 처리 내역이 사라진다.

- PR head: `mydocs/orders/20260711.md` line 9
- `upstream/devel`: 같은 위치에 #2196 기록 존재

수정 방향:

- #2196 항목을 보존한다.
- 그 아래에 #2185 / #2194 항목을 추가한다.

이 finding은 코드 정확성 문제가 아니라 merge 기록 보존 문제다. 작업지시자 판단에 따라 #2194는 먼저
merge하고, #2196 기록 복구와 #2194 검토 기록 추가는 옵션 2 문서 PR에서 처리한다.

### P2. HWPX `breakNonLatinWord` 정오표/스펙 주석 반영이 PR 범위에서 빠져 있다

#2185 코멘트에서는 OWPML `breakNonLatinWord` 열거값 설명과 한컴 실동작이 반대임을
`mydocs/tech/hwp_spec_errata.md`의 별도 HWPX 정오 항목과
`mydocs/tech/한글문서파일형식_5.0_revision1.3.md`의 표 44 보완 주석으로 남기겠다고 정리했다.
이번 PR은 보고서에는 해당 판단을 설명하지만, 장기 스펙 문서는 갱신하지 않는다.

수정 방향:

- `mydocs/tech/hwp_spec_errata.md`에 HWPX `breakNonLatinWord` 항목을 추가한다.
- `mydocs/tech/한글문서파일형식_5.0_revision1.3.md` 표 44 근처에 같은 주의사항을 남긴다.
- 핵심 결론은 HWP5 bit7 `0=어절, 1=글자`는 올바르고, OWPML 설명과 한컴 실동작이 반대라는 점이다.

이 항목은 blocker는 아니며, 이번 PR이 semantic 계약을 고치는 PR이므로 옵션 2 문서 PR에서 함께 보강한다.

### P3. 추가 입력 후 repaint/표시 지연 문제는 후속 보완 후보로 남긴다

작업지시자가 #2194 검증 중 별도 화면 갱신 문제를 확인했다. PR 본래 목적인 한글 줄 나눔 단위 의미
반전 정정은 달성했으므로 merge blocker로 보지 않는다. 다만 다음 증상은 후속 보완 후보로 보존한다.

- `issue1949_giant_cell_nested_tables_perf.hwp` 1쪽에서 `1.1.1` 문단 끝까지 `1`을 입력하면 캐럿은
  마지막 줄 끝에 위치한다.
- 이어서 `1`을 계속 입력하면 캐럿이 사라지고 추가 입력 내용도 즉시 표시되지 않는다.
- 시간이 지나면 기존 입력분까지 일부 사라진 것처럼 보인다.
- `1.1.1` 마지막 문단에서 Enter를 치면 입력 내용이 다시 repaint되어 나타난다.

증적 asset:

- `mydocs/pr/assets/pr_2194/pr_2194_followup_01_line_end_cursor.png`
- `mydocs/pr/assets/pr_2194/pr_2194_followup_02_extra_input_not_painted.png`
- `mydocs/pr/assets/pr_2194/pr_2194_followup_03_delayed_text_disappears.png`
- `mydocs/pr/assets/pr_2194/pr_2194_followup_04_enter_repaints.png`

## 검증 결과

로컬 검증은 PR head `ccbfe5e93bd9a0094d961de116f95d9040a78e51` 기준으로 수행했다.
PR review 규칙에 따라 cargo 검증 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다.

- `git diff --check upstream/devel...HEAD`
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2185_korean_break_unit -- --nocapture`
  - HWP: load 약 1.429s, edit 약 0.546ms, flush 약 1.368s, save 약 9.10ms, reload 약 1.181s
  - HWPX: load 약 1.227s, edit 약 0.095ms, flush 약 1.255s, save 약 17.08ms, reload 약 1.230s
  - 결과: 1 passed
- `CARGO_INCREMENTAL=0 cargo test --profile release-test korean_break_unit -- --nocapture`
  - `test_reflow_korean_break_unit_contract` 통과
- `CARGO_INCREMENTAL=0 cargo test --profile release-test test_tokenize_korean -- --nocapture`
  - `test_tokenize_korean_eojeol` 통과
  - `test_tokenize_korean_character_unit` 통과
- `CARGO_INCREMENTAL=0 cargo build`
- GitHub Actions 최신 상태 확인
  - Build & Test, CodeQL, Render Diff, Native Skia, Canvas visual diff 모두 pass

## Visual Sweep

렌더 영향 PR로 판정해 `issue1949_giant_cell_nested_tables_perf.hwp`와 기존 HWP 2020 기준 PDF를 사용해
대표 1쪽 visual sweep을 수행했다.

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr2194-issue1949 \
  --hwp samples/issue1949_giant_cell_nested_tables_perf.hwp \
  --pdf pdf/issue1949_giant_cell_nested_tables_perf-2020.pdf \
  --page 1 \
  --rhwp-bin target/debug/rhwp \
  --out output/visual_sweep_pr2194
```

결과:

- SVG pages: 115
- PDF pages: 115
- selected pages: 1
- flagged: 0/1
- pixel match: 89.79961%
- visual accuracy proxy: 11.8903%
- 자동 drift 후보: 없음

Asset:

- `mydocs/pr/assets/pr_2194/pr_2194_issue1949_p1_review.png`
- `mydocs/pr/assets/pr_2194/pr_2194_issue1949_p1_overlay.png`
- `mydocs/pr/assets/pr_2194/pr_2194_visual_summary.json`

판정:

- 대표 1쪽에서 자동 시각 drift 후보는 없다.
- 낮은 ink match는 HWP 2020 PDF와 native SVG의 기존 폰트/래스터 차이 성격으로 보이며,
  이번 PR의 핵심 주장인 편집 후 `LINE_SEG.text_start`, 다음 문단 `vpos`, 전체 115쪽 보존은
  전용 통합 테스트가 직접 검증한다.

## PR 주장 검증

- `korean_break_unit` 소비 의미 반전 주장은 코드와 이슈 분석에 부합한다.
- PR의 최소 수정은 parser/serializer 저장 bit 매핑을 건드리지 않고 renderer consumer만 바로잡는다.
- 실제 HWP/HWPX 입력 후 `[0, 44, 84, 122]`, 다음 문단 `vpos=17160`, 전체 115쪽 보존 주장은
  로컬 통합 테스트로 재확인했다.
- 성능 지연은 #2193으로 분리하는 판단이 타당하다.
- PR 본문은 `Refs #2185`만 사용하므로 merge 후 #2185 자동 close는 되지 않는다. merge 시 PR 본문을
  `Closes #2185`로 고치거나, merge 후 수동 close/comment가 필요하다. #2193은 open 유지가 맞다.

## 최종 권고

코드 수정 방향과 회귀 테스트는 타당하고, PR 본래 목적은 달성했다. `mydocs/orders/20260711.md`의 #2196
기록 복구, HWPX `breakNonLatinWord` 정오표, PR review 문서/asset 보존, 추가 repaint 후보 기록은
옵션 2 문서 PR로 처리한다.

#2194는 최신 GitHub Actions 통과 상태에서 `cb51cdfccb1f3f3d2d33958c247766660d5a3e96`로 merge 완료했다.
#2185는 옵션 2 문서 PR merge 후 close 대상이고, #2193은 성능 후속 이슈로 open 유지한다.
