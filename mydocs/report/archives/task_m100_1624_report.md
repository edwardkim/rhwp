# 최종 결과보고서 — Task #1624

**제목**: footer over-push 수정 — #1611 vpos 동기화의 +1쪽 부작용 정밀화
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1624 · **브랜치**: `local/task1624` (base: `local/task1618`)

## 1. 문제
#1611(footer Page+Bottom page-fit)이 −1쪽을 줄였으나, footer 의 stored vpos 가 본문 흐름과
동떨어진(앵커/누적 노이즈) 경우 vpos 동기화가 footer 를 spurious 하게 다음 쪽으로 밀어 +1쪽
over-push 유발(통제셋 +1쪽 6건, 코퍼스 오라클 ~3%).

## 2. 근본 (진단 RHWP_DBG_FOOTER)
판별자 = **gap (footer vpos − 본문 cur_h)**:
| 케이스 | vpos | cur_h | gap | 정당성 |
|--------|------|------|------|------|
| 36387725 (해소 −1) | 640.7 | 627.5 | +13 | sync 정당(push) |
| 36395270/36394590/36389909 (+1) | 987~1628 | 28~602 | 959~1026 | sync = spurious push |

over-push 케이스는 footer vpos 가 available(~990) 근접/초과하며 본문과 수백~천px 동떨어짐.

## 3. 수정 (`src/renderer/typeset.rs`)
vpos 가 흐름을 plausibly 따를 때만 동기화:
```rust
let sync_h = if target_y <= st.current_height + block_height {
    st.current_height.max(target_y)
} else {
    st.current_height // gap > footer 높이 → vpos 무시(over-push 방지)
};
```

## 4. 결과
| 지표 | #1611 후 | #1624 후 |
|------|--------|--------|
| 통제셋 일치 (92건) | 72 (78.3%) | **75 (81.5%)** |
| −1쪽 | 12 | **12 (회귀 0)** |
| +1쪽 | 6 | **3** (footer over-push 3건 해소) |

net **+3, −1 회귀 0**. footer-gap over-push 클래스 완결(잔여 +1 3건은 #1608 native tolerance
등 별 클래스). 전 회귀 게이트 통과: #1611 footer 테스트 유지, hwpx/visual baseline·opengov
스냅샷·lib 1976 tests·clippy/fmt 0.

## 5. 산출물
- 소스: `src/renderer/typeset.rs`
- fixture: `samples/hwpx/opengov/36395270_footer_overpush.hwpx` (+스냅샷 행)
- 테스트: `tests/issue_1624_footer_overpush_pagination.rs` (page_count==2)
- 문서: `_plan`, 본 보고서
