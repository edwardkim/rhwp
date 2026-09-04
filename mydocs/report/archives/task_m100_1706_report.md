# 최종 결과보고서 — Task #1706: 표 직후 빈 문단 누락(rhwp_pNone) 수정

- 마일스톤: M100 (v1.0.0) / 이슈: edwardkim/rhwp#1706 / 브랜치: `local/task1706` (← upstream/devel)
- 작성일: 2026-07-01

## 1. 문제

`treat_as_char`(자리차지) 또는 TopAndBottom 블록 표 직후의 **빈 문단**이 페이지 배치에서
누락(`rhwp_pNone`). 한컴 한글(OLE)은 이 빈 문단을 현재 페이지 하단의 빈 줄 1개로 유지하므로,
문단→페이지 매핑이 어긋남.

## 2. 원인 (정밀 특정)

기본 페이지네이션 엔진은 **`TypesetEngine`(`src/renderer/typeset.rs`)** (engine.rs Paginator 는
`RHWP_USE_PAGINATOR=1` fallback). 계측 트레이스로 드롭 지점 2곳 확인 — 둘 다 **빈 문단이 현재
페이지에 fit 안 되면 `continue` 로 통째 드롭**(단독 빈 페이지 차단 의도이나 문단까지 소실):

1. `typeset.rs` `next_will_vpos_reset` 빈 문단 분기 (`!(height_fits && vpos_fits)` → continue)
2. `typeset.rs` Task#967 force-break 빈 문단 분기 (`empty_h_px > avail` → continue)

직전 대형 표가 페이지를 채워 fit 실패 → 드롭. (예 `2957879 pi3`: tac 표 사이 빈 문단)

## 3. 해결

두 드롭 지점에서 `continue`(드롭) 대신 **현재 페이지에 0-높이로 흡수 기록**:
```rust
st.hidden_empty_paras.insert(para_idx);
st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
continue;
```
- `hide_empty_line`(L9223) 과 동일 시멘틱 — 빈 문단을 현재 페이지 하단 빈 줄로 흡수.
- **페이지를 advance 하지 않으므로** 단독 빈 페이지 회귀(synam-001/sample18/aift) 없음.
- 빈 문단이 한글과 동일 페이지(현재 페이지)에 남아 문단→페이지 매핑 정합.

변경: `typeset.rs` 단일 파일, 2개 분기.

## 4. 검증

| 항목 | 결과 |
|------|------|
| 고정 6건(이전 rhwp_pNone) | **6/6 MATCH 전환** |
| 회귀 격리 A/B (300건, old vs new 동일 한글데이터) | **REGRESSION 0**, IMPROVED +2 |
| **rhwp 페이지수 변동** | **0건** (단독 빈 페이지 회귀 없음 직접 입증) |
| `cargo test --release` | 사전존재 `form_01_keeps_nine_cfb_streams`(Windows 경로구분자) 1건 외 신규 회귀 0 |
| `hwpx_roundtrip_baseline` | 4 passed, 0 failed |

**회귀 0의 근거**: 드롭 경로는 빈 문단이 fit 안 될 때만 발동 → 그런 문단을 가진 문서는 종전에
이미 비-MATCH(문단수 부족). MATCH 문서는 이 경로를 타지 않아 영향 없음. 0-높이 흡수라 페이지
수 불변. A/B 가 실증(페이지수 변동 0).

검증용 한글 문서 2건을 `samples/task1706/` 에 동봉(메모리 룰 `rhwp-pr-include-hangul-docs`).

## 5. 비고

- 본 수정은 **누락(pNone) 제거**가 목표. 일부 케이스에서 빈 문단의 페이지가 한글과 한 칸
  어긋나는 잔여(표 적재량 차이)는 #1705(높이 모델)의 영역으로 별개.
- `form_01_keeps_nine_cfb_streams` 실패는 본 변경과 무관한 사전존재 플랫폼 테스트 버그.
