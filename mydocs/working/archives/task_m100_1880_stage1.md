# Task #1880 Stage 1 — 자리차지 판정 원시 attr → 의미 필드 + 소스 게이트

## 구현 내용

- `src/renderer/typeset.rs` `format_table`:
  - 제거: `let table_text_wrap = (table.attr >> 21) & 0x07;` (원시 비트 판정 —
    렌더러 전체에서 유일한 `>>21` 소비처였음, 전수 확인)
  - 신규: `table_wrap_take_place = !self.is_hwpx_source.get()
    && matches!(table.common.text_wrap, TextWrap::TopAndBottom)`
  - `before` 분기(`!is_tac && table_text_wrap == 1`)를 신규 판정으로 교체.
  - 근거 주석: HWPX 파스 attr 미채움 vs HWP5 재파스 원시 attr 전체의 비대칭,
    native 비트⇔열거형 전단사(shape.rs:394) 불변 근거, #1886 origin 전달 연장.

## 경로별 효과 (설계 그대로)

| 경로 | 종전 | 신규 | 변화 |
|------|------|------|------|
| native HWP5/HWP3 | 원시 비트=1 → sb 제외 | 게이트 통과(의미 동치) → sb 제외 | 불변 |
| 순수 HWPX | attr=0 → sb 포함 | is_hwpx_source → sb 포함 | 불변 |
| convert-HWP (#1886 variant) | 원시 비트=1 → sb 제외 (비대칭) | is_hwpx_source → sb 포함 | **HWPX 정합** |

## 회귀 테스트

- fixture 2건 추가 (정부 행정규칙, issue1770 선례):
  - `samples/issue1880_takeplace_host_before.hwpx` (2780073, 39KB)
  - `samples/issue1880_takeplace_oracle_p13.hwpx` (3075729, 39KB)
- `tests/issue_1880_takeplace_host_before.rs`:
  - `takeplace_host_before_pagination_self_consistent`: (section,pi)→page 맵
    HWPX vs convert-HWP 재파스 완전 일치 + 총 쪽수 일치.
  - `oracle_3075729_heading_on_page13_both_paths`: sec1 pi=121 heading 이 양
    경로 모두 global_idx=12 (13쪽째, 한컴 2022 oracle 확정값).

## 검증

```bash
cargo test --test issue_1880_takeplace_host_before   # 2 passed
```

수정 전 상태에서는 두 테스트 모두 실패함을 실측으로 확인
(3075729 conv p12, 2780073 s0:pi9 이동 — Stage 2 A/B 하니스 참조).
