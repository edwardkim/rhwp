---
kind: implementation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 83 — 페이지 경계 owner와 #4090 HWPX tail PDF fidelity

## 이전 단계 인계

Stage 82는 저장 `LINE_SEG`가 없는 true-empty 문단의 높이를 pagination·measurer·SVG layout에서
동일하게 계산하도록 정렬했다. 또한 `fidelity_compare.py`에 `page-boundary-fidelity-candidates.tsv`를
추가해 인접 PDF↔SVG text owner 이동과 동일 source table fragment를 한 원장으로 결합했다.

현재 이 원장은 다음 후보를 자동 수집한다.

| 대상 | 자동 후보 | PDF로 확인할 내용 |
| --- | --- | --- |
| `76076_regulatory_analysis.hwp` p70→71 | `text_owner_shift`, 9자, rhwp earlier | `② 구내운반차 …` label/본문의 실제 page owner |
| 같은 문서 p81→82 | `table_fragment_text_owner_drift`, 39자, rhwp later, `pi=842/ci=0` | p81 첫 근거설명 줄이 p82에서 반복되지 않고 이어지는지 |

후보는 PDF visual review 전 결함 확정이 아니다. 특히 페이지 수·raw text count·이전 rhwp 산출물은
한컴 2024/2020 PDF의 개별 page owner를 대체하지 않는다.

## #4090 후속 범위

Issue #3820 comment `5207347524`가 이관한 대상은 다음이다.

- 입력: `samples/issue4090/156492236_규제샌드박스_min.hwpx`
- 기준 PDF: `pdf/issue4090/156492236_규제샌드박스_min-hancom2020-production-verify.pdf`
- PDF provenance: HWP 2020 MCP `PrintToPDFEx`, `PrintMethod=0`
- 확인된 물리 줄 경계: `pi=59` p5→6, `pi=74` p7→8, `pi=183` p15→16

Stage 83은 기준 PDF와 rhwp SVG/render tree를 17쪽 전체에 직접 대조한다. tail 줄 수나 페이지 수가
맞더라도 글꼴·표·개체·머리말을 포함한 raster와 page owner가 다르면 별도 원인으로 기록한다.

## 실행 규약

- [`local_validation.md`](../manual/pr_review/local_validation.md)의 PR 검증 target인
  `target/pr-review`를 고정해 사용하고, 모든 Cargo 명령에 `CARGO_INCREMENTAL=0`을 지정한다.
  Cargo build·test·clippy·`wasm-pack`은 이 target을 공유하므로 **동시에 실행하지 않는다**. nextest의
  `--test-threads`만 해당 host CPU 수 이내에서 사용한다.
- [`webhwpctrl_compat_development.md`](../manual/webhwpctrl_compat_development.md)의 경계를 따른다.
  이 작업의 PDF fidelity 정답은 Hancom 2020/2024 PDF이며, macOS의 rhwp WASM 검증은 자체 회귀
  증거다. 새 Windows COM Oracle fixture를 수집하거나 갱신해야 할 때만 Windows 한글 2022에서 문서별
  직렬로 실행하고, 일반 Cargo 병렬화 근거로 사용하지 않는다.
- PDF 직접 대조와 text-only ledger처럼 Cargo 산출물을 바꾸지 않는 읽기 전용 분석만 Cargo 실행 사이에
  수행한다.
- 구현 원인은 parent `PartialTable`과 nested cell의 RowCut/paint continuation 관계로 한정해 찾는다.
  페이지 한도 완화, baseline 재기록, reference PDF 교체로 후보를 없애지 않는다.

## 순서

1. p70→71, p81→82의 기준 PDF·SVG·render tree를 함께 대조해 두 후보의 source row/cell owner를 확정한다.
   p81→82는 Hancom PDF에서 p81가 `일시적/반복적` 행과 `○ 구내운반차 … 사고` 첫 줄을 소유하고,
   p82는 `를 예방함으로써 …`로 재개함을 확인했다. 현재 rhwp는 p81에서 row 0..2까지만 그리고,
   p82에서 row 3 및 nested `근거설명` 전체를 다시 시작하므로 실제 결함이다.
2. p81→82의 nested continuation이 parent fragment cut으로 전파되지 않는 경로를 수정하고 회귀를 추가한다.
3. #4090 HWPX 17쪽을 `fidelity_compare --text-only --export-all-svg --layout-ledger`로 전수 triage한 뒤
   후보 페이지를 raster/visual sweep으로 확정한다.
4. focused Rust·Python 검증을 통과한 단위마다 커밋하고 다음 stage로 넘긴다. 전체 release-test는
   focused 결과와 visual evidence를 먼저 기록한 뒤 별도 지시에 따라 실행한다.
