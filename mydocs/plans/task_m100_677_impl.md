# 구현계획서 — Task #677

## 사전 진단 노트

수행계획서 작성 후 추가 코드 추적으로 결함 #1 의 본질을 더 좁혔다. **Stage 1 진단에서 최종 확정**.

### 결함 #1 — pi=16 라인 누적 263~273px 정량 분석

**HWP IR 사실관계**:
- ls[0]: vpos=55288 lh=1000 (~13.3px) — 채움 placeholder 198 chars
- ls[1]: vpos=56888 lh=21296 (~284px) — 표 인라인 라인 (chars 198~218: table + shape control)
- ls[1].vpos - ls[0].vpos = 1600 HU = 21.3px (HWP 가 확정한 라인 1 시작 y)
- ls[1].vpos + ls[1].lh = 78184 HU = 1042.5px (body 상대) = **1080.3px (page 상대)** → body bottom 1084.7px **안에 들어맞음** ✅

**rhwp 실측**:
- LAYOUT_OVERFLOW_DRAW: line=1 y=1348.2 col_bottom=1084.7 overflow=263.5px
- y(line=1 bottom) = 1348.2 → text_y(line=1) = 1348.2 - 284 = 1064.2 → text_y - body_offset = 1026.4px (body 상대) = 76983 HU
- HWP IR 정답 y(line=1 top) = 56888 HU = 758.5px (body 상대)
- **실측 - 정답 = 76983 - 56888 = 20095 HU ≈ 268px ≈ 표 높이 (21016 HU = 280.2px)**

**가설** (Stage 1 진단에서 검증/반박):
- A. **layout 의 y 누적이 ls.vpos 무시 + 표 높이 가산**: 라인 별 y 를 ls.vpos 로 매번 보정하지 않고, 직전 라인의 (corrected) line_height + line_spacing 만큼 누적. 라인 0 의 표 marker 가 line_height 에 표 전체 높이 (280px) 를 가산했고, 그 결과 라인 1 의 y 가 표 바닥 + 약간 (line_spacing 차이) 위치로 밀림.
- B. **TAC 표 라인 1 의 line_height 자체 결함**: ls[1].lh=21296 이 표 + 본문 합산이 아니라 표만의 높이여야 한다는 가설. (덜 유력 — HWP IR 표준 정합 시 lh=line 전체 가시 높이로 봄.)
- C. **본문 2줄 (※ 군필자)이 별도 라인이 아니라 ls[1] 안에 내포**: ls 가 2개 뿐이지만 PartialParagraph 는 lines=1..2 이라 line=1 만 partial. 본문 ※ 2줄은 ls 가 누락되었거나 다른 곳에서 생성. 만일 측정 의존 fallback 이 라인 추가하면서 y 가 누적되면 결함.

**우선 가설 A** (가장 유력) — 진단 단계에서 코드 추적으로 확정.

### 결함 #2 — 워터마크 효과 정량 분석

**HWP IR 사실관계**:
- effect=GrayScale, brightness=-50, contrast=70, watermark=custom (`is_watermark()` true, `is_hancom_watermark_preset()` false)
- 한컴 워터마크 프리셋: GrayScale + brightness=+70 + contrast=-50 (`src/model/image.rs:75`) — **부호 반대**

**rhwp 실측**:
- SVG `<filter>` 가 brightness=-50 + contrast=70 적용 → 어두운 + 고대비 (표준 워터마크의 반대)

**가설**:
- A. **한컴 편집기는 effect=GrayScale 시 brightness/contrast 를 워터마크용 inverse 변환으로 적용** (저장값을 그대로 적용하지 않음). 즉 brightness=-50 저장값은 실제로 +50 시각 효과로 해석.
- B. **파서 부호 처리 결함** — `read_i8` 자체는 정상 i8 부호확장이므로 거의 가능성 낮음. 다른 fixture 의 워터마크 저장값 교차 검증 필요.
- C. **별도 alpha/opacity 필드** — HWP5 image_attr 에 brightness/contrast 외 watermark alpha 가 있을 가능성 (HWP5 spec 추가 확인).

**우선 가설 A** 또는 추가 알파 채널 — Stage 1 진단에서 spec 확인 + 다른 워터마크 fixture 교차 검증으로 결정.

---

## 단계 분해 (5 stages)

### Stage 1 — 진단 (코드 + 완료 위치)

**산출물**: `mydocs/working/task_m100_677_stage1.md`

**조사 영역**:

