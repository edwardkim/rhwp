# Stage 2 보고 — Task M100-1271

## 범위

- 이슈: [#1271](https://github.com/edwardkim/rhwp/issues/1271)
- 단계: TypesetEngine paper-anchored `BehindText`/`InFrontOfText` 표 out-of-flow 보정
- 수정 파일: `src/renderer/typeset.rs`

## 변경

`src/renderer/typeset.rs` 의 글뒤로/글앞으로 표 단축 분기에서 `oversized_multirow` 예외를 좁혔다.

기존 동작:

```text
row_count > 1 && measured_height > base_available_height
```

이면 `BehindText`/`InFrontOfText` 표라도 데코레이션 단축 분기에서 제외하고 `typeset_block_table` 로 넘겼다.

변경 후:

```text
row_count > 1
&& measured_height > base_available_height
&& !(treat_as_char=false && vert_rel_to=Paper && horz_rel_to=Paper)
```

즉, 종이 기준 절대좌표의 배경/전경 표는 크기가 본문 가용 높이를 넘더라도 본문 흐름을 차지하지 않는 `PageItem::Shape` 로 유지한다.

## 대상 샘플 확인

명령:

```text
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 1
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 2
cargo run --quiet --bin rhwp -- dump-pages "samples/hwpx/[2027] 온새미로 1 본교재.hwpx" -p 3
```

결과:

```text
문서 로드: samples/hwpx/[2027] 온새미로 1 본교재.hwpx (46페이지)

page 2:
  FullParagraph pi=4 "MEMO"

page 3:
  Shape pi=5 ci=0

page 4:
  section=1, page_num=4
  FullParagraph pi=0 "강의 01."
```

Stage 1 에서 재현된 page 2 의 `PartialTable pi=3 ci=0` 은 사라졌다.

## 검증

```text
cargo fmt --check
cargo test --test issue_1271_hwpx_behind_text_table -- --nocapture
cargo test --test issue_703 -- --nocapture
cargo test --test issue_775 -- --nocapture
cargo test --test issue_1156_rowbreak_fragment_fit -- --nocapture
cargo test --lib test_typeset_703_behind_text_table_no_flow_advance -- --nocapture
cargo test --test exam_eng_multicolumn -- --nocapture
```

결과:

- `cargo fmt --check`: 통과
- `issue_1271_hwpx_behind_text_table`: 1 passed
- `issue_703`: 3 passed
- `issue_775`: 1 passed
- `issue_1156_rowbreak_fragment_fit`: 3 passed
- `test_typeset_703_behind_text_table_no_flow_advance`: 1 passed
- `exam_eng_multicolumn`: 1 passed

## 판단

- #1271 의 직접 원인인 paper-anchored 글뒤로 표의 `PartialTable` 분할은 해결됐다.
- #703 단일 컬럼 데코레이션 표, #775 다단 표 위치, RowBreak 분할 표 회귀는 집중 테스트에서 깨지지 않았다.
- Stage 3 에서는 구현계획서대로 구역 간 쪽번호 carry 이후 최종 `page_number` 기준 바탕쪽 홀짝 선택을 보정한다.
