# Task #864 최종 결과 보고서

**이슈**: https://github.com/edwardkim/rhwp/issues/864
**브랜치**: `local/task864`
**작성일**: 2026-05-13
**제목**: HWP3 sample14 WMF metafile + 그림 caption 위치 한컴 정합

## 1. 본질 (3 가지)

### 본질 1 — WMF metafile element y 좌표 비일관 (Stage A-C)

WMF metafile 의 element 좌표 변환이 비일관:
- text/polygon/rect: `point_s_to_absolute_point` 로 origin-relative (device) 좌표 변환
- image (TernaryRasterOperator): 변환 없이 raw logical 좌표 그대로 사용

→ HWP3 sample14 page 2 의 캡션 outline + text 가 BMP 위에 그려짐 (한컴과 반대).

### 본질 2 — HWP3 inline picture caption 위치 결함 (Stage D-E)

`src/renderer/layout.rs:2984` 에서 inline picture 의 caption y 계산이 `pic.common.height` 만 사용. 하지만 paragraph_layout 가 image 를 baseline-aligned 위치 (= `pic_y + baseline - pic_h`) 에 emit 하므로, caption 도 `pic_y + baseline` (= image bottom) 에 위치해야 함.

→ HWP3 sample14 page 3 의 "Cut&Paste 할 영역" caption 이 image 안에 가려짐.

### 본질 3 — inline TAC picture 중복 emit (Stage G)

`paragraph_layout` 와 `layout.rs` 의 TAC inline branch 가 동일 picture 를 두 곳에서 emit. `layout.rs` 의 `already_registered` 체크가 paragraph_layout 의 picture 등록 누락으로 무력화 → top-aligned + baseline-aligned 두 image 가 동시 출력.

→ caption 이 두 image 사이 또는 안쪽에 가려짐. body 와의 간격도 비정상.

추가로 `result_y` 가 caption 높이를 무시 → 다음 paragraph 가 caption 위에 겹침 (Stage F).

## 2. 정정 내용

### Stage A-C: WMF y 좌표 정정

1. `src/wmf/converter/svg/mod.rs` — 5개 image record handler 에 `point_s_to_absolute_point` 변환 적용:
   - `bit_blt`, `device_independent_bitmap_bit_blt`, `device_independent_bitmap_stretch_blt`, `stretch_blt`, `stretch_device_independent_bitmap`
   - `element_max_y` 함수: text element 의 `font-size` 기반 viewBox 자동 확장 추가

