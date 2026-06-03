# Task #1105 Stage 3 진행 기록 — K-water 2024 후속 정합

- 이슈: [edwardkim/rhwp#1105](https://github.com/edwardkim/rhwp/issues/1105)
- 기준 브랜치: `tmp/test-1085-1103-1107-final`
- 승인: 2026-05-24 작업지시자 승인

## 1. 추가 범위

세 PR(#1085, #1103, #1107)을 순차 병합한 임시 브랜치에서 `samples/k-water-rfp-2024.hwp`를 확인한 결과
두 가지 정합 문제가 남았다.

| 항목 | 한컴오피스/PDF 기준 | rhwp 현재 |
|------|--------------------|-----------|
| 전체 페이지 수 | 27쪽 (`pdf/k-water-rfp-2024.pdf`) | 28쪽 |
| 표지 하단선 | 표시 안 됨 | 꼬리말 표 선 표시 |

## 2. 진단 근거

페이지 수 문제:

```text
pdfinfo pdf/k-water-rfp-2024.pdf → Pages: 27
rhwp dump-pages samples/k-water-rfp-2024.hwp → 28 pages
```

28쪽 발생의 첫 차이는 `s1/pi52` 4x4 RowBreak 표 분할이다.

```text
k-water-rfp.hwp:
page 5  PartialTable pi=52 rows=0..4 start_cut=[] end_cut=[...]
page 6  PartialTable pi=52 rows=2..4 start_cut=[...] end_cut=[]

k-water-rfp-2024.hwp (수정 전):
page 5  PartialTable pi=52 rows=0..3
page 6  PartialTable pi=52 rows=3..4
```

두 파일의 표 구조와 셀 높이는 같지만, 셀 내부 페이지 브레이크 신호가 다른 위치에 있다. 기존 샘플은
`cell[14]`의 `LINE_SEG.vpos` 리셋이 문단 첫 줄에서 발생하고, 2024 샘플은 같은 문단 안의 두 번째
줄에서 `vpos`가 31840 → 1652로 감소한다. 기존 `cell_units()`는 문단 첫 줄 리셋만 하드 브레이크로
보았기 때문에 2024 샘플의 RowBreak rowspan 블록 컷을 놓쳤다.

표지 하단선 문제:

```text
page 1 SVG:
<line x1="80" y1="1034.8666666666668" x2="713.96" y2="1034.8666666666668" stroke="#787878" stroke-width="1.5"/>
```

이 선은 본문 밑줄이 아니라 첫쪽 꼬리말 표 선이다. HWP5 구역 정의는 `flags=0x00000003`으로
첫쪽 머리말/꼬리말 감춤을 지시하지만, 기존 파서는 머리말/꼬리말 감춤을 `0x0100/0x0200`으로
해석하고 있어 적용하지 못했다.

## 3. 작업 방침

1. #1105 회귀 테스트에 `k-water-rfp-2024.hwp=27`을 추가한다.
2. 1쪽 SVG에 꼬리말 표 하단선이 남지 않는지 회귀 테스트를 추가한다.
3. HWP5 `SectionDef.flags` 감춤 비트를 `hwpspec-2024.pdf` 표 119 기준으로 정정한다.
4. 표 분할 정합은 `s1/pi52`의 RowBreak rowspan 블록에서 같은 문단 내부 `LINE_SEG.vpos` 리셋을
   하드 브레이크로 인식하는 일반 규칙으로 해결한다.
5. 파일명 기반 특수 처리는 금지한다.

## 4. 구현 결과

- `tests/issue_1105.rs`
  - `k-water-rfp-2024.hwp` 전체 27쪽 회귀 추가
  - `pi52`가 page 5에서 `rows=0..4 + end_cut`을 갖는지 추가 검증
  - 첫쪽 꼬리말 표 하단선 미표시 검증 추가
- `src/parser/body_text.rs`, `src/parser/hwpx/section.rs`, `src/document_core/queries/rendering.rs`
  - HWP5/HWPX 첫쪽 머리말/꼬리말 감춤 비트를 bit 0/1 기준으로 정정
  - 같은 구역 머리말/꼬리말 보정 이후에도 첫쪽 감춤을 재적용
- `src/renderer/layout/table_layout.rs`
  - `cell_units()`가 같은 문단 안의 후속 줄 `LINE_SEG.vpos` 감소도 `hard_break_before`로 표시하도록 확장

## 5. 검증

```text
cargo test --test issue_1105 -- --nocapture
cargo test --test issue_713 -- --nocapture
cargo test --test issue_1086 -- --nocapture
cargo test --test issue_1035_alignment -- --nocapture
cargo test --test issue_554 -- --nocapture
cargo test --test issue_nested_table_border -- --nocapture
cargo fmt --all -- --check
git diff --check
```

모두 통과했다.
