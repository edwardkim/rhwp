# Task #523 Stage 1 — drift 측정 + 발원지 식별

## 1. 측정 방법

`paragraph_layout.rs` 에 `RHWP_DEBUG_T523` 진단 로그 삽입 + 기존 `RHWP_TYPESET_DRIFT` 환경변수 활성화. `samples/exam_science.hwp` page 1 을 export 하여 col 1 (오른쪽 단) 의 pi 별 cur_h 와 HWP first_vpos 를 수집.

## 2. col 1 누적 drift (cur_h vs HWP first_vpos)

| pi | HWP first_vpos (px) | renderer cur_h | drift (cur_h − HWP) |
|----|---|---|---|
| 16 | 119.05 | 119.1 | +0.05 |
| 17 | 172.39 | 166.3 | -6.09 |
| 18 | 303.59 | 291.3 | -12.29 |
| 20 | 394.47 | 372.6 | -21.87 |
| 21 | 415.93 | 388.0 | -27.93 |
| **22** | **560.47** | **668.1** | **+107.64** ← 큰 점프 |
| 25 | 780.43 | 868.7 | +88.27 |
| 26 | 801.89 | 884.1 | +82.21 |
| 27 | 823.36 | 899.4 | +76.04 |
| 29 | 1003.09 | 1069.2 | +66.11 |

**핵심 관찰**: pi=21 → pi=22 사이에 cur_h 는 280.1 px 증가하지만 HWP vpos 는 144.54 px 만 증가 → **단일 paragraph 에서 ~135.6 px 과적**.

## 3. pi=21 분석

`./target/release/rhwp dump -s 0 -p 21` 결과:

```
[0] 그림: bin_id=8, common=11250×10230 (39.7×36.1mm), tac=false
    위치: 가로=단 오프셋, 세로=문단 오프셋, 정렬=Right/Top
    배치: 어울림 (= Square wrap), 글자처럼=false
```

- 그림 크기 = 11250×10230 HU = 150×136.4 px
- **wrap=Square (어울림)**, **vert_rel_to=Para**, **tac=false**

pi=21 의 fmt_total = 149.9 px ≈ HWP vpos jump 144.54 px (정합). 정상적이라면 cur_h 도 ~144 px 만 증가해야 한다.

그러나 cur_h 는 280 px 증가 → 그림 높이 136.4 px 가 별도로 누적됨.

## 4. 코드 경로 — 누적 위치

`src/renderer/typeset.rs`:

- **line 1025/1061/1077**: `st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height }` — paragraph 자체 높이 추가 (정합)
- **line 656-680**: Square wrap Picture 발견 시 `st.wrap_around_pic_bottom_px = body_y + pic_h_px` 등록
- **line 510-513**: wrap zone 종료 시 `st.current_height = current_height.max(wrap_around_pic_bottom_px)` — 그림 하단까지 cur_h 끌어올림

추가로 어딘가에서 그림 높이가 또 한번 누적되어 현재 280 px 증가가 발생하는 것으로 추정 (정확한 라인 확정에는 추가 trace 필요). 후보:
1. Square wrap Picture 의 `wrap_around_pic_bottom_px = current_height + pic_h_px` 산식이 "그림 하단" 이 아니라 "현재 + 그림 높이" 로 해석되어 height_for_fit 와 중복 가산
2. inline Shape/Picture 처리 루프가 paragraph 컨트롤 순회 중 별도로 current_height 에 그림 높이 추가

## 5. col 0 vs col 1 drift 패턴

col 0 (page 1 왼쪽 단) 의 paras 는 drift 가 ±18 px 범위로 작은 편. col 1 의 큰 점프는 pi=21 의 Square wrap 그림이 단일 발원이며, 이후 paras (pi=22~29) 는 그 잔여 drift 를 그대로 이어받음. col 0 의 pi=8, pi=13 등에도 인라인 도형/수식이 있으나 모두 tac=true 인 inline 으로, line_seg lh 에 정상 반영됨.

## 6. 표 pi=30 overflow 계산

- 표 pi=30 HWP vpos = 78952 HU = 1052.69 px
- 표 pi=30 높이 = 8033 HU = 107.1 px
- 표 pi=30 HWP 기준 종료 위치 = 1159.79 px
- column body 높이 = 91136 HU = 1215.13 px
- HWP 좌표상 잔여 = 55.34 px → fit

- renderer cur_h before pi=30 = 1112.6 px (drift +66 from pi=29)
- column avail = 1205.1 px
- renderer 잔여 = 92.5 px → 표 107.1 px **불충족** → page 2 로 push

drift 60 px 만 제거되어도 fit. drift 의 발원은 pi=21 의 Square wrap 그림 처리.

## 7. 처리 방향 제안

본 결함은 단일 분기의 단순 정정이 아니다. Square wrap Picture 의 current_height 누적 로직 (typeset.rs:656-680, 510-513) 을 정합화 해야 하며, 다음 위험을 동반:

- 다른 샘플 (KTX, exam_math 등 Square wrap Picture 사용처) 에 회귀 가능성 큼
- Layout Phase 3 본질 (`#496` family) 와 동일 영역. 해당 Phase 의 종합 검토와 함께 처리하는 편이 안전

**권고**:
- (A) **deferral / Phase 3 통합 처리** — 본 task 를 close-deferred 로 마감, Phase 3 / Layout 리팩터 진행 시 함께 정정. exam_science page 1/2 시각 결함은 Phase 3 완료 시 자연 해소 기대.
- (B) **단발 minimal fix 시도** — typeset.rs:680 (`wrap_around_pic_bottom_px = body_y + pic_h_px`) 산식 검증 후 중복 가산 제거. Stage 2 에서 좁은 분기로 한정하여 회귀 검증 후 채택 여부 결정.

Stage 1 결론: drift 발원지 식별 완료, Stage 2 진행 전 작업지시자 결정 필요 (A vs B).
