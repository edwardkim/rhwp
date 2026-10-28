# #7015 — HWP5 그림 자르기: 한 축만 진짜 자르기일 때 적응 배율이 틀린다

## 무엇

`30442_[권고문] 민원현장 발굴 제도개선 방안 (Ⅰ) - 영업부담 완화.hwp`(권익위, 17쪽)
**3쪽** 하단의 국민권익위원회 로고가 일부만 그려진다. CLI 렌더와 studio 가 같고,
`layout-anomaly` 는 이 쪽에 아무 신호도 내지 않는다 — 기하가 아니라 그림 자르기
축이기 때문이다.

## 왜

`compute_image_crop_src`(`src/renderer/svg.rs`)는 기준 크기(`imgDim`)가 있으면 그것을
쓰고, 없으면 crop `right`/`bottom` 이 **전체 좌표 범위**라고 보는 적응 폴백을 쓴다
(`#3239`). 그런데 `Picture::img_dim` 은 HWPX 파서만 적재하므로 **HWP5 는 항상 폴백**
이다.

그 가정은 **그 축을 자르지 않았을 때만** 성립한다. `left > 0` 이면 `right` 는,
`top > 0` 이면 `bottom` 은 자르기 경계일 뿐 전체 범위가 아니다. 종전 코드는 축을
나누지 않고 두 값을 늘 전체 범위로 썼다.

이 문서의 원값(`HWPTAG_SHAPE_COMPONENT_PICTURE`)과 디코딩 크기:

```text
  crop  left=0  top=20745  right=88560  bottom=45453      이미지 1181 × 945 px

  x 축  left = 0  → right 가 전체 범위    88560 / 1181 = 75.0   (표준 7200/96 과 일치)
  y 축  top  > 0  → bottom 은 자르기 경계   45453 /  945 = 48.1   ← 36% 작다
```

y 배율이 작아지면 자르기 창이 아래로 밀리고 길어진다.

```text
  결함  src_y = 20745 / 48.1 = 431.3 · src_h = (45453−20745) / 48.1 = 513.7
  정정  src_y = 20745 / 75.0 = 276.6 · src_h = (45453−20745) / 75.0 = 329.5
```

## 실측 — 원본 이미지의 로고 잉크 위치

BinData 의 JPEG(1181×945)에서 밝기 235 미만 화소의 범위:

```text
  잉크 행   y 324 .. 562
  잉크 열   x  78 .. 1108
```

```text
  결함 viewBox  y 431.3 .. 945.0   위 107행이 잘리고 아래 383행은 흰 여백
  수정 viewBox  y 276.6 .. 606.1   로고를 온전히 감싼다
```

## 어떻게

적응 폴백을 축별로 판정한다. 한 축만 전체 범위가 확인되면 그 배율을 두 축에
쓴다 — HWP5 crop 좌표는 등방이다. 둘 다 없으면 [Task #477] 표준 75 HU/px 로 떨어진다.

```rust
let axis_scale = |crop_start: i32, crop_end: i32, img_px: f64| {
    (crop_start == 0 && crop_end > 0 && img_px > 0.0)
        .then(|| crop_end as f64 / img_px)
        .filter(|scale| scale.is_finite() && *scale > 0.0)
};
match (axis_scale(cl, cr, img_w_px), axis_scale(ct, cb, img_h_px)) {
    (Some(sx), Some(sy)) => Some((sx, sy)),
    (Some(scale), None) | (None, Some(scale)) => Some((scale, scale)),
    (None, None) => None,
}
```

새 상수·새 문턱을 만들지 않는다.

## `#3239` 와의 관계

폴백을 도입한 그 픽스처(`samples/issue3239/evaluation_form_200dpi_scan.hwp`)의 원값은
`crop (0, 0, 59520, 84240)` 으로 **자르기가 없다**(전체 범위를 기록). `left = top = 0`
이라 새 판정에서도 두 축 모두 적응 배율을 쓴다 — 200dpi 스캔의 36 HU/px 가 그대로
살아 있다.

## 검증

- `compute_image_crop_src` 단위 시험 8개 중 **7개 그대로 통과**.
- 바뀌는 것은 `test_compute_image_crop_src_offset_top_left` 하나. 오라클 핀이 아니라
  폴백 동작을 기술한 **합성** 시험이고, 그 전제가 자기 설정과 모순이었다 — 주석은
  "right/bottom 이 전체 좌표 범위"라면서 설정은 `left = 1000` · `top = 500` 이다.
  두 축 다 잘렸으므로 전체 범위를 확인할 축이 없고 표준 75 HU/px 로 떨어지는 것이
  새 규약이다. 근거를 적어 갱신했다.
- 회귀 핀 `tests/cases/issue_7015_hwp5_single_axis_crop_scale.rs` — 수정 전 FAILED ·
  수정 후 ok 양쪽 실증.
- source-side 시험 총량 불변(tier 래칫 4205) — `pub(crate)` 함수의 단위 시험 대신
  공개 렌더 경로(`render_page_svg_native`)로 잠갔다.
