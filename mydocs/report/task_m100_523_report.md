# Task #523 최종 결과보고서 — exam_science p1→p2 누적 vpos drift (deferred)

GitHub Issue: [#523](https://github.com/edwardkim/rhwp/issues/523)
브랜치: `local/task523`
상태: **Stage 1 진단 완료 → deferral (Layout Refactor Phase 3 통합 처리)**

## 1. 결함 요약

`samples/exam_science.hwp` rhwp SVG 출력에서 문제 6번 `<보기>` 박스 + 답안 ① ② ③ ④ ⑤ 가 page 1 우측 단 하단에 들어갈 자리가 있음에도 page 2 로 밀려나, page 2 가 거의 비어있는 채로 출력된다. PDF (한컴) 에선 같은 콘텐츠가 page 1 안에 정상 수용.

## 2. 진단 결과 (Stage 1)

`RHWP_TYPESET_DRIFT` + 추가 진단 로그로 page 1 col 1 (오른쪽 단) 의 pi 별 cur_h vs HWP first_vpos 를 측정.

### Drift 추이

| pi | drift (cur_h − HWP first_vpos) |
|----|---|
| 16 | +0.05 |
| 17~21 | -6 ~ -28 (under-count) |
| **22** | **+107.6 (단일 paragraph 에서 ~135 px 점프)** |
| 25~29 | +66 ~ +88 |

표 pi=30 (`<보기>`) 직전 cur_h=1112.6 px, avail=1205.1 px, 잔여 92.5 px → 표 107.1 px 미달 → page 2 로 push. drift 60 px 만 제거되어도 fit.

### 발원

pi=21 (문제 5 본문) 의 **wrap=Square** 그림 (39.7×36.1 mm = 136.4 px 높이) 가 typeset 의 `current_height` 누적 시 paragraph 자체 높이 (height_for_fit ≈ 144 px) 와 별도로 한 번 더 가산됨. 코드 경로:

- `src/renderer/typeset.rs:656-680` — Square wrap Picture 발견 시 `wrap_around_pic_bottom_px` 등록
- `src/renderer/typeset.rs:510-513` — wrap zone 종료 시 cur_h max 보정

cur_h 가 ~135 px 추가 누적되어 후속 paragraphs 가 모두 그 만큼 아래로 밀리고, page 1 잔여 공간을 침식.

## 3. 영향 범위

- exam_science 페이지 1 → 2 의 problem 6 `<보기>` overflow.
- 다른 샘플 (KTX, exam_math 등 Square wrap Picture 사용처) 도 동일 family 의 drift 가능성. 단 본 task 범위 외.

## 4. 처리 방향 — deferral 결정

작업지시자 결정: **(A) Phase 3 통합 처리**.

### 결정 근거

- Square wrap Picture 의 cur_h 누적 로직은 단발 분기 정정만으로 안전하게 재현 회피 어려움.
- KTX / exam_math / synam-001 등 다수 샘플이 동일 코드 경로를 통과 → 회귀 위험 큼.
- `#496` Layout Phase 3 (누적 baseline / vpos drift) 본질의 일부. Phase 3 종합 검토 시 함께 정정하는 편이 안전.
- exam_science page 1/2 시각 결함은 Phase 3 완료 시 자연 해소 기대.

### 향후 처리 가이드

Phase 3 진행 시 본 task 의 진단 데이터 (Stage 1 보고서) 를 입력으로 사용:

- `typeset.rs:656-680` 의 `wrap_around_pic_bottom_px = body_y + pic_h_px` 산식이 paragraph 의 line_seg lh 와 중복 가산되는지 검증
- HWP 가 Square wrap 의 그림 높이를 paragraph vpos 에 반영하는 사례 vs 별도 vpos jump 로 인코딩하는 사례 구분
- 두 경우를 분기하여 처리

## 5. 산출물

- `mydocs/plans/task_m100_523.md` — 수행계획서
- `mydocs/plans/task_m100_523_impl.md` — 구현계획서
- `mydocs/working/task_m100_523_stage1.md` — 진단 + 발원지 식별
- 본 보고서

## 6. 코드 변경

없음. Stage 1 진단 로그는 본 보고서 작성 후 제거. `git diff --stat src/` 결과 clean.

## 7. 결론

본 결함은 layout drift family (#496 / Phase 3) 의 일부로, 단발 정정 보다 Phase 3 종합 검토에서 처리하는 것이 안전하다. Stage 1 진단 데이터로 발원지(typeset.rs Square wrap 누적)를 명시적으로 식별 — Phase 3 진행 시 입력으로 활용 가능.
