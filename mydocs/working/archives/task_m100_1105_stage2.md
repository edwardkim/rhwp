# Task #1105 Stage 2 완료 보고서 — 구현 및 검증

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 구현 커밋: `d4587b27 fix: preserve hwp3 conversion page break`

## 1. 변경 요약

### 1.1 bridge reset 복원

`src/renderer/typeset.rs`와 `src/renderer/pagination/engine.rs`에 다음 조건을 추가했다.

```text
prev real LINE_SEG 이후 현재 paragraph 전까지
LINE_SEG 없는 visible text 문단이 2개 이상 있고,
현재 paragraph는 real LINE_SEG를 가지며,
curr_first_vpos <= 1500,
prev_end_vpos > body * 0.75
```

이로써 `pi=437..439` 누락 bridge 뒤의 `pi=440` page-reset을 복원한다.

### 1.2 synthetic line height compact 보정

`src/renderer/mod.rs`에 `corrected_line_height_for_variant_synthetic()` helper를 추가했다.

HWP3-origin HWP5 변환본에서 `PARA_LINE_SEG`가 없는 visible text 문단은 raw fallback line height가 폰트보다 작을 때 `max_fs`를 줄높이로 사용한다. 일반 문서와 real LINE_SEG 문단은 기존 `corrected_line_height()`를 그대로 사용한다.

적용 경로:

- `src/renderer/typeset.rs`
- `src/renderer/height_measurer.rs`
- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/layout/table_layout.rs`

### 1.3 variant flag 전달

`DocumentCore::paginate()`와 `DocumentCore::build_page_tree()`에서 `document.is_hwp3_variant`를 측정/레이아웃 엔진으로 전달했다.

`LayoutEngine`에는 `set_hwp3_variant()`를 추가했다.

## 2. 회귀 테스트

새 파일:

```text
tests/issue_1105.rs
```

검증:

```text
samples/hwp3-sample16-hwp5.hwp = 64 pages
page 21 contains pi=439
page 21 does not contain pi=440
page 22 contains pi=440, pi=441, pi=449
```

## 3. 덤프 확인

수정 후:

```text
문서 로드: samples/hwp3-sample16-hwp5.hwp (64페이지)

page 21:
  FullParagraph pi=426 ... pi=439
  pi=440 없음

page 22:
  FullParagraph pi=440 "4. 서버통합 및 원격지 재해복구센터 시스템 구성요건"
  Table pi=441
  FullParagraph pi=442..449
  PartialParagraph pi=450 lines=0..1
```

## 4. 자동 검증

통과:

```text
cargo fmt --all -- --check
cargo test --test issue_1105 --test issue_1086 --test issue_1035_alignment --test issue_554
cargo test --test issue_nested_table_border
git diff --check
```

세부 결과:

```text
issue_1105: 1 passed
issue_1086: 4 passed
issue_1035_alignment: 1 passed
issue_554: 12 passed
issue_nested_table_border: 2 passed
```

## 5. 남은 주의점

이 변경은 #1103의 #1086 page-count guard 위에 적층되어 있다. #1103이 merge되기 전에는 PR 본문에 stacked 상태를 명시해야 한다.

또한 `CLAUDE.md` 절차상 새 PR 생성은 작업지시자 승인 후 진행한다.
