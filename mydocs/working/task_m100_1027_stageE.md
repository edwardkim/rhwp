# Stage D+E1+E2 완료보고서 — #1027: 페이지네이터↔렌더러 측정 정합 (단단)

- 타스크: #1027 / 브랜치 `local/task1027`
- 작성일: 2026-05-20
- 단계: Stage D(vpos 스냅) + E1(표 advance) + E2(atomic-tac Shape) — 함께 착지
- 검증 샘플: `samples/2. 인공지능(AI) 기반 재정통합시스템 구축 용역 제안요청서.hwpx`
  (184p, 비공개) + `pdf/2. …-2022.pdf` (한컴 2022 권위)

## 1. 배경

Stage D 스냅 단독은 노트를 8쪽으로 옮기나 +5 LAYOUT_OVERFLOW(조사: `..._stageD.md`).
근본은 페이지네이터·렌더러의 **per-item advance 불일치**(설계 `..._stageE_impl.md`).
단단에서 표·Shape advance 를 렌더러와 정합하면 깨끗이 해결됨을 확인.

## 2. 변경 (`typeset.rs`, 단단 한정)

### Stage D — 항목 fit 직전 vpos 스냅
- `TypesetState` 에 vpos 상태(page/lazy base, prev, col_anchor) 보유, 컬럼 경계 reset.
- `vpos_snap_current_height()`: Stage C `HeightCursor::vpos_adjust` 를 current_height
  상대공간에서 호출해 단락 누적 drift 제거(렌더러 VPOS_CORR 와 동일 측정).

### Stage E1 — treat_as_char 인라인 표 advance 정합
- `typeset_block_table`: 글자처럼취급 표(단일 LINE_SEG 호스트)를 렌더러와 동일하게
  **호스트 LINE_SEG(fmt.total_height)** 로 advance(기존 effective_height 만 더해
  ~16.9px 과소 → 표 이후 overflow).

### Stage E2 — atomic top-fit 스필에서 위아래 글상자 제외
- `is_atomic_tac_singleton`: 진짜 인라인 atomic 개체(차트/그림, #409)만 60px 스필 허용.
  **위아래(TopAndBottom) 글상자(Shape)** 는 한컴이 본문 항목처럼 다음 페이지로 넘기므로
  제외 — 그렇지 않으면 페이지 하단에 잘못 스필(box pi=142 가 9쪽에 40.6px 스필).

## 3. 결과 (한컴 2022 PDF 대조)

| 항목 | Stage C(전) | Stage D+E1+E2(후) | 한컴 PDF |
|------|------|------|------|
| 노트 "추진일정은"(pi=127) | 9쪽(오류) | **8쪽** | 8쪽 ✓ |
| 글상자(pi=142) | (미회귀) | **10쪽** | 10쪽 ✓ |
| AI 184p 페이지 수 | 185 | 185 | — |
| AI 184p LAYOUT_OVERFLOW | 13 | **13** (−para642 19.7 +para429 5.1, 순개선) | — |
| svg_snapshot(공개) | 5 pass / 3 debt | 5 pass / 3 debt (회귀 0) | — |
| lib 테스트 | 1316 | 1316 | — |
| k-water-rfp | 29p / 3 ov | 29p / 3 ov (불변) | — |

- clippy 0. 전체 cargo test: issue_598(각주 마커 nav) 2건 실패는 **Stage C HEAD 에서도
  실패**하는 사전 부채(병합본, 본 변경 무관 — stash 검증 완료).

## 4. 진단 요약 (per-item parity)

- **plain 문단**: 이미 정합(diff=0) — 불변.
- **표**(p71 pi=349): 페이지네이터 65.2(effective) vs 렌더러 82.1(LINE_SEG) → E1 정합.
- **Shape**(pi=142): advance 자체는 정합(148.9≈148.8). 문제는 atomic top-fit 스필
  허용폭(60px) 과적용 → E2 로 위아래 글상자 제외.

## 5. 다음 (Stage E3, F)

- E3: 분할표 잔여행 + 다단(col_count>1) 스냅/advance(현재 단단 한정 가드 해제).
- F: 병합본 골든 부채(svg_snapshot 267/617/677, issue_598) 한컴 PDF 재판정·복구 + 최종 보고.
