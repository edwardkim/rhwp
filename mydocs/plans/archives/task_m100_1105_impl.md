# 구현계획서 — Task #1105: sample16-hwp5 p21 page break 정합

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 수행계획서: `mydocs/plans/task_m100_1105.md`
- 구현 커밋: `d4587b27 fix: preserve hwp3 conversion page break`

## Stage 1 — 재현과 원인 정리

### 재현 명령

```bash
cargo run --quiet --bin rhwp -- dump-pages samples/hwp3-sample16-hwp5.hwp -p 20
cargo run --quiet --bin rhwp -- dump-pages samples/hwp3-sample16-hwp5.hwp -p 21
cargo run --quiet --bin rhwp -- dump samples/hwp3-sample16-hwp5.hwp --section 0 --para 440
cargo run --quiet --bin rhwp -- dump samples/hwp3-sample16-hwp5.hwp --section 0 --para 442
```

### 원인

`pi=440`은 HWP3 원본과 한컴 HWP5 변환본 모두 page-reset 성격의 `vpos=852` 신호를 갖지만, #1035에서 aux trigger를 제거한 뒤 rhwp는 cumulative height 기준으로 `pi=440`을 이전 페이지 끝에 넣었다.

과거 aux trigger를 단순 복원하면 `64 -> 65 pages` over-split 이 발생했다. 원인은 `pi=442` 이후 `PARA_LINE_SEG`가 없는 본문 문단들의 합성 줄높이가 `max_fs * line_spacing(160%)`로 계산되어 새 페이지가 과대 높이로 측정되는 것이다.

## Stage 2 — 구현

### 2.1 page-reset bridge 가드

기존 조건:

```text
prev_end > body * 0.95 && curr_first < low_threshold
```

추가 조건:

```text
prev_real_line_seg 와 현재 paragraph 사이에
LINE_SEG 없는 visible text 문단이 2개 이상 있고,
현재 paragraph 자체는 real LINE_SEG 를 가지며,
curr_first <= 1500,
prev_end > body * 0.75
```

적용 위치:

- `src/renderer/typeset.rs`
- `src/renderer/pagination/engine.rs`

이 조건은 p21의 `pi=437..439`처럼 변환 과정에서 line segment bridge가 누락된 구간을 좁게 복원한다.

### 2.2 HWP3-origin synthetic line height 보정

`PARA_LINE_SEG`가 없는 HWP3-origin HWP5 변환 문단의 합성 줄은 raw line height가 폰트보다 작다. 일반 문서에서는 ParaShape 줄간격으로 보정하지만, 변환본의 이 영역에서는 한컴이 compact하게 배치하므로 `max_fs`를 줄높이로 사용한다.

공통 helper:

```rust
corrected_line_height_for_variant_synthetic(...)
```

적용 위치:

