# #1937 Stage 3 — 구현 + 검증

- 브랜치: `local/task1937` / 커밋: 본 단계 소스 + 보고서

## 1. 수정 내용 (1줄 실질 변경)

`src/renderer/typeset.rs` `typeset_block_table` 의 행 분할 루프에서, **연속(continuation)
페이지의 `page_avail` 을 시작 페이지 기준 `table_available` 대신 신선 페이지 기준
`base_available` 로 변경**한다(레퍼런스 Paginator `engine.rs:2502-2503` 과 정합).

```rust
let page_avail = if is_continuation {
    // 시작 페이지의 table_available(=표 전체 각주 total_footnote 를 available 에서 차감)을
    // 연속 페이지에 재사용하면, 각주 많은 큰 RowBreak 표가 시작 페이지 좁은 잔여를 매
    // 페이지 물려받아 페이지당 ~1행으로 과분할된다. 표 각주는 첫 fragment fit 판정에서만
    // 보수적으로 예약하고 연속 페이지는 신선 본문 가용을 쓴다. zone/tolerance 유지.
    (base_available - st.current_zone_y_offset - st.layout.pagination_tolerance_px).max(0.0)
} else if ... { ... };
```

근본 원인(Stage 2): 표 pi=306 은 22개 셀 각주(721px, projected 820px)를 가지는데,
`available = base − total_footnote = 895.8 − 820 = 75.8px` 가 `table_available` 로 굳어
**모든 연속 페이지에 재사용**되어 페이지당 ~1행 → 122행 표가 188쪽으로 폭주.

## 2. 검증 — 효과

| 문서 | 수정 전 | 수정 후 | 한글 | 판정 |
|---|---:|---:|---:|---|
| 소상공인 중간보고서(canonical) | 231 | **52** | 50 | +181 → **+2** |
| └ pi=306 표 단독 | 188쪽 | **9쪽** | ~9 | 정합 |
| 문체부 GCC 중간보고서 | 404 | **322** | 320 | +84 → **+2** |

- canonical pi-page 오라클 재측정: PAGE_DELTA +181 → **+2**(잔여는 pi73~ ±1 시프트,
  일부 #1920 캐럿 계열 — 본 이슈와 무관한 별개 축).

## 3. 검증 — 무회귀 (과도 수정 아님)

- **`cargo test --lib`: 2126 passed, 0 failed** (pagination 유닛 테스트 포함).
- 통합 테스트: issue_1073_nested_table_split 등 표 분할/각주 계열 통과.
- #1658 게이트: byeolpyo1=4쪽, byeolpyo4=26쪽 (기대값 유지).
- 정상 분할 표 pi=99: 8쪽 무변화.
- **별개 기전 문서 무변화**(과도 수정 아님 확인): +88 공급망(212→212)·+27 거제시(387→387)
  는 각주-예약 버그가 아니라 별도 원인 → 본 수정이 건드리지 않음(별도 추적 대상).

## 4. 잔여/후속

- +88 공급망·+27 거제시 등 **비-각주 과대 페이지**는 별개 기전(행높이/기타). 본 이슈
  범위(각주 예약 continuation 재사용) 밖 — 필요 시 별도 이슈로 분리.
- 일반형(연속 페이지에 실제 각주가 걸치는 표의 per-page 각주 예약)은 레퍼런스
  Paginator 도 continuation 에서 각주 미예약이라, 본 수정으로 두 엔진 동작을 통일.

## 5. 게이트 결론

로컬 lib 2126 통과 + 표 분할/각주 통합 테스트 통과 + #1658 양 게이트 유지 + canonical/GCC
대폭 개선 + 과도 수정 없음. **풀 스위트/스냅샷은 CI 에서 확인**(작업지시자 지침).
