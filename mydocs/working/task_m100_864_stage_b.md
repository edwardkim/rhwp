# Task #864 Stage B 정정 후보 평가

**작성일**: 2026-05-13
**브랜치**: `local/task864`

## B.1 정정 후보 식별

Stage A 진단으로 본질 위치 식별: **`src/wmf/converter/svg/mod.rs`** 의 image record handler 들이 좌표 변환 누락.

### 후보 1 (권장): Image 좌표 origin-relative 변환

**변경 위치**:

1. `bit_blt` (line 146+) — `x_dest, y_dest` 를 `point_s_to_absolute_point` 변환
2. `device_independent_bitmap_stretch_blt` (line 296+) — `x_dest, y_dest` 변환
3. `stretch_blt` (line 385+) — `x_dest, y_dest` 변환
4. `stretch_device_independent_bitmap` (line 471+) — `x_dst, y_dst` 변환

**구현 패턴**:
```rust
let p = PointS { x: x_dst, y: y_dst };
let p = self.context_current.point_s_to_absolute_point(&p);
// then pass p.x, p.y to TernaryRasterOperator::new
```

**viewBox 동시 변경**:
- `device_context.rs:280` `as_view_box` 를 Task #860 Stage D 변경 revert → `(0, 0, x.abs(), y.abs())`
- 또는 변경 유지 + auto-expansion 로직이 알아서 origin-relative 공간으로 정합 (이쪽이 안전)

### 후보 2 (비권장): text/polygon 좌표 변환 제거

기존 `point_s_to_absolute_point` 호출 모두 제거. 변경 범위 큼 + 다른 sample 회귀 위험 높음.

## B.2 회귀 위험 평가

### WMF/EMF 사용 sample 식별

각 sample 에 대해 `RHWP_DEBUG_WMF=1 export-svg` 실행 후 WMF_WINDOW_ORG record 추출:

| sample | WMF window records | Window org |
|---|---|---|
| hwp3-sample14.hwp | 26 | (329, 1536), (624, 1872) |
| hwp3-sample14-hwp5.hwp | 13 | 동일 |
| hwp3-sample4.hwp | 13 | **(0, 0), (1, 0), (1, 1)** |
| hwp3-sample4-hwp5.hwp | 13 | 동일 |
| 그 외 sample | 0 | WMF 미사용 |

**핵심 관찰**: hwp3-sample4 는 window origin 이 모두 (≈0, ≈0). 따라서 origin-relative 변환 적용 시:
- raw (x, y) - origin (0, 0) = (x, y) → **변경 없음**

hwp3-sample14 만 origin 이 의미있는 값. 후보 1 의 변경은 **이 sample 에만 시각 영향**.

### EMF 사용 sample

EMF converter (`src/emf/converter/`) 는 별도 코드. 본 task 범위는 WMF only. EMF 영향 없음.

### 회귀 영향 매트릭스

| sample | 변경 영향 | 회귀 위험 |
|---|---|---|
| hwp3-sample14.hwp | image y 좌표 origin-relative 변환 적용 | **양** (한컴 정합) |
| hwp3-sample4.hwp | origin=(0,0) → 변경 없음 | 없음 |
| 기타 WMF 미사용 | 변경 없음 | 없음 |

### 후보 1 의 viewBox 처리 선택

- **선택 A**: `as_view_box` revert → `(0, 0, x, y)`
- **선택 B**: `as_view_box` 유지 + auto-expansion 으로 정합

**선택 A** 가 깔끔. element 좌표가 origin-relative (0~ext_x, 0~ext_y) 가 되므로 viewBox 도 (0, 0, ext_x, ext_y) 가 자연스럽다.

**위험**: Task #860 Stage D 의 `vb_y < origin_y` 확장 로직이 origin-relative 공간에서 어떻게 동작할지 확인 필요. element 가 음수 y 를 가질 수 있다면 (e.g. point.y < origin_y → abs 후 양수) 그대로 동작.

→ **선택 A 채택**, auto-expansion 로직은 그대로 유지 (필요 시 자동 확장).

## B.3 정정 구현 단계

Stage C 에서 실시:

1. `mod.rs` 의 4개 handler 에 image 좌표 변환 적용
2. `device_context.rs` 의 `as_view_box` revert
3. cargo build --release
4. cargo test --release --lib (회귀 0 확인)
5. hwp3-sample14 page 2 SVG → 한컴 PDF 시각 정합 확인
6. hwp3-sample4 모든 페이지 회귀 0 확인 (origin=0,0 이므로 변경 없을 것)
7. 디버그 print 제거

## Stage B 결론

**후보 1** 채택. 회귀 위험 매우 낮음 (hwp3-sample14 외 영향 없음). Stage C 진행.

📋 **Stage B 완료. Stage C 정정 구현 진행합니다.**
