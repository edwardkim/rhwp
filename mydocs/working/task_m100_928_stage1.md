# Task #928 Stage 1: 정밀 재현 및 ROOT CAUSE 측정

## 1. 측정 환경

- 빌드: `cargo build --release` (task928 worktree, `c1eb48c6` 기반)
- 샘플: `samples/exam_kor.hwp` 5쪽 (`pi=145` 표 셀[5])
- 산출물: `/tmp/task928/exam_kor_005.svg`

## 2. IR 측정 (rhwp dump -s 0 -p 145)

**구역 0 문단 145** (페이지 5의 13번 문제 `<보 기>` 박스): 3×3 표, `tac=true wrap=TopAndBottom`

**문제의 셀**: 셀[5] r=2,c=0 rs=1,cs=3 (병합 셀), h=2850 w=30615, 안에 paragraph 3개:

| sub-para | ps_id | text_len | controls | 설명 |
|----------|-------|----------|----------|------|
| p[0] | 17 | 295 | 0 | "A 단계는 확산 모델 과정 중 한 단계이다. ㉠은 원본…" |
| p[1] | 37 | 22 | **사각형 ctrl[0]** tac=true wrap=TopAndBottom | **다이어그램 행** (`(가) ⇨ ⇨ (나)`) |
| p[2] | 38 | 26 | **그림 ctrl[0..2]** 모두 tac=true wrap=TopAndBottom | ㉠ ㉡ ㉢ 행 (그림 3개 화살표) |

p[1] 의 `ls[0] vpos=7852 lh=1984 ls=688` — **단일 LineSeg**. 즉 IR 상으로는 1줄.

p[2] 의 그림 3개도 동일 속성 (tac=true wrap=TopAndBottom) 인데 회귀 미발생.

## 3. SVG 출력 측정 (다이어그램 행, y≈421-423)

| baseline | x | 텍스트 | 비고 |
|----------|---|--------|------|
| **y=421.73** | 239.88 | `(` | font 15.33 |
|             | 246.88 | `가` | |
|             | 260.68 | `)` | |
|             | 279.88 | `⇨` | shape 앞 영역 ⇨ |
|             | 386.32 | `⇨` | shape 뒤 영역 ⇨ |
|             | 405.87 | `(` | |
|             | 412.80 | `나` | |
|             | 426.60 | `)` | |
| **y=423.12** | 292.88 | `(` | **baseline 2** (1.39 px 아래) |
|             | 299.40 | `가` | |
|             | 313.20 | `)` | |
|             | 332.75 | `⇨` | |
|             | 439.32 | `⇨` | |
|             | 458.87 | `(` | |
|             | 465.38 | `나` | |
|             | 479.18 | `)` | |
| y=420.63   | 379.34 | `A 단계` | font 6.88, 사각형 안 텍스트 |

**핵심**: y=421.73 라인과 y=423.12 라인 모두 **동일한 텍스트 패턴 `(가) ⇨ [공간] ⇨ (나)`** 을 emit. 단순 평행이동 (Δx≈+53, Δy≈+1.39). IR `ls[0]` 는 1줄뿐인데 SVG 에는 같은 패턴이 **2개 baseline 에 2회 emit** 됨.

같은 셀 p[2] (㉠㉡㉢) 는 y=482.62 한 줄만 emit (회귀 미발생).

## 4. ROOT CAUSE 1차 결론

p[1] 사각형 컨트롤이 포함된 표 셀 paragraph 가 layout 단계에서 **두 코드 경로로 중복 emit** 된다:

- **경로 A** (y=421.73): paragraph 텍스트가 한 baseline 에 그려짐 (`(가) ⇨` 사각형 위치 `⇨ (나)` 형태로 inline TAC split)
- **경로 B** (y=423.12, +1.39px): 동일 paragraph 텍스트가 다른 baseline 에 평행이동되어 한 번 더 그려짐

두 baseline 모두 같은 `⇨` 2개 (paragraph 원본 text 의 두 화살표) 를 포함한다 — split 결과가 아닌 paragraph 전체 텍스트가 그대로 출력되었음. 즉 동일 paragraph 가 **2회 layout** 되어 그려진 것.

같은 셀의 p[2] 그림 컨트롤 (tac=true wrap=TopAndBottom 동일 속성) 은 회귀 미발생 → **회귀는 사각형 (`Control::Shape`) 컨트롤 전용 경로에 한정**.

## 5. Stage 1 잠정 가설 (Stage 2 에서 검증)

### 가설 A: shape pushdown 누적 + inline TAC 동시 적용 (유력)

`src/renderer/typeset.rs:1006-1013` 에서 `Control::Shape` 가 `!tac && wrap=TopAndBottom && vert=Para` 조건일 때만 pagination `current_height` 에 shape 높이 누적 (pushdown). 하지만 tac=true 인 경우 이 조건을 빠져나가므로 pushdown 자체는 발생 안 함 — 단독으로는 회귀 원인 아님.

다른 경로: paragraph_layout 의 `tac_offsets_px` 와 typeset 의 wrap-area sub-paragraph 처리가 **양립 가능** 한 구조로 보임. Stage 2 에서 다음 검증:

1. `paragraph_layout.rs:1685 run_tacs` 분할이 동일 paragraph 에 2회 호출되는지
2. `typeset.rs` 에서 `Control::Shape (tac=true wrap=TopAndBottom)` 가 wrap-around sub-paragraph 등록을 일으키는지 (picture 와 다른 경로)
3. `set_inline_shape_position` 호출과 sub-paragraph 호출이 동일 ci 에 대해 둘 다 발생하는지

### 가설 B: HWP3 파서가 사각형 컨트롤 paragraph 를 sub-paragraph 로 복제 (가능)

`src/parser/hwp3/mod.rs` 의 도형 (rectangle) 컨트롤 처리 분기에서 paragraph 자체를 본문 + 셀 양쪽에 등록할 가능성. 그림 컨트롤은 동일 회귀 없음 → 도형 전용 분기 의심.

### 가설 C: char_offsets / FFFC 마커 중복 (낮음)

같은 셀 p[2] 의 그림 3개도 FFFC 다중 처리가 필요한데 회귀 없음 → 가설 C 는 가능성 낮음.

→ Stage 2 에서 가설 A 와 B 의 코드 경로를 실제 trace 출력으로 분기 확정한다.

## 6. 결정적 측정 결과

- ✅ 재현 100% (release 빌드, 위 명령으로 매번 동일)
- ✅ 회귀는 `Control::Shape` tac=true wrap=TopAndBottom 한정 (picture 무관)
- ✅ 동일 paragraph 가 2개 baseline 에 2회 layout (IR 의 `ls[]` 와 불일치)
- ⏳ 어느 코드 경로에서 중복 호출되는지는 Stage 2 에서 확정

## 7. Stage 2 진입 조건

본 보고 승인 후 Stage 2 에서:
1. 가설 A/B 검증 — `Control::Shape` tac=true wrap=TopAndBottom 의 inline 처리 vs wrap-around 처리 코드 경로 trace
2. ROOT CAUSE 코드 위치 확정
3. Fix 방향 결정 → 구현계획서 (`task_m100_928_impl.md`) 작성 및 승인 요청

코드 수정은 Stage 3 이후에만 진행.
