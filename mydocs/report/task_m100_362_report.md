# Task #362 최종 결과 보고서 — kps-ai p56 외부 표 안 콘텐츠 클립 + PartialTable nested + Square wrap

## 이슈

[#362](https://github.com/edwardkim/rhwp/issues/362) — `samples/kps-ai.hwp` p56 의 외부 표 안 콘텐츠가 클립되는 결함. 본 task 진행 중 외부 PartialTable + Square wrap 표 옆 paragraph 흡수 / 큰 nested table 분할 등 광범위한 TypesetEngine 결함 식별 + 정정.

## 결론

본 task 는 v0.7.3 (Paginator) → v0.7.6 (TypesetEngine default) 전환 시 누락된 **PartialTable nested table 처리 + Square wrap 어울림 영역 처리** 시멘틱을 TypesetEngine 에 이식하여 복원.

작업지시자가 본 시각 결함 (kps-ai p56, p67, p68, p70, p72) 모두 정정.

## 수정 내용 (8 항목 누적)

### 측정 단계 정정 (height_measurer.rs)
4. **nested table 셀 capping 강화** — `remaining_content_for_row` 의 nested 분기에서 외부 행 높이로 cap → PartialTable typeset 단계 partial_height 정확화

### 분할 단계 정정 (table_layout.rs)
1. **외부 셀 vpos 가드** — nested table 셀에서 LineSeg.vertical_pos 적용 제외
2. **PartialTable nested 분할 허용** — 한 페이지보다 큰 nested table atomic 미루기 대신 분할 표시
3. **PartialTable 잔여 height 정확 계산** — `calc_visible_content_height_from_ranges_with_offset` 신설

### 페이지네이션 시멘틱 정정 (typeset.rs)
5. **hide_empty_line 이식** — Paginator 의 페이지 시작 빈 줄 최대 2개 height=0 처리 시멘틱
6. **wrap-around 메커니즘 (Square wrap) 이식** ★ — Paginator engine.rs:288-372 의 wrap zone 매칭 + 활성화 시멘틱 이식. 외부 표 옆 paragraph 들이 height 소비 없이 흡수
7. **vpos-reset 가드 wrap zone 안 무시** — wrap zone 활성 중 vpos=0 paragraph 의 가드 오발동 차단
8. **Task #359 빈 paragraph skip 가드 강화** — 표/도형 컨트롤 보유 paragraph 는 skip 안 함 (pi=778 같은 빈 텍스트 + 표 케이스 보호)

## 검증

### 자동 회귀

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **1008 passed, 0 failed** |
| `cargo test --test svg_snapshot` | 6/6 통과 |
| `cargo test --test issue_301` | 1/1 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo check --target wasm32-unknown-unknown --lib` | 통과 |

### 7 핵심 샘플 + 추가 회귀

| 샘플 | 페이지 (수정 전→후) | LAYOUT_OVERFLOW (전→후) |
|------|-----|-----|
| form-01 | 1 → 1 | 0 → 0 |
| aift | 77 → 77 | 3 → 3 |
| KTX | 27 → 27 | 1 → 1 |
| **k-water-rfp** | 28 → **27** | **0 → 0** |
| exam_eng | 11 → 11 | 0 → 0 |
| **kps-ai** | **88 → 79** | 60 → 5 |
| hwp-multi-001 | 10 → 10 | 0 → 0 |

kps-ai 88 → 79 페이지 (Paginator 78 와 1 차이). LAYOUT_OVERFLOW 60→5 대폭 개선.

### 시각 판정 (작업지시자 통과)

- **kps-ai p56**: 외부 표 안 콘텐츠 클립 차단 ✅
- **kps-ai p67**: PartialTable nested 표 정상 표시 ✅
- **kps-ai p68**: 외곽 표 높이 정상화 ✅
- **kps-ai p68-70**: 빈 페이지 2개 차단 → 정상 흐름 ✅
- **kps-ai p72-73**: pi=778 표 누락 차단, 정상 흐름 ✅

## 진단 과정 요약

### Stage 1 — 결함 origin 정량
- height_measurer 의 측정값은 v0.7.3 와 main 동일
- → 결함 origin 은 layout 단계

### Stage 2 — layout 단계 origin 식별
- 셀 안 첫 paragraph y 가 main 에서 +19.47 px (vpos=2000 HU 추가 적용)
- Task #347 의 vpos 적용 로직이 nested table 케이스에 부적절 → 옵션 A 확정

### Stage 3 — 옵션 A → 옵션 B 확장
- 옵션 A 적용 후 시각 판정에서 추가 결함 발견 (page 67, 68, 71 등)
- 차근차근 진단:
  - PartialTable rows=0..3 의 nested 큰 표 atomic 미루기 결함 → 분할 허용
  - PartialTable 잔여 height 계산 결함 → calc_visible_content_height_with_offset 신설
  - 외부 표 옆 76 빈 paragraph 가 typeset 에 모두 표시 → wrap-around 이식
  - vpos-reset 가드 wrap zone 안에서 오작동 → 가드 무시
  - 빈 paragraph + 표 컨트롤 paragraph 잘못 skip → 가드 강화
- 8 항목 누적 수정 → 작업지시자 시각 판정 통과

### Stage 4 — WASM 빌드 + 시각 판정 + 보고서
- WASM Docker 빌드 (1m 18s)
- 작업지시자 시각 판정 통과
- 최종 보고서 + 트러블슈팅 + orders 갱신

## 산출물

- 코드:
  - `src/renderer/typeset.rs` (TypesetState 확장 + wrap-around 이식 + 가드 강화)
  - `src/renderer/layout/table_layout.rs` (외부 셀 vpos 가드 + PartialTable nested 분할 + height 계산 신설)
  - `src/renderer/layout/table_partial.rs` (PartialTable 잔여 height 계산 호출 변경)
  - `src/renderer/height_measurer.rs` (nested table 셀 capping)
  - `src/document_core/queries/rendering.rs` (typeset_section 시그니처 hide_empty_line 추가)
- 문서: 수행계획서, 구현계획서, stage1~3 보고서, 본 보고서, 트러블슈팅
- WASM: `pkg/rhwp_bg.wasm` (4.1 MB)

## 후속 과제 (별도 task 후보)

- **kps-ai 1 페이지 차이** — Paginator 78 vs main 79 (다른 미세 결함 가능)
- **kps-ai 잔존 LAYOUT_OVERFLOW 5건** — 다른 origin 으로 추정
- **Task #347 의 cell_y -7.2 px 보정 origin** — kps-ai p56 마지막 텍스트 y 가 v0.7.3 와 7.2 px 차이 (Stage 3 시점 잔존)

## 관련

- 이슈: [#362](https://github.com/edwardkim/rhwp/issues/362)
- 회귀 도입 commit: `edddebd Task #313: 2-3단계 - TypesetEngine default 전환`
- 관련 task: Task #313 (도입), Task #347 (좌표 정합), Task #359 (fit drift), Task #361 (page_num + PartialTable fit), Task #324 (compute_cell_line_ranges 재작성)
- 검증 기준: v0.7.3 (Paginator) 의 페이지 분할 + 시각 결과
