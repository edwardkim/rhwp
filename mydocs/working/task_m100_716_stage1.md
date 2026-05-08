# Task #716 Stage 1 (RED) 완료 보고서

**Issue**: [#716](https://github.com/edwardkim/rhwp/issues/716)
**Stage**: 1 — TDD RED
**작성일**: 2026-05-08
**브랜치**: `local/task716` (integration/3pr-stack 베이스)

---

## 산출물

- **신규 회귀 테스트**: `tests/issue_716.rs`
- **단언**:
  - `samples/20250130-hongbo.hwp` 페이지 0 의 RenderTree 에서 **Body 하위 모든 TextLine 의 bbox 하단이 Body 하단(=col_bottom) 이내**
  - 0.5 px 허용 오차 (sub-pixel rounding)
- **머리말/꼬리말 제외**: `RenderNodeType::Header` / `Footer` 자식은 본 결함과 무관하므로 traversal 시 cutoff
- **Body 하단 산출**: 페이지 RenderTree 의 `RenderNodeType::Body { .. }` 노드 bbox.y + bbox.height 를 동적으로 사용 (하드코딩 회피)

## 테스트 실행 결과 (RED — 의도된 FAIL)

```
$ cargo test --test issue_716 -- --nocapture

LAYOUT_OVERFLOW_DRAW: section=0 pi=15 line=2 y=1048.2 col_bottom=1028.0 overflow=20.1px
LAYOUT_OVERFLOW: page=0, col=0, para=15, type=PartialParagraph, y=1059.4, bottom=1028.0, overflow=31.3px
[issue_716] page 0 body=[x=75.59 y=94.47 w=642.53 h=933.57 bottom=1028.04]
            text_lines=38 max_bottom=1048.19 overflow=+20.15

panicked at tests/issue_716.rs:86:5:
page 0 본문 텍스트 줄이 Body 하단 초과: max_bottom=1048.19, body_bottom=1028.04, overflow=+20.15 px

test issue_716_page1_last_text_line_within_body ... FAILED
```

→ 결함 정확 검출:
- Body 하단 = 1028.04 px (이슈 본문 col_bottom=1028.0 과 일치, 0.04 px sub-pixel)
- 최대 max_bottom = 1048.19 px (이슈 본문 LAYOUT_OVERFLOW_DRAW y=1048.2 와 일치)
- overflow = +20.15 px (이슈 본문 overflow=20.1 px 와 일치)

stderr 의 `LAYOUT_OVERFLOW_DRAW` / `LAYOUT_OVERFLOW` 출력도 이슈 본문 그대로 재현.

## 베이스라인 환경

- 브랜치: `local/task716` (integration/3pr-stack 베이스 — Task #643/#712/#713 적용 완료 상태)
- page_count = 4 (4페이지 문서)
- 결함 페이지 인덱스: 0 (= page 1)
- 결함 위치: pi=15 line 2 (LAYOUT_OVERFLOW_DRAW)

## Body bbox 노출 검증

`find_body_bbox` 헬퍼가 `RenderNodeType::Body { .. }` 의 bbox 를 정확히 수집:
- x=75.59, y=94.47, w=642.53, h=933.57 → bottom=1028.04
- dump-pages 출력의 `body_area: x=75.6 y=94.5 w=642.5 h=933.6` 과 정합

## 다음 단계 (Stage 2 — 분석)

1. `RHWP_TASK716_DEBUG=1` 환경변수 가드 instrument 추가 (3 위치):
   - `layout.rs:1383` 영역 — column item 진입 시 pi/y_offset/expected/drift 출력
   - `layout.rs:2245-2252` 영역 — TAC 표 호스트 outer_margin 시점에 host_ls/advance 출력
   - `paragraph_layout.rs:2657` 영역 — line advance 시 lh/ls/y_after 출력
2. page 0 trace 수집 → drift 발생 단계 확인
3. 빈 문단(pi=1, pi=3)의 `+8/+12 px drift` 발생 경로 식별 (composer / paragraph_layout / layout)
4. H1 (TAC 표 호스트 ls 가산) 정확한 적용 위치 확정
5. Stage 2 보고서 작성 → Stage 3 GREEN 진입 승인 요청

## 승인 요청

Stage 1 RED 완료. Stage 2 (분석/instrument) 진행 승인 요청.
