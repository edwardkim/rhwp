# Task #1001 Stage 3 — Fix 적용 보고서

이슈: [#1001](https://github.com/edwardkim/zhwp/issues/1001)
Stage 1/2: [`task_m100_1001_stage1.md`](task_m100_1001_stage1.md), [`task_m100_1001_stage2.md`](task_m100_1001_stage2.md)

## 1. 변경 사항

### 1-1. 파일 / 위치
- `src/renderer/layout.rs:976-1042` `build_page_borders` 메서드

### 1-2. 변경 내용 (후보 A — bit 1/2 처리 추가)

**핵심 추가**:
```rust
let header_inside = (pbf.attr & 0x02) != 0;
let footer_inside = (pbf.attr & 0x04) != 0;

// ... 기존 base/spacing 계산 ...

// bit 1/2 clip — 머/꼬 영역 진입 차단 (paper 기준 회귀 해소).
if !header_inside {
    let header_bottom = layout.body_area.y;
    if by < header_bottom {
        bh -= header_bottom - by;
        by = header_bottom;
    }
}
if !footer_inside {
    let footer_top = layout.body_area.y + layout.body_area.height;
    if by + bh > footer_top {
        bh = footer_top - by;
    }
}
```

**부가 변경**:
- `RHWP_DEBUG_PAGE_BORDER` 출력에 `bit1`, `bit2`, `header_inside`, `footer_inside` 추가 (Task #987 디버그 영속화 정책 정합)
- `(bx, by, bw, bh)` 변수 mutable (clip 적용 위함)

### 1-3. 설계 결정

- **paper / body 양쪽에 clip 적용 (spec 정공법)**:
  - paper 기준: paper 5mm 안쪽 → header/footer 제외 시 body_area 한계
  - body 기준: body ± spacing → header/footer 제외 시 body_area 한계
  - body 기준 + spacing>0 케이스에서 외곽선 spacing 효과 일부 약화 가능 (HWP3 sample16) — Stage 4 시각 검증 대상
- **좌/우는 clip 대상 아님**: spec 표 136 bit 1/2 는 머/꼬 (상/하) 영역만 정의

## 2. 단위 검증

### 2-1. 격차 A 정합 확인 (Primary)

`samples/hwp3-sample16-hwp5.hwp` 페이지 16:

| 항목 | Fix 전 | Fix 후 |
|------|--------|--------|
| 외곽선 하단 y (px) | ~1118 (paper-5mm) | **~1047** (body 하단) |
| 페이지 번호 y (px) | ~1079 | ~1079 |
| 관계 | 1079 < 1118 (외곽선 안) ✗ | **1079 > 1047 (외곽선 밖)** ✓ |

**Debug 출력**:
```
PAGE_BORDER: attr=0x00000001 bit0=1 bit1=0 bit2=0 paper_based=true 
  header_inside=false footer_inside=false bfid=2 spacing(L=1420,R=1420,T=1420,B=1420)
```

### 2-2. HWP3 원본 비교

`samples/hwp3-sample16.hwp` 페이지 16:

| 항목 | Fix 전 | Fix 후 |
|------|--------|--------|
| 외곽선 하단 y (px) | ~1047 (body+5mm spacing) | **~1047** (clip 미발동) |

본래 body 기준 + spacing 5mm 외부로 외곽선이 body_area 범위에 거의 일치하던 영역. clip 적용해도 변동 미미 (body_area 한계 = 같은 값 부근).

### 2-3. cargo test / clippy

- `cargo test --release --lib`: **1306 passed**, 0 failed (baseline 유지)
- `cargo clippy --release -- -D warnings`: 0 warnings

## 3. 잔존 검증 / 회귀 Risk (Stage 4 위임)

- **시험지 (`exam_kor/math/eng`, paper-based + spacing=0)** 시각 회귀 확인 필요
- **aift / biz_plan / 통합재정통계 등** paper-based + spacing=1417 sample 확인 필요
- HWP3 sample16 외곽선 위치 미세 변동 시각 판정

## 4. 격차 B (페이지 분할 drift)

본 Task 범위 외. Stage 5 보고서에서 후속 issue 분리 권고 명시 예정.

## 5. Stage 4 진입 계획

- SVG sweep (시험지 + 일반 paper-based + body-based sample) 회귀 측정
- WASM 빌드 검증
- rhwp-studio 실제 렌더링 확인
