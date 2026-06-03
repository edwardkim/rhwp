# Task 261 수행 계획서: 페이지 리플로우 근본 수정

## 현상

쪽 나누기/문단 삽입 후 후속 문단의 y 위치가 비정상:
- "가. 인력투입 계획" 사라짐
- 표와 문단 사이 간격 비정상
- E2E 재현: `e2e/page-break.test.mjs` (biz_plan.hwp, 문단 68)

## 근본 원인

`vertical_pos`(vpos) 체계의 불일치:
- 원본 HWP: 각 문단의 `line_segs.vertical_pos`가 **구역 시작** 기준 절대값
- `reflow_line_segs`: vpos를 원본 첫 LineSeg 값부터 누적 재계산
- 쪽 나누기로 문단이 다른 페이지로 이동하면 원본 vpos와 실제 페이지 내 위치가 불일치
- layout.rs의 vpos 보정이 이 불일치를 처리하지 못함

## 과거 관련 작업

- **Task 142**: Break Token 자료구조 도입
- **Task 198**: vpos 보정을 page_index==0 전용에서 전체 페이지로 확장 (vpos_page_base / vpos_lazy_base)
- **Task 215**: typeset_block_table() Break Token 기반 행 분할
- **연구 문서**: `mydocs/tech/layout_engine_research.md` — MS Word, LibreOffice, Chromium LayoutNG, Typst 분석

이미 Break Token 패턴을 도입하여 측정-배치 불일치를 해소한 경험이 있으므로,
동일한 패턴을 vpos 문제에도 적용하는 것이 일관성 있는 접근이다.

## 해결 방향

### B안: 편집 후 후속 문단 vpos 재계산 (권장)
- 문단 삽입/삭제/쪽 나누기 후 **영향받는 모든 문단**의 vpos를 순차적으로 재계산
- 이전 문단의 `vpos_end`를 다음 문단의 `vpos_start`로 전파
- 표 문단은 `line_height`를 보존하고 vpos만 갱신
- 장점: vpos 체계 정합성 보장, Break Token 패턴과 일관
- 단점: 표/도형 등 특수 문단의 line_height 보존 필요

## 참조 파일

| 파일 | 역할 |
|------|------|
| `src/renderer/layout.rs` (1010~1043) | vpos 보정 로직 |
| `src/renderer/composer/line_breaking.rs` | reflow_line_segs (vpos 재계산) |
| `src/renderer/pagination/engine.rs` | 페이지 분할 |
| `src/document_core/commands/text_editing.rs` | insert_page_break_native |

## E2E 테스트

- `e2e/page-break.test.mjs`: biz_plan.hwp 쪽 나누기 검증
- `e2e/footnote-vpos.test.mjs`: footnote-01.hwp vpos 점프 검증
