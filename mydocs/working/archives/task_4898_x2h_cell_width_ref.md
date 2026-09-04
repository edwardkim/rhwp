# #4898 ② x2h 셀 `width_ref` micro-grid 휴리스틱 제거

## 목표

HWPX → HWP5 저장(x2h)에서 한글이 원본보다 쪽수를 1쪽 더 세는 잔존군을 줄인다. 한글 2022
오라클 10k 전수(s36) 기준 x2h 쪽수 결함은 71경로였고, 그중 63경로가 "원본 1쪽 → 저장본 2쪽"인
서식 문서였다.

## 원인

`hwpx_to_hwp.rs` 의 `table_requires_cell_width_ref_contract` 가 `table.col_count >= 30` 인 표를
micro-grid 로 보고, **셀 자신이 안 여백을 쓰지 않는데도**(`apply_inner_margin == false`)
LIST_HEADER `width_ref` bit0(=aim, 자기 여백 사용)을 세우고 표 기본 여백을 셀 padding 으로
물질화했다. 한글은 그 값을 액면대로 읽어 셀 안 여백을 크게 잡고, 행 높이 → 표 높이 → 쪽수 순으로
번져 1쪽에 맞던 서식이 2쪽이 됐다.

이 휴리스틱은 #1809(admrul micro-grid 계열에서 한컴이 셀 내부 줄나눔 폭을 너무 좁게 잡던 문제)
때 들어왔으나, 그 계약을 지키는 회귀 테스트는 저장소에 없다.

## 변경

- `table_requires_cell_width_ref_contract` 를 제거했다.
- `materialize_cell_list_header_contract` 에서 `use_width_ref` 인자와 표 여백 물질화 분기를
  제거했다. `width_ref` bit0 는 이제 셀 자신의 `apply_inner_margin` 만 따른다.
- `raw_list_extra`(셀 폭 4바이트 + 13바이트 슬롯) 물질화는 종전대로 모든 셀에 유지한다.
- 계약 테스트 `tests/cases/issue_4898_x2h_cell_width_ref.rs` 3건을 추가했다.

## 검증 실측 — 한글 2022 오라클

baseline `rhwp_s36.exe`(devel `9d352d56d`) 와, 같은 커밋에서 이 휴리스틱만 끈 프로브 바이너리로
코퍼스 10,000건(HWPX 입력 3,418건)을 각각 변환해 한글 2022 로 쪽수를 재측정했다.

**영향 범위 확정**: 두 산출을 SHA-256 으로 비교해 휴리스틱이 실제로 산출을 바꾸는 문서
**1,239건**을 뽑았다(변환은 결정적임을 재변환 12/12 동일로 확인). 그 전수를 한글로 측정했다.

| 구분 | 건수 |
|---|---|
| 고침 — 기존 쪽수 결함이 원본 쪽수로 복귀 | **58** |
| 깨짐 — 정상이던 문서가 틀어짐 | **0** |
| 둘 다 정상 | 1,180 |
| 둘 다 틀림(별개 원인, 08818) | 1 |
| 텍스트 길이 달라진 문서 / 컨트롤 집계 달라진 문서 | **0 / 0** |

x2h 쪽수 잔존군 71경로 중 **58경로(82%)가 이 한 원인**이었다. 남은 13경로는 별개 원인군이다
(01052 6→7, 02319 5→6, 08818 81→86 등).

## 검증 기준

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --test regression_suite_010 -- issue_4898` (새 계약 테스트 3건)
- `cargo test --test regression_suite_001 --test regression_suite_007`
  — 셀 여백/어댑터 계약을 담은 기존 스위트(`hwpx_to_hwp_adapter`,
  `issue_1785_cell_padding_rule_consistency`)