2. `src/wmf/converter/svg/device_context.rs` — `as_view_box` revert: `(0, 0, x.abs(), y.abs())` (Task #860 Stage D 미봉책 정정).

### Stage D-E: HWP3 picture caption 위치 정정

3. `src/renderer/layout.rs` — inline picture caption y 를 `pic_y + max(baseline, pic_h)` 로 정정. paragraph_layout 의 baseline-aligned image 와 정합.

4. `src/main.rs` — dump 출력에 picture caption 정보 (방향/너비/문단수/텍스트) 표시 추가.

### Stage F: caption 후 result_y 진행

5. `src/renderer/layout.rs` — caption (Bottom direction) 렌더 후 `result_y = max(result_y, cap_y + caption_h)` 진행. 다음 paragraph 가 caption 위에 그려지던 결함 정정.

### Stage G: inline TAC picture 중복 emit 정정

6. `src/renderer/layout/paragraph_layout.rs` — inline TAC picture emit 시 `tree.set_inline_shape_position` 호출. layout.rs 의 `already_registered` 체크 통과 → 중복 emit 방지.

## 3. 검증 결과

| 검증 항목 | 결과 |
|---|---|
| cargo build --release | ✓ |
| cargo test --release --lib (1230 passed) | ✓ 회귀 0 |
| cargo clippy --release --lib | ✓ 경고 0 |
| hwp3-sample14 page 2 한컴 PDF 정합 | ✓ image 위, 캡션 아래 |
| hwp3-sample14 page 3 한컴 PDF 정합 | ✓ "Cut&Paste 할 영역" caption 정상 + body 자연 간격 |
| hwp3-sample14 page 4 한컴 PDF 정합 | ✓ "Visual Block을 이용한 대소문자 변경" caption 정상 + body 자연 간격 |
| hwp3-sample14 전체 11 페이지 회귀 | ✓ 0 |
| hwp3-sample14-hwp5 동등 | ✓ |
| hwp3-sample4 회귀 (36 페이지) | ✓ 0 |

### 중복 image emit 정정 확인

| 페이지 | 변경 전 outer `<image>` | 변경 후 |
|---|---|---|
| page 3 | 4 (= 2 pic × 2 emit) | 2 |
| page 4 | 4 | 2 |

## 4. 영향도

| sample | Stage A-C | Stage D-G |
|---|---|---|
| hwp3-sample14.hwp page 2 | **시각 정합** | 영향 없음 |
| hwp3-sample14.hwp page 3 | 영향 없음 | **시각 정합** |
| hwp3-sample14.hwp page 4 | 영향 없음 | **시각 정합** |
| hwp3-sample14-hwp5.hwp | 동일 | 동일 |
| hwp3-sample4.hwp | 무영향 | 무영향 (caption 빈) |
| 기타 sample | 무영향 (WMF 미사용) | inline image 의 SVG 출력에서 중복 제거 (시각 동등) |

EMF converter 는 본 task 범위 외.

## 5. 단계별 보고서

- Stage A: `mydocs/working/task_m100_864_stage_a.md` (WMF 본질 진단)
- Stage B: `mydocs/working/task_m100_864_stage_b.md` (WMF 정정 평가)
- Stage C: `mydocs/working/task_m100_864_stage_c.md` (WMF 정정 구현)
- Stage D: `mydocs/working/task_m100_864_stage_d.md` (HWP3 caption 진단)
- Stage E: `mydocs/working/task_m100_864_stage_e.md` (caption 위치 정정)
- Stage F: `mydocs/working/task_m100_864_stage_f.md` (caption 후 result_y)
- Stage G: `mydocs/working/task_m100_864_stage_g.md` (중복 image emit 정정)

## 6. 커밋 메시지 (안)

```
Task #864: WMF image y 좌표 + HWP3 picture caption + 중복 emit 정정

본질 1 (WMF): 5개 image record handler 의 image x/y 를 point_s_to_absolute_point
로 변환. text/polygon 과 동일 device 공간 정합. as_view_box 를 (0, 0, ext_x, ext_y)
로 revert (Task #860 Stage D 미봉책 정정).

본질 2 (HWP3 caption): inline (TAC) picture caption 의 y_start 를 image_bottom
(pic_y + max(baseline, pic_h)) 에 정합. paragraph_layout 의 baseline-aligned
image 와 정합 (HWP3 sample14 page 3 "Cut&Paste 할 영역", page 4 "Visual Block
을 이용한 대소문자 변경" caption 가 image 에 가려지던 결함).

본질 3 (중복 emit): paragraph_layout 의 inline TAC picture emit 시
set_inline_shape_position 호출 추가. layout.rs 의 already_registered 체크
통과시켜 중복 emit (top-aligned + baseline-aligned 동시 출력) 방지.

추가: caption 렌더 후 result_y 를 caption bottom 까지 진행하여 다음 paragraph
와의 겹침 방지.

closes #864
```

## 7. 결론

본 task 는 세 본질을 모두 정정하여 HWP3 sample14 의 page 2, 3, 4 모두 한컴 PDF 와 시각 정합. 중복 image emit 결함도 함께 정정하여 모든 inline TAC picture 의 SVG 출력 정합. 1230 테스트 회귀 0, clippy 경고 0.

📋 **Task #864 최종 결과 보고서 — 커밋 + 이슈 클로즈 승인 요청드립니다.**