1. **결함 #1 코드 추적 (paragraph_layout.rs)**:
   - `paragraph_layout.rs` 의 line 반복 진입 시 `y` 의 초기값 + 라인별 advance 로직 정확 식별
   - `comp_line.line_height` (ComposedLine) vs `LineSeg.line_height` (HWP IR) 매핑 확인
   - 라인 0 의 corrected_line_height 실측 (eprintln 디버그 출력 추가 후 재현)
   - `tac_offsets_px` 가 line_height 또는 y advance 에 영향 주는 분기 식별

2. **결함 #1 IR vs 측정 검증**:
   - `dump-pages` 기반 표 위치 + 라인 위치 정량 측정
   - 다른 인라인 표 fixture (예: `table-text.hwp`, `form-002.hwp`) 와 교차 — pi=16 영역 결함이 본 fixture 한정인지 광범위인지 확인

3. **결함 #2 HWP5 spec 추가 확인**:
   - HWP5 image_attr 필드 명세 재검 (effect/brightness/contrast 외 추가 alpha/opacity 영역)
   - 한컴 편집기의 워터마크 프리셋 변환 로직 (`mydocs/tech/` 의 image effect 자료 + 작업지시자 권위 영역)
   - 다른 워터마크 fixture (effect=GrayScale + watermark=custom) 교차 검증

4. **본질 영역 결정**:
   - 결함 #1 의 가설 A/B/C 중 본질 영역 확정
   - 결함 #2 의 가설 A/B/C 중 본질 영역 확정
   - 정정 코드 영역 좁힘 (특정 함수 + 라인 영역)

**검증**:
- BEFORE 측정값 정량 기록 (LAYOUT_OVERFLOW + SVG byte size + 시각)
- 본질 영역 식별 + 정정 코드 좁힘 명시
- 가설 A 검증 / 가설 A 반박 시 가설 B/C 로 전환

**승인 요청**: 본질 영역 + 정정 영역 확정 후 Stage 2 진행 승인

---

### Stage 2 — 결함 #1 본질 정정

**산출물**: `mydocs/working/task_m100_677_stage2.md`

**대상 영역** (Stage 1 진단 결과로 확정, 예비):
- 가설 A 시: `src/renderer/layout/paragraph_layout.rs` 의 line 반복 y 누적 영역 — 라인별 y 를 LineSeg.vpos 기준으로 보정 (HWP IR 표준 직접 사용, 측정 의존 회피)
- 가설 B 시: `src/parser/control/table.rs` 또는 typeset 의 LineSeg.lh 계산 영역
- 가설 C 시: `src/renderer/typeset.rs` 의 ComposedLine 생성 영역

**제약**:
- **케이스별 명시 가드** — 인라인 TAC 표 + ls.vpos 정합 분기로만 좁힘. 다른 paragraph 영역 회귀 0
- **단일 룰** — HWP IR 표준 (LineSeg.vpos / LineSeg.line_height) 직접 사용, 측정/휴리스틱 회피
- **HWP3 분기 금지** — 본 영역은 HWP5 결함 (`src/parser/hwp3/` 외부 미수정)
- **함수 시그니처 변경 시 호출처 동기 정정** — Task #621 학습 영역 정합

**검증**:
- LAYOUT_OVERFLOW: pi=16 0건 (BEFORE: 3건)
- cargo test --release --lib: 1141+ passed (회귀 0)
- cargo test --release: GREEN
- clippy: 0 warnings
- build --release: 성공
- 시각 1차 (rsvg-convert PNG): 접수증 영역 정상 배치 확인

**커밋**: 코드 정정 + Stage 2 보고서를 같은 단위로 커밋

**승인 요청**: 결함 #1 정정 + 회귀 0 확정 후 Stage 3 진행 승인

---

### Stage 3 — 결함 #2 본질 정정

**산출물**: `mydocs/working/task_m100_677_stage3.md`

**대상 영역** (Stage 1 진단 결과로 확정, 예비):
- 가설 A 시: `src/renderer/svg.rs` 의 `ensure_brightness_contrast_filter` + `ensure_image_effect_filter` — `is_watermark()` true 케이스에서 한컴 워터마크 변환 (저장값 부호 inverse + opacity) 적용
- 가설 B 시: `src/parser/control/shape.rs:848` 영역 부호 처리 (가능성 낮음)
- 가설 C 시: 추가 alpha 필드 파싱 + 렌더 영역

**제약**:
- **케이스별 명시 가드** — `is_watermark()` true + (effect=GrayScale 또는 BlackWhite) 케이스만 정정. RealPic 영역 회귀 0
- **단일 룰** — HWP5 spec 정합 변환 함수, 휴리스틱 회피
- **WASM API 영향** — `web_canvas.rs:94-117` 의 `compose_image_filter` 도 동일 영역 동기 정정 (SVG / Canvas 양쪽 일관)

