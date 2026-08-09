---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 33 — #3637 page-owner fidelity 보정

## Stage 32에서 넘긴 사실

- 입력은 `samples/issue3637/regulatory_impact_nested_table_escape.hwpx`이며, 독립 기준은
  `pdf/issue3637/regulatory_impact_nested_table_escape-current-2020.pdf`의 새 HWP 2020
  `PrintToPDFEx` 출력(31쪽)이다.
- Stage 32의 renderer는 focused Rust 회귀에서 31쪽이고 `overflow_cell_baseline`도 19로
  통과한다. 그러나 이 결과는 PDF fidelity 완료를 뜻하지 않는다.
- 동일한 31쪽끼리 p25–p31 raster 대조를 하면 pixel difference가 16.71–21.15%다. PDF p27은
  `사업체노동력조사…`에서 시작하지만 rhwp p27은 그 문장의 중간부터 시작하며, 다음 표 행도
  앞당겨 배치한다. 이 문제는 페이지 밖 clipping이 아니라 **가시 source owner가 이전 쪽으로
  이동한 것**이다.

## 현재 gate가 놓치는 이유

`tests/overflow_cell_baseline.rs`는 줄의 윗변이 물리 페이지 하단 밖에 있을 때만 증가한다.
row cut이 커서 다음 페이지 소유의 내용까지 현재 페이지 clip 안에 넣으면 이 값은 감소하거나
동일할 수 있다. 따라서 이 gate는 유효한 anti-clipping ratchet이지만 page-owner fidelity gate가
될 수 없다.

`fidelity_compare.py`의 SVG text ledger는 ancestor clip 밖의 text도 기록할 수 있으므로 단독
정답지로 쓰지 않는다. 새 HWP 2020 PDF raster, PDF text 순서, 그리고 rhwp SVG raster를 함께
사용해야 한다.

## 원인 축과 기각한 방법

`TypesetEngine::scan_block_table_split_rows`는 `advance_row_cut`의 content-only 소비량으로
cut을 정한 뒤 `row_cut_content_height`의 mixed nested-tail 및 padding을 물리 높이에 더한다.
HWPX row-break의 허용치(64px) 안에 들어가는 p26 조각은 content-unit boundary보다 더 많은
visible source를 현재 쪽에 배치할 수 있다.

실험적으로 p26의 cut을 `[1, 57]`에서 `[1, 56]`으로 엄격하게 줄였더니 물리 하단 초과는
사라졌지만 전체 출력이 31쪽에서 30쪽으로 바뀌었다. 즉 전역 tolerance를 단순 축소하거나
현재 쪽에서만 줄을 버리는 수정은 문서 전체 owner mapping을 깨뜨리므로 채택하지 않는다.

## 수행 결과

raw SVG text는 ancestor clip 밖의 이전 표 조각까지 세므로 `text-owner-shift-candidates.tsv`가
비어 있었다. `fidelity_compare.py`에 다음을 추가했다.

1. root viewport와 rhwp가 출력한 axis-aligned `body-clip-*`/`cell-clip-*` 교집합을 따라 실제
   보이는 baseline band만 추출하는 `svg_visible_text`.
2. PDF text가 보존된 상태에서 visible SVG text가 48자 이상 과잉인 경우를 남기는
   `visible-text-excess-candidates.tsv`.
3. clip 밖 text를 제외하고, 부분적으로 보이는 줄은 보수적으로 유지하는 Python unit 회귀.

새 HWP 2020 direct text-only run은 p26을 다음처럼 자동 후보화했다.

```text
page  reference_only  visible_svg_only  clip_excluded_chars
26    0               66                415
```

따라서 이 Stage는 “31쪽인데도 page-owner가 틀린” 상태를 자동 탐지하는 보정을 완료했다.
renderer 자체의 p25–p31 source owner는 아직 고쳐지지 않았으며, Stage 34에서 mixed nested-tail
소비량과 continuation 원본 unit 재개를 보정한다. 전역 tolerance 축소는 출력이 30쪽으로 변한
반증 때문에 다시 사용하지 않는다.

## 증적

- [visible text excess candidate](../pr/assets/task_m100_3820_stage33_issue3637_page_owner_fidelity/visible_text_excess_candidates.tsv)
- [run state](../pr/assets/task_m100_3820_stage33_issue3637_page_owner_fidelity/run_state.tsv)
- [page count ledger](../pr/assets/task_m100_3820_stage33_issue3637_page_owner_fidelity/page_count_ledger.tsv)
