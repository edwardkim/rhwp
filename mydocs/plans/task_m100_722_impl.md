# Task #722 구현 계획서

## 개요

Issue #722 "hwp3-sample5.hwp 페이지 8 wrap=Square 그림 paragraph 첫 줄 위치 불일치" 정정 구현. 3 단계 진행.

## Stage 1: 본질 진단 — 가설 A/B/C 식별

### 목표

paragraph 175 LINE_SEG cs/sw 인코딩의 본질 위치 식별. HWP3 파서, layout, typeset 중 어느 영역의 결함인지 정확히 판단.

### 진단 절차

1. **HWP5 변환본 IR 비교**
   - `samples/hwp3-sample5-hwp5-v2018.hwp` / `_v2024.hwp` 의 동일 paragraph (page 8 "아래에 디렉토리 트리...") IR 덤프
   - cs/sw 값 비교 — 한컴이 어떻게 인코딩하는가
   - HWP3 native vs HWP5 변환본 cs/sw 차이 영역

2. **HWP3 파서 cs/sw 처리 추적**
   - `src/parser/hwp3/mod.rs` cs/sw 인코딩 영역 (Task #604 Stage 3 정정 영역)
   - paragraph 175 의 cs/sw 가 어디서 결정되는지

3. **그림 wrap_zone vs paragraph 첫 줄 영역**
   - 그림의 vertical_offset (65.9mm) 과 paragraph 175 시작 vpos (12960) 의 관계
   - paragraph 첫 줄이 그림 시작 전 영역에 들어가야 하는지

### 진단 산출물

`mydocs/working/task_m100_722_stage1.md` — 본질 진단 결과 + 가설 A/B/C 중 정정 영역 결정.

### 승인 요청

Stage 1 진단 결과 + Stage 2 정정 방향 승인 요청.

## Stage 2: 본질 정정 — 정정안 E (2026-05-08 재갱신)

### 목표

PDF 권위 시각 판정 결과 D'' 가설 폐기 후 본질 재진단 (Stage 1 §13~§15) 에 따른 정정.

paragraph 175 (anchor host paragraph) 가 본 환경 typeset.rs wrap_around state machine 에서 wrap_anchors 에 미등록 → paragraph_layout 의 LINE_SEG cs/sw 적용 분기 미진입 → col_area 전체 폭 layout → image 영역 침범 → image z-order 후 그려져 텍스트 가려짐.

### 정정 절차

**1. 기존 Stage 2 정정 rollback**

다음 변경 전체 rollback (D'' 가설 기반, 본질 미정합):
- `src/renderer/layout/paragraph_layout.rs` — ComposedLine merge 분기, host_pic_vertical_offset 가드, wrap_anchor.anchor_vpos 가드
- `src/renderer/pagination.rs` — WrapAnchorRef.anchor_vpos 필드 추가
- `src/renderer/typeset.rs` — anchor_vpos 추출 + DEBUG_TASK722 로그

**2. 본질 정정** (`src/renderer/typeset.rs`)

wrap_around state machine 에서 anchor host paragraph (자기 자신) 도 wrap_anchors 에 등록:

```
기존: anchor 다음 paragraph (176, 177) 만 register
정정: anchor host (paragraph 175) + 다음 paragraph (176, 177) 등록
```

paragraph_layout 의 wrap_anchor 처리는 그대로 활용. 한 곳 수정.

### Case 가드 (회귀 위험 좁힘)

본 환경의 wrap_around state machine 의 register 영역에 host paragraph 추가:
- wrap=Square 그림이 있는 paragraph 자기 자신도 wrap zone layout 처리
- 다른 paragraph 영향 없음 (typeset.rs 의 wrap_around state machine 발현 paragraph 한정)

### 검증 절차

1. **결정적 검증** (release 모드)
   - `cargo test --lib --release` 회귀 0 (1165+ passed)
   - svg_snapshot 6/6 + issue_546 1/1 + issue_554 12/12
   - `cargo clippy --release` 신규 경고 0
2. **시각 검증** (rsvg-convert PNG 변환 후 작업지시자 자체 판정)
   - `samples/hwp3-sample5.hwp` 페이지 8 — paragraph 175 가 image 우측 wrap zone 첫 줄 표시
   - `pdf/hwp3-sample5-2022.pdf` 페이지 8 정합
   - 다른 페이지 회귀 0
3. **광범위 페이지네이션 sweep** (Stage 3 와 통합)

### 정정 산출물

- 영향 코드 변경 (`src/renderer/typeset.rs`) — rollback 후 신규 정정
- `mydocs/working/task_m100_722_stage2.md` 단계별 보고서

### 승인 요청

Stage 2 정정 결과 + 시각 판정 통과 후 Stage 3 진행 승인.

## Stage 3: 페이지 27 본질 진단

### 목표

페이지 27 본문 image (anchor paragraph 440 추정) 영역의 wrap zone 후속 paragraph layout 결함 본질 식별.

baseline (정정 적용 전) 과 정정안 E 적용 후 결과가 동일 → 본 결함은 페이지 8 정정과 별개 본질.

### 진단 절차

1. **IR 덤프**
   - `rhwp dump samples/hwp3-sample5.hwp -s 0 -p 440` 등으로 paragraph 440 (anchor host) + 441/442/443 (wrap_anchors 등록 paragraph) IR 비교
   - LINE_SEG cs/sw vpos 인코딩 확인

2. **PDF 권위 vs 본 환경 SVG 출력 비교**
   - PDF: 본문 image 우측 wrap zone 텍스트 흐름
   - 본 환경: 큰 빈 공간 — 텍스트가 어디로 layout 되는지 추적

3. **paragraph_layout / typeset wrap_around 분기 추적**
   - paragraph 441 등 wrap_anchors 등록 후 layout 결과
   - cs/sw 적용 영역 vs 실제 SVG 출력 차이

### 진단 산출물

`mydocs/working/task_m100_722_stage3.md` — 본질 진단 결과 + Stage 4 정정 방향 결정.

### 승인 요청

Stage 3 진단 결과 + Stage 4 정정 방향 승인 요청.

## Stage 4: 페이지 27 본질 정정

### 목표

Stage 3 진단에 따라 본질 위치에 정정 적용. 정정 영역·case 가드 명시.

### 검증 절차

1. **결정적 검증** (release 모드)
   - cargo test --lib --release 회귀 0
   - cargo clippy --release 신규 경고 0
2. **시각 검증** (rsvg-convert PNG 변환 자체 판정)
   - 페이지 8 정합 보존 (Stage 2 정정 영역 회귀 0)
   - 페이지 27 PDF 권위 정합

### 정정 산출물

- 영향 코드 변경
- `mydocs/working/task_m100_722_stage4.md` 단계별 보고서

### 승인 요청

Stage 4 정정 결과 + 시각 판정 통과 후 Stage 5 진행 승인.

## Stage 5: inter-image-text gap 본질 진단

### 목표

페이지 48 paragraph 1394 (Stage 4 후 wrap zone 회복) 의 image 끝 → 텍스트 시작 사이 gap 차이 본질 식별.

### 진단 결과 요약

- 모든 wrap=Square image 의 outer margin = **852 HU = 3.0mm** (좌우상하 동일)
- 한컴 viewer 는 `text 시작 x = col_area.x + cs + image_margin_right` 으로 추정
- 본 환경은 `text 시작 x = col_area.x + cs` (margin 미적용)
- 차이 = ~5.3 px = ~1.33mm (image margin_right=852 HU = 3mm = 11.3 px 만큼 한컴이 더 우측에 위치, 대신 첫 char width 6px 만큼 본 환경 보정 → 실제 차이 ~5px)

### 진단 산출물

`mydocs/working/task_m100_722_stage5.md` — 본질 진단 결과 + Stage 6 정정 방향 결정.

### 승인 요청

Stage 5 진단 결과 + Stage 6 정정 방향 승인 요청.

## Stage 6: inter-image-text gap 본질 정정

### 목표

WrapAnchorRef 에 image outer margin 정보 추가 + paragraph_layout 의 LINE_SEG cs 적용 시 margin_right 보정.

### 정정 영역

- `src/renderer/pagination.rs` — WrapAnchorRef 에 `anchor_image_margin_right: i32` 필드 추가
- `src/renderer/typeset.rs` — anchor host paragraph register 시 image margin_right 추출 + register
- `src/renderer/layout/paragraph_layout.rs` — wrap_anchor 처리에서 LINE_SEG cs 에 margin_right 보정

### 검증 절차

1. **결정적 검증** (release 모드): cargo test/clippy
2. **시각 검증** (rsvg-convert PNG)
   - 페이지 8/27/48 inter-image-text gap 정합
   - 다른 페이지 회귀 0

### 정정 산출물

- 영향 코드 변경
- `mydocs/working/task_m100_722_stage6.md` 단계별 보고서

### 승인 요청

Stage 6 정정 결과 + 시각 판정 통과 후 Stage 7 진행 승인.

## Stage 7: 광범위 회귀 sweep (HWP3 native 한정)

페이지 8/27/48 정합 + 209 fixture 페이지 수 차이 0 확인. HWP5 변환본 페이지 16 결함 발견 → Stage 8+ 로 추가 정정.

## Stage 8~9: HWP5 변환본 paragraph 441 시도 — rollback 후 별도 issue 분리

Stage 8 진단 + Stage 9 정정 (`anchor_full_width_match` 가드 추가) 시도했으나 가드가 paragraph 442/443 도 broad 매칭 → 페이지 분할 왜곡 회귀 발생. Stage 9 rollback 후 별도 issue 로 분리.

본 task #722 는 hwp3-sample5.hwp (HWP3 native) 영역 (페이지 8 paragraph 175, 페이지 27 paragraph 779, 페이지 48 paragraph 1394) 한정으로 종결.

상세: `mydocs/working/task_m100_722_stage8.md`, `task_m100_722_stage9.md` 참조.

---

## (이하 Stage 8 본질 진단 — 별도 issue 분리 후 참조용)

## Stage 8 (참조용): HWP5 변환본 paragraph 441 본질 진단

### 목표

hwp3-sample5-hwp5.hwp 페이지 16 paragraph 441 wrap zone 매칭 실패 본질 식별.

### 진단 결과

- paragraph 440 (anchor host): cs=0, sw=51024 (col_area 전체 폭 인코딩)
- paragraph 441 (wrap text): cs=22800, sw=28224 (wrap zone 정합)
- 매칭: cs/sw 모두 다름 → 매칭 실패 → wrap_anchors 미등록
- 결과: paragraph 441 좁은 폭 분할 (composer 또는 paragraph_layout 의 LINE_SEG sw 사용 결과)

### 정정 방향 (Stage 9)

typeset 매칭 가드 확장:
- anchor host cs=0 + sw=body_w (전체 폭) + 다음 paragraph cs>0 인 경우 → wrap zone 매칭

### 산출물

`mydocs/working/task_m100_722_stage8.md`

## Stage 9: HWP5 변환본 매칭 가드 정정

### 목표

`src/renderer/typeset.rs` 의 wrap_around state machine 매칭 분기에 HWP5 변환본 case 가드 추가.

### 검증

- 페이지 16 hwp3-sample5-hwp5.hwp 정합
- 페이지 8/27/48 hwp3-sample5.hwp 정합 보존
- 광범위 sweep 회귀 0

### 산출물

`mydocs/working/task_m100_722_stage9.md`

## Stage 10: 광범위 회귀 sweep + 최종 검증

### 목표

광범위 페이지네이션 회귀 sweep + 시각 판정 게이트웨이 통과 (HWP3 native 페이지 8/27/48 + HWP5 변환본 페이지 16 + 다른 페이지 회귀 0).

### 검증 절차

1. **광범위 페이지네이션 회귀 sweep**
   - 187+ fixture BEFORE/AFTER 페이지 수 차이 0
2. **결정적 검증 (release 모드)**
   - cargo test --lib --release 1165+ passed
   - cargo test --release 전체 GREEN
   - cargo clippy --release 신규 경고 0
3. **시각 판정 게이트웨이 (작업지시자)**
   - `samples/hwp3-sample5.hwp` 페이지 8 시각 판정
   - 다른 페이지 회귀 0 (특히 Task #604 정정 영역)

### 산출물

- `mydocs/report/task_m100_722_report.md` 최종 보고서
- `mydocs/orders/20260508.md` Task #722 상태 갱신

### 승인 요청

최종 보고서 + 시각 판정 통과 후 fork push + PR 생성 승인.

## 회귀 위험 영역 좁힘 원칙

- 수정 영역 명시 — 본질 위치 단일 분기 또는 함수
- 케이스 가드 — 본 case 발현 영역만 정정
- 광범위 sweep 검증 — 187 fixture 페이지 수 차이 0
- Task #604 영역 보존 — HWP3 파서 정정 시 기존 정합 영역 회귀 0

## 의존성

- 선행 의존: 없음 (devel 분기)
- 후행 의존: 없음

## 최종 결과 영역

본 task 완료 후:
- Issue #722 close (closes #722 키워드)
- `samples/hwp3-sample5.hwp` 페이지 8 시각 결함 해소
- HWP3 wrap=Square paragraph 첫 줄 정합 영역 강화

## 작업지시자 결정 영역

Stage 1 진단에서 가설 A/B/C 중 정정 위치 결정 후 승인 요청.