**검증**:
- 시각 1차: 워터마크 흐림 정합 (PDF 와 비교)
- cargo test --release --lib: 1141+ passed
- cargo test --release: GREEN
- clippy: 0 warnings
- 다른 워터마크 fixture (있다면) 회귀 0
- WASM build: success (Docker)
- rhwp-studio web 환경: 워터마크 정합 (`feedback_image_renderer_paths_separate` 정합)

**커밋**: 코드 정정 + Stage 3 보고서

**승인 요청**: 결함 #2 정정 + 회귀 0 확정 후 Stage 4 진행 승인

---

### Stage 4 — 회귀 가드 영구 보존 + 광범위 sweep

**산출물**: `mydocs/working/task_m100_677_stage4.md`

**작업**:

1. **svg_snapshot 회귀 가드 추가**:
   - `tests/svg_snapshot.rs::issue_677_bokhakwonseo_pi16` 신규
   - `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` 영구 보존 (Stage 2/3 정정 후 산출물)

2. **광범위 페이지네이션 sweep**:
   - 본 환경 fixture (`samples/` 167+ 영역) sweep — 페이지 수 차이 0 확인
   - BEFORE/AFTER SVG byte 비교 — 의도된 영역 외 byte-identical 확인

3. **다른 워터마크 fixture 교차 회귀 검증**:
   - effect=GrayScale + watermark=custom 영역 다른 fixture 시각 회귀 0

4. **회귀 차단 가드 정합 영역**:
   - 기존 svg_snapshot 6/7 통과
   - issue_546 1/1, issue_554 12/12, issue_617 1/1, issue_598 4/4 통과

**검증**:
- cargo test --release --lib: 1142+ passed (issue_677 신규 +1)
- cargo test --release: GREEN
- 광범위 sweep 페이지 수 차이 0
- BEFORE/AFTER 의도 영역만 변경 확인

**커밋**: svg_snapshot + golden + 광범위 sweep 결과를 같은 단위로 커밋

**승인 요청**: 회귀 가드 + 광범위 회귀 0 확정 후 Stage 5 진행 승인

---

### Stage 5 — 메인테이너 시각 판정 + 잔존 영역 점검

**산출물**: `mydocs/working/task_m100_677_stage5.md`

**작업**:

1. **메인테이너 시각 판정 영역**:
   - `output/svg/task677/복학원서.svg` + PNG 생성
   - PDF (`pdf/복학원서-2022.pdf`) 와 시각 비교 → 작업지시자 시각 판정 통과 확인
   - rhwp-studio web 환경 시각 판정 (`feedback_visual_judgment_authority` 정합)

2. **잔존 영역 점검**:
   - 다른 페이지 / 다른 영역 회귀 0 재확인
   - body-clip width 1619.92 영역 (시각 영향 없음, 별도 후속 task 후보로 분리 권유)

3. **WASM 빌드**:
   - Docker WASM 빌드 + 현재 사이클 baseline 대비 byte 변화 정량 측정
   - rhwp-studio npm run build TypeScript 타입 체크 + dist 빌드

**검증**:
- 작업지시자 시각 판정 ★ 통과
- 모든 회귀 0 재확인
- WASM 빌드 성공 + byte 변화 정합

**커밋**: WASM + studio 빌드 산출물 + Stage 5 보고서

**승인 요청**: 시각 판정 + 잔존 점검 후 **최종 보고서** (`task_m100_677_report.md`) 작성 진행 승인

---

## 회귀 위험 영역 요약

| 영역 | 위험도 | 가드 |
|------|--------|------|
| `paragraph_layout.rs` line y 누적 | **고** — 모든 paragraph 영역 영향 | 광범위 sweep + svg_snapshot 6+ + issue_xxx 회귀 가드 |
| `svg.rs` brightness/contrast filter | **중** — 모든 brightness/contrast 적용 image | 다른 워터마크 fixture 시각 + 광범위 sweep |
| `web_canvas.rs` compose_image_filter | **중** — 모든 brightness/contrast 적용 image (web) | rhwp-studio web 환경 시각 |

## 처리 후속 영역 (별도 task 후보)

- **body-clip width 1619.92 결함** — 시각 영향 없음, 코드 위생 영역 (별도 task 권유)
- **다른 인라인 TAC 표 fixture 사례** — 본 task 정정이 좁은 영역인지 광범위 영역인지 — Stage 1 진단 결과 따라 분리 결정

## 승인 요청

본 구현계획서 승인 후 **Stage 1 (진단)** 진행하겠습니다.
