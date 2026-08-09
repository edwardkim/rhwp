---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 32 — #3637 새 HWP 2020 PDF 기준 gate 검증

## 기준과 실행

- 입력: `samples/issue3637/regulatory_impact_nested_table_escape.hwpx`
- 독립 기준: `pdf/issue3637/regulatory_impact_nested_table_escape-current-2020.pdf`
  - Hancom HWP 2020 `PrintToPDFEx`, 31 pages
  - SHA-256: `1b55d6d6545b7ac3e576fae738c74dba771f55ae324f7f2e0c9762fd747af7bd`
- rhwp 실행 파일: `target/task-3820-3821-fidelity/release-test/rhwp`

`fidelity_compare.py` direct pair를 p25–p31에 실행했다. 새 PDF와 rhwp SVG/render tree는
모두 31 pages이므로, 페이지 수만으로는 이 문서의 현재 차이를 판정할 수 없다.

## gate 판정

`overflow_cell_baseline`은 셀 줄의 **윗변이 물리 페이지 하단 밖**에 있는 경우만 센다.
즉, 다음 페이지로 가야 할 표 행·문단이 현재 페이지의 clip 안에 완전히 들어가 버린
경우의 source owner 이동은 판정 대상이 아니다. 이는 gate 구현의 결함이 아니라, 정의된
보조 신호의 범위다. 따라서 이 gate의 통과를 Hancom PDF fidelity 완료로 사용해서는 안 된다.

이번 실행에서 대상 fixture 값은 기존 601에서 19로 감소했다. 이 값은 다음 실행에서 다시
증가하지 않도록 baseline을 19로 낮춘다. 검증 명령은 exit 0으로 완료됐다.

```text
CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --test overflow_cell_baseline -- --quiet

1 passed; 0 failed; finished in 77.08s
issue3637/regulatory_impact_nested_table_escape.hwpx    19
```

## PDF 직접 대조 결과와 다음 원인 축

페이지 수는 같지만 p25–p31 pixel difference가 16.71–21.15%다. 특히 PDF p27은
`사업체노동력조사…`에서 시작하고 다음 직접편익 표는 아직 나오지 않지만, rhwp p27은 그
문장의 중간에서 시작해 다음 표의 2025–2027 행을 이미 배치한다. 이어 p28은 PDF의 2025
행에서 시작하지만 rhwp는 2028 행에서 시작한다. 따라서 남은 문제는 물리 하단 탈출이 아니라
1×1 RowBreak host 안 mixed nested table의 unit viewport가 HWP 2020의 페이지 capacity보다
과대하게 계산되는 것이다.

SVG text 원장의 `svg_only` 수치는 ancestor clip 밖의 비가시 SVG text도 포함하므로 단독 확정
근거가 아니다. 이번에는 PDF raster와 SVG raster를 나란히 보관해 실제 보이는 row owner
이동을 확인했다.

## 증적

- [p26 PDF/rhwp pair](../pr/assets/task_m100_3820_stage32_issue3637_hwp2020_gate_validation/review_p026_pair.png)
- [p27 PDF/rhwp pair](../pr/assets/task_m100_3820_stage32_issue3637_hwp2020_gate_validation/review_p027_pair.png)
- [pixel report](../pr/assets/task_m100_3820_stage32_issue3637_hwp2020_gate_validation/p025_p031_pixel_report.tsv)
- [page-count ledger](../pr/assets/task_m100_3820_stage32_issue3637_hwp2020_gate_validation/page_count_ledger.tsv)
- [table fragment candidates](../pr/assets/task_m100_3820_stage32_issue3637_hwp2020_gate_validation/table_fragment_candidates.tsv)
- [provenance](../pr/assets/task_m100_3820_stage32_issue3637_hwp2020_gate_validation/provenance.tsv)

## 단계 결론

이 단계는 lower-bound clip gate를 19로 고정하고, 그 gate가 PDF 기준 page-owner 이동을
판정하지 못한다는 한계를 새 HWP 2020 증적으로 확정했다. p25–p31 직접 차이는 **미해결**이며,
다음 단계는 baseline을 다시 완화하지 않고 fragment의 source unit과 physical viewport를
분리해 다음 페이지에 속한 행이 앞 페이지에 보이지 않도록 보정한다. 이후 같은 fresh PDF
direct pair로 p25–p31을 다시 확인한다.
