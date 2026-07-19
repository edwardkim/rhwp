# 구현계획서 — Task #1248: render/pagination trailing 모델 통일 (조사)

- **이슈**: edwardkim/rhwp#1248
- **브랜치**: `local/task1248` (base: `stream/devel`)
- **수행계획서**: [`task_m100_1248.md`](task_m100_1248.md)
- **성격**: 조사·설계 전용 — **코드 0줄 수정**. 산출물은 문서뿐.

> 본 타스크는 코드 변경이 없으므로 각 단계의 "커밋"은 **문서 커밋**(조사 문서 + stage 보고서)만 포함한다.

---

## 단계 구성 (3단계)

### Stage 1 — 현황 맵: 양쪽 trailing 분기 전수 추출

**목적**: render 와 pagination 이 trailing(line_spacing)을 다루는 모든 지점을 한 표로 집약.

**작업**:
1. `src/renderer/height_cursor.rs` — `vpos_adjust` 및 주변에서 trailing/line_spacing 을 가산·제외·복원하는 지점 추출
2. `src/renderer/pagination/engine.rs` — `trailing_ls` 관련 분기 전수 (현재 식별: 507, 547, 1148, 1248, 1858, 1884, 2204 부근) — 각 분기의 ① 위치 ② 조건 ③ 동작(제외/복원/유지)
3. `src/renderer/typeset.rs`, `src/renderer/layout.rs` 에 trailing 주입/배선 지점 있으면 포함

**산출물**:
- `mydocs/tech/investigations/issue-1248/trailing_model_render_vs_pagination_1248.md` §1 "현황 맵" 표 초안
- `mydocs/working/task_m100_1248_stage1.md` (단계 완료 보고서)

**완료 기준**: 양쪽 trailing 처리 지점이 누락 없이 표로 정리됨. → 승인 요청.

---

### Stage 2 — vpos_adjust 특례 8종 해부 + 핀 고정 테스트 역추적

**목적**: 각 특례가 "왜 존재하는가 / 무엇이 깨지면 안 되는가"를 증거와 함께 고정.

**작업** (특례 8종):
`compact_endnote_new_note_jump` / `stale_note_gap` / `tac_picture_gap` / `bottom_rewind` /
`deep_backtrack` / `title_tail_backtrack` / `safe_vpos_backtrack` / (#1246) `min-gap`
- 각각: ① 트리거 조건(코드) ② 도입 이슈/샘플(주석·커밋·issue 번호) ③ 핀 고정 회귀 테스트(`issue_*`, 단위테스트) 매핑
- 메모리 `tech_lazy_base_trailing_ls_gate` (Task #1022 v2) 교훈을 "통일 시 위험" 근거로 연결

**산출물**:
- 조사 문서 §2 "특례 8종 해부" + §3 "불일치 지점" 초안
- `mydocs/working/task_m100_1248_stage2.md`

**완료 기준**: 8종 모두 존재 이유 + 핀 고정 테스트가 표로 매핑됨. → 승인 요청.

---

### Stage 3 — 판정 + 후속 이슈 제안 + 최종 보고

**목적**: "통일 가능/불가" 결론과 다음 행동 제안.

**작업**:
1. 조사 문서 §4 "판정": trailing 모델을 ⓐ 통일 가능 영역 ⓑ 게이트 필수(통일 불가) 영역 으로 구분
2. 실제 리팩터링 착수 권고 여부 + (권고 시) 후속 이슈 골격 제안
3. 조사 문서 완결 + 최종 결과보고서 작성

**산출물**:
- `mydocs/tech/investigations/issue-1248/trailing_model_render_vs_pagination_1248.md` 완성본
- `mydocs/report/task_m100_1248_report.md` (최종 결과보고서)
- `mydocs/working/task_m100_1248_stage3.md`

**완료 기준**: 판정과 근거가 문서로 확정. → 최종 승인 요청.

---

## 검증 방법

- 코드 변경이 없으므로 빌드/테스트 회귀는 비대상.
- 단, 조사 중 인용하는 테스트/거동은 **실제 실행**(`cargo test issue_xxxx`, `dump-pages`)으로 사실 확인 후 기재.
- 문서 내 모든 코드 인용은 `file:line` 으로 추적 가능하게 표기.

## 비범위 (재확인)

코드 수정 / 동작 변경 / 새 테스트 / orders 갱신 — 전부 제외.

## 승인 요청

본 구현계획서 승인 후 Stage 1 착수.