- `src/renderer/mod.rs`
- `src/renderer/height_measurer.rs`
- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/layout/table_layout.rs`
- `src/renderer/typeset.rs`

### 2.3 variant flag 전달

높이 측정과 실제 레이아웃이 같은 판단을 하도록 기존 `document.is_hwp3_variant`를 전달한다.

적용 위치:

- `DocumentCore::paginate`
- `DocumentCore::build_page_tree`
- `LayoutEngine::set_hwp3_variant`
- `TypesetEngine` 내부 table split helper

## Stage 3 — 회귀 테스트

새 테스트:

- `tests/issue_1105.rs`

검증 내용:

```text
page_count == 64
page 21 contains pi=439
page 21 does not contain pi=440
page 22 contains pi=440, pi=441, pi=449
```

관련 회귀:

- `tests/issue_1086.rs`
- `tests/issue_1035_alignment.rs`
- `tests/issue_554.rs`
- `tests/issue_nested_table_border.rs`

## Stage 4 — PR 준비

PR #1106은 문서화 전에 생성되어 회수했다. 문서화와 검증 커밋을 추가한 뒤 새 PR 또는 재오픈은 작업지시자 승인 후 진행한다.

## Stage 5 — K-water 2024 후속 정합

### 5.1 회귀 테스트 추가

`tests/issue_1105.rs`에 K-water 2024 정답지 가드를 추가한다.

검증 내용:

```text
samples/k-water-rfp-2024.hwp page_count == 27
page 5 의 s1/pi52 RowBreak 표가 rows=0..4 + end_cut 으로 내부 컷을 가져야 함
page 1 SVG 에 꼬리말 표 하단선(#787878, y=1034.866...)이 없어야 함
```

### 5.2 첫쪽 머리말/꼬리말 감춤

`hwpspec-2024.pdf` 표 119 기준으로 HWP5 `SectionDef.flags`의 첫쪽 감춤 비트를 해석한다.

```text
bit 0 = 머리말 감춤
bit 1 = 꼬리말 감춤
bit 2 = 바탕쪽 감춤
bit 3 = 테두리 감춤
bit 4 = 배경 감춤
bit 5 = 쪽 번호 위치 감춤
```

적용 위치:

- `src/parser/body_text.rs`
- `src/document_core/queries/rendering.rs`

동일 구역 머리말/꼬리말 보정 단계가 첫쪽 감춤을 다시 되돌리지 않도록, 보정 이후에도 첫쪽 감춤을 유지한다.

### 5.3 K-water 2024 표 분할 정합

`samples/k-water-rfp-2024.hwp`의 28쪽 발생 첫 차이는 `s1/pi52` 표 분할이다. 기존
`samples/k-water-rfp.hwp`는 같은 표에서 `rows=0..4 + end_cut`을 만들어 다음 페이지에 더 많은
본문을 싣지만, 2024 샘플은 `rows=0..3` 뒤에 `rows=3..4`를 통째로 보내 전체 흐름이 한 페이지
밀린다.

원인은 셀 내부 페이지 브레이크 신호의 위치 차이다. 기존 샘플은 `cell[14]`의 `LINE_SEG.vpos`
리셋이 문단 첫 줄에 나타나지만, 2024 샘플은 같은 리셋이 같은 문단의 두 번째 줄에서 발생한다.
기존 `cell_units()`는 문단 첫 줄 리셋만 `hard_break_before`로 표시해 `row_block_has_internal_hard_break`
판정이 실패했다.

후속 구현은 다음 순서로 좁힌다.

1. `pi52`의 분할 높이와 HWP 원본 `k-water-rfp.hwp`의 같은 표 분할을 비교한다.
2. `cell_units()`에서 같은 문단 내부 후속 줄의 `LINE_SEG.vpos` 리셋도 하드 브레이크로 표시한다.
3. 파일명 특수 처리 없이 일반 표 분할 규칙으로 27쪽을 회복한다.

PR 본문 초안:

```markdown
## Summary
- restore the Hancom page break before sample16-hwp5 section 4 when HWP3-origin conversion lost explicit break metadata
- use compact synthetic line height for HWP3-origin paragraphs without PARA_LINE_SEG across pagination, measurement, and layout
- add issue #1105 regression coverage for page 21/22 paragraph boundaries

## Tests
- cargo fmt --all -- --check
- cargo test --test issue_1105 --test issue_1086 --test issue_1035_alignment --test issue_554
- cargo test --test issue_nested_table_border
- git diff --check

Refs #1105
Stacked on #1103 until #1103 lands.
```

## Stage 6 — sample16 2010/2022 변환본 page break 후속 정합

### 6.1 재현

`hwp3-sample16-hwp5-2010.hwp`와 `hwp3-sample16-hwp5-2022.hwp`에서 HWP3 원본 및
한컴오피스 정답지 기준 page break가
`pi=140` 뒤, `pi=141 "(4) 사업자 선정방식"` 앞에 있어야 한다.

현재 rhwp:

```text
page 5: pi=140, pi=141, pi=142 lines=0..1
page 6: pi=142 lines=1..3, pi=144...
```

정답 기준:

```text
page 4: pi=118 까지
page 5: pi=119 ... pi=140
page 6: pi=141, pi=142, pi=144...
```

### 6.2 원인

2010 변환본은 page break 대상 제목이 이전 페이지 하단에 단일 `LINE_SEG`로 남고, 다음 본문
문단이 페이지 상단 좌표대로 돌아간다.

```text
pi=140: vpos=55868, 57820
pi=141: vpos=59772
pi=142: vpos=2584, 4600, 6616
```

2022 변환본의 해당 구간은 `LINE_SEG.vpos`가 음수 좌표대로 이어진다.

```text
pi=140: vpos=57560, -13060
pi=141: vpos=-11108
pi=142: vpos=-9092, -7076, -5060
```

현재 cross-paragraph reset 판단은 직전 문단의 마지막 줄과 현재 문단 first `vpos`만 기준으로
삼는다. 따라서 2010에서는 하단 단일 제목 뒤의 다음 문단 상단 복귀를 놓치고, 2022에서는
`pi=140`의 마지막 음수 좌표가 직전 좌표로 들어가 `pi=141`의 page reset 판단이 실패한다.

또한 2022 변환본은 page 5 시작 경계도 흔들린다.

```text
pi=117: vpos=52472
pi=118: vpos=0
pi=119: vpos=1692, 3772, 5852
```

정답 기준은 `pi=118`이 page 4 끝, `pi=119`가 page 5 첫 문단이다. 일반 `vpos=0` reset 가드를
그대로 적용하면 `pi=118`이 page 5로 밀리고, 이를 단순 억제하면 `pi=119`까지 page 4에 붙는다.
따라서 낮은 spacing의 단일 본문 `vpos=0` 뒤에 큰 bullet heading이 이어지는 지연 reset 패턴을
별도로 다뤄야 한다.

### 6.3 구현 방향

1. `tests/issue_1105.rs`에 2010/2022 변환본 경계 회귀 테스트를 먼저 추가한다.
2. HWP3-origin 변환본 한정으로, 직전 문단 마지막 줄이 음수이고 같은 문단 앞쪽에 큰 양수
   `vpos`가 있으면 reset 판단용 직전 끝 좌표를 양수 line segment 기준으로 계산한다.
3. 현재 문단이 단일 `LINE_SEG` heading이고 first `vpos`가 음수이면서 resolved spacing 기준
   `spacing_before >= 250 HU`, visible text일 때 page break를 인정한다.
4. 현재 문단이 body 하단의 단일 `LINE_SEG` heading이고 다음 real 문단 first `vpos`가 상단
   좌표대로 작아지면 page break를 인정한다.
5. 낮은 spacing의 단일 `vpos=0` 본문 문단 바로 뒤에 상단 좌표대의 큰 bullet heading이 있으면
   본문 문단 앞 reset은 억제하고 heading 앞 지연 reset을 인정한다.
6. 동일 시맨틱을 fallback paginator에도 반영한다.

### 6.4 검증

```bash
cargo test --test issue_1105 -- --nocapture
cargo test --test issue_1035_alignment --test issue_1086 --test issue_554 --test issue_nested_table_border -- --nocapture
cargo fmt --all -- --check
git diff --check
```
