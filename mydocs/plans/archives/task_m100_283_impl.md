# Task #283 구현 계획서 — 수식 괄호 path 폭 조정

## 참고

- 수행계획서: [`task_m100_283.md`](task_m100_283.md)
- 이슈: [#283](https://github.com/edwardkim/rhwp/issues/283)

## 단계 구성 (5단계)

### 단계 1 — 기준선 + Times 글리프 실측

**목적**: 현재 파렌과 Times 폰트 글리프의 정량 차이 측정. 튜닝 목표치 도출 근거 확보.

**작업**:
- `samples/equation-lim.hwp` 현재 SVG → PNG (Task #280 단계 4 의 after 를 그대로 재사용하거나 재생성)
- 파렌 영역 확대 크롭 → 정밀 측정:
  - 파렌 폭 (픽셀)
  - 파렌 높이
  - 파렌과 인접 글자(`f`, `h`) 간 간격
- Chrome headless 로 Times New Roman `(` 글리프 실측:
  - `<canvas>.measureText("(")` → `.width` (advance width)
  - `<text>(</text>` + `getBBox()` → 실제 글리프 bounds
  - 결과: advance_width / em, bbox_width / em, bbox_height / em
- **참조 파렌 렌더**: `samples/equation-lim.hwp` 에서 `lim _{h rarrow 0}` 의 `lim` 과 같은 크기로 Times `<text>(</text>` 를 별도 SVG 로 렌더해, 우리 path 파렌과 시각적으로 나란히 배치

**산출물**:
- `mydocs/working/task_m100_283_stage1/`:
  - `current.svg`, `current.png`, `current_paren_crop.png` (현재 파렌)
  - `times_glyph.svg`, `times_glyph.png` (Times `(` 글리프 단독 렌더)
  - `compare.png` (나란히 비교 합성)
  - `metrics.md` (숫자 측정 결과)
- `mydocs/working/task_m100_283_stage1.md` (단계 보고서)

**완료 조건**: 작업지시자 승인 + 측정값 확정.

---

### 단계 2 — 튜닝안 파라미터 도출

**목적**: 단계 1 측정 결과로 새 `paren_w` · 제어점 비율 결정. 코드 수정 전 숫자 확정.

**작업**:
- 측정한 Times `(` 비율을 기준으로:
  - 새 `paren_w` 제안값 (예: `fs * 0.25` 또는 실측 비율)
  - `draw_stretch_bracket` 의 `(` 제어점 x 좌표 제안값 (예: `x + w*0.05` 또는 `x`)
- **프로토타입 검증**: 실제 코드 수정 없이 수동 SVG 샘플 작성 (`<path>` 직접 작성) 으로 시각 확인:
  - 파라미터 3~5 조합 테스트 (예: `paren_w ∈ {0.22, 0.25, 0.28}` × `ctrl_x ∈ {x+0, x+0.05w, x+0.1w}`)
  - 각 조합을 `f(2+h)` 문맥에 합성해 PNG 로 렌더
  - 최적 조합 선정
- 스트레치 케이스(가상 큰 파렌 예: 분수 감싼 `left( a over b right)`) 도 같은 파라미터로 확인
- `)` 도 대칭으로 확정

**산출물**:
- `mydocs/working/task_m100_283_stage2/`:
  - `variants/` — 파라미터 조합별 샘플 PNG (5~9개)
  - `selected.md` — 선정 파라미터와 근거
- `mydocs/working/task_m100_283_stage2.md` (단계 보고서)

**완료 조건**: 작업지시자가 선정 조합 승인.

---

### 단계 3 — 코드 변경 + 회귀 테스트

**목적**: 승인된 파라미터를 실제 코드에 반영.

**작업**:
- `src/renderer/equation/layout.rs:832` — `layout_paren` 의 `paren_w`
- `src/renderer/equation/svg_render.rs`:
  - L230, 237 — `LayoutKind::Paren` arm 의 `paren_w`
  - L204, 205 — `LayoutKind::Matrix` arm 의 `fs * 0.3` (동기)
  - `draw_stretch_bracket` 의 `(` `)` 제어점 (L279-294)
- `src/renderer/equation/canvas_render.rs`:
  - svg_render 와 동일 위치 동기 수정
  - L184, 188 — Paren arm
  - L160, 161 — Matrix arm
  - `draw_stretch_bracket` 의 `(` `)` 제어점 (L239-249)
- **매직넘버 주석 강화** — 각 변경 지점에 `// (Task #283) paren_w 튜닝 근거: Times ( glyph advance = 0.27em 실측` 등
- 회귀:
  - `cargo test --lib equation` — 수식 단위 테스트 (svg_render 내부 `test_paren_svg`, `test_eq01_svg` 포함) 전체 통과
  - `cargo test --test svg_snapshot` — 스냅샷 회귀
  - `cargo clippy --lib -- -D warnings`
  - `cargo check --target wasm32-unknown-unknown --lib`
- 스냅샷에 `fs * 0.3` 상수 기반 수치(예: `d="M... Q..."`) 가 포함되어 있으면 업데이트 필요 (단계 3 에서 실제 확인)

**산출물**: 코드 변경 + 단계 3 보고서 `mydocs/working/task_m100_283_stage3.md`

**완료 조건**: 작업지시자 승인.

---

### 단계 4 — 시각 비교 검증

**목적**: 수정이 실제 텍스트 파렌 어색함을 해소했는지, 스트레치 파렌·행렬 파렌·기타 수식이 깨지지 않는지 확인.

**작업**:
- 변경 후 SVG 생성 → PNG:
  - `samples/equation-lim.hwp` → after2.svg/png/crop (Task #280 기준)
  - `samples/exam_math.hwp` 회귀 5~7 페이지 — **스트레치 파렌 포함 페이지 우선**
    - Task #280 단계 4 와 동일 페이지(001/005/009/013/017) + 추가 페이지(스트레치 파렌 있는 곳)
- **3×1 비교 이미지**: 
  - pdf_crop (한컴 기준) / Task #280 after_crop (Phase 1 후) / Task #283 after_crop (Phase 2 후)
  - 3단계 개선 추이 시각화
- 확인 체크리스트:
  - [ ] `f(2+h)` 등 텍스트 높이 파렌이 본문과 어우러짐
  - [ ] 분수·sum 감싸는 스트레치 파렌 높이·모양 유지 (회귀 없음)
  - [ ] `{`, `[`, `|` 괄호 변화 없음 (이번 수정 범위 밖)
  - [ ] 행렬 파렌 변화 (`LayoutKind::Matrix` arm 수정 영향) — 샘플 있으면 확인
  - [ ] `left{ a_{n} right}` 같은 중괄호 회귀 없음

**산출물**: 
- `mydocs/working/task_m100_283_stage4/` — after2 이미지 + 회귀 페이지 PNG + 3단계 비교 합성
- `mydocs/working/task_m100_283_stage4.md` — 체크리스트 + 결과

**완료 조건**: 작업지시자 승인.

---

### 단계 5 — 최종 보고서 + orders 갱신

**목적**: 타스크 종결.

**작업**:
- `mydocs/report/task_m100_283_report.md` — 최종 보고서
- `mydocs/orders/20260424.md` 에 Task #283 섹션 추가
- Phase 3 후속 이슈 후보 정리 (B안 글리프 분기, 공용 상수 추출, 기타 괄호 `{` `[` 등 동일 조사) — 필요 시 이슈 등록
- **주의**: 단계 5 커밋 전 `git status` 로 미커밋 파일 없는지 확인

**커밋**: 최종 보고서 + orders 갱신

**완료 조건**: 작업지시자 승인 → `local/task283` 를 `devel` 에 merge → `gh issue close 283`

---

## 전체 변경 파일 예상

| 파일 | 단계 | 변경 유형 |
|------|------|-----------|
| `src/renderer/equation/layout.rs` | 3 | `paren_w` 상수 1곳 |
| `src/renderer/equation/svg_render.rs` | 3 | `LayoutKind::Paren`(2), `Matrix`(2), `draw_stretch_bracket`(2) |
| `src/renderer/equation/canvas_render.rs` | 3 | svg_render 동일 수정 |
| `mydocs/plans/task_m100_283{,_impl}.md` | 작성 완료 | 신규 |
| `mydocs/working/task_m100_283_stage{1,2,3,4}.md` | 각 단계 | 단계별 보고서 |
| `mydocs/working/task_m100_283_stage{1,2,4}/*` | 각 단계 | 시각 비교 이미지 |
| `mydocs/report/task_m100_283_report.md` | 5 | 최종 보고서 |
| `mydocs/orders/20260424.md` | 5 | Task #283 섹션 |

## 리스크

- **스트레치 파렌 품질 저하** — 텍스트 파렌 최적화가 스트레치 파렌 품질을 떨어뜨릴 수 있음. 단계 2 에서 스트레치 케이스 동시 확인 필수.
- **행렬 파렌 회귀** — 행렬 샘플 확보 어려움. 단위 테스트(`test_paren_svg`, 가능하면 행렬 테스트 추가) 로 최소 방어.
- **Task #280 머지 순서** — `local/task283` 이 `local/task280` 위에 쌓임. 병합 순서:
  1. `local/task280` → `devel` 먼저 머지 + push
  2. `local/task283` → `devel` 머지 + push
  반대 순서로 하면 task280 변경이 task283 머지에 포함되어 혼란. 단계 5 머지 전 반드시 확인.

## 예상 소요

- 단계 1: ~25분 (Chrome 측정 스크립트 + 현재 샘플 크롭)
- 단계 2: ~30분 (파라미터 조합 프로토타입 렌더)
- 단계 3: ~20분 (코드 수정 + 회귀)
- 단계 4: ~30분 (시각 비교 + 회귀 샘플)
- 단계 5: ~20분 (최종 보고서 + orders)
- **총 예상**: 약 2시간
