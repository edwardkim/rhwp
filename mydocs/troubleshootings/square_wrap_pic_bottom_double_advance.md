---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# Square wrap 그림 하단 보정의 double advance 함정

## 한 줄 요약

Square wrap 그림이 있는 paragraph 의 wrap zone 종료 시 `current_height` 를 단순히 그림 하단으로 advance 하면, **wrap-around paragraph 의 누적 height 와 결합한 double advance** 발생. exam_science.hwp 같은 specific 조합 (2단 + 그림이 단 0 끝 + 풍부한 wrap-around paragraph) 에서 페이지/단 강제 분리 회귀.

## 발생 사례

| 사례 | 결함 |
|------|------|
| **Task #460 보완5 (`82e41ba`)** | HWP3 Square wrap 그림 아래 텍스트가 그림과 겹치는 결함 정정 시도 → exam_science.hwp 2페이지 페이지네이션 회귀 (4 페이지 → 6 페이지, p2 본문 37 items → 2 items) |
| Task #546 (revert) | `82e41ba` 전체 revert 로 정정 — Task #460 보완5 의 본 의도 손실, HWP3 fixture 시각 결함 재발 가능성은 현재 환경에서 미검출 |

## 결함의 본질

### 정상 케이스 (간단한 wrap)

```
wrap zone 진입 (current_height = X)
  paragraph_anchor (Square wrap 그림 보유)
  paragraph_옆텍스트 (그림 옆 짧게 흘러감)
wrap zone 종료
  current_height = max(current_height, 그림 하단 y)  ← wrap-around 가 그림 옆을 다 못 갔으므로 그림 아래로 advance 정합
후속 paragraph (그림 아래 정합 출력)
```

### 결함 트리거 케이스 (exam_science 같은 풍부한 wrap)

```
wrap zone 진입 (current_height = 531.69 px)
  paragraph_anchor (그림 39.7×36.1mm)
  paragraph_옆1 ~ 옆N (wrap-around, current_height 누적)
  ← 이 시점에 wrap-around 이미 그림 옆을 다 통과 + 그림 하단 도달
wrap zone 종료
  current_height = max(current_height, 668.09)
  ← double advance: wrap-around 가 이미 그림 아래까지 도달했는데도 추가로
    그림 하단 y(=668.09) 까지 advance → 자체로는 무해해 보이지만…
후속 paragraph 들이 큰 current_height 로 시작
  → 단/페이지 끝 강제 도달 → 페이지 분리 발생
```

## 진단 방법

### 1. Bisect 로 회귀 origin 식별

```bash
git bisect start
git bisect bad <PR 머지 후>
git bisect good <PR 직전>
# 각 단계: cargo build --release --bin rhwp + dump-pages 의 페이지 수 비교
```

### 2. dump-pages 로 페이지 단위 영향 점검

```bash
rhwp dump-pages samples/exam_science.hwp -p 1
# 단 0 (items=2, used=132.7px) ← 회귀 시 본문 누락
# 단 0 (items=37, used=1133.6px) ← 정상
```

### 3. Square wrap 그림 위치 식별

```bash
rhwp dump samples/exam_science.hwp -s 0 | grep "배치: 어울림" -B 30 | grep "문단 0\."
# 본 결함이 trigger 되는 paragraph 식별
```

## 옵션 C (col 경계 검사) 가 효과 없는 이유

직관적 가드: `bottom_px <= col_h` 일 때만 보정 적용.

진단 결과 (exam_science.hwp):

| Square wrap | bottom_px | col_h | 가드 |
|-------------|-----------|-------|------|
| pi=21 | 668.09 | 1215.15 | 통과 |
| pi=37 | 752.05 | 1215.15 | 통과 |
| pi=60 | 1052.87 | 1215.15 | 통과 |

→ 보정값 자체는 col 영역 내. **결함은 col 경계가 아니라 wrap-around paragraph 누적 height 와의 결합 영역**. col 경계 검사로는 검출 불가.

## 정합한 정정 방향 (향후 재시도 시)

옵션 C 가 효과 없음 → 정정은 다음 영역 중 하나에서 시도:

### 옵션 1 — wrap-around paragraph 누적 height 추적

wrap zone 동안 wrap-around paragraph 들의 누적 height 를 추적. wrap zone 종료 시:
```
if (wrap-around 누적 height < 그림 높이) {
    // 아직 그림 옆에 머물러 있음 → 그림 아래로 advance 필요
    current_height = max(current_height, 그림 하단 y);
} else {
    // 이미 그림 아래로 흘러감 → 추가 advance 불필요
    // (보정 skip)
}
```

### 옵션 2 — 그림 본질을 layout/pagination 에서 분리 처리

현재 typeset 단계에서 `current_height` 를 직접 manipulation. 대신 layout 단계에서 그림 영역 자체를 reserved zone 으로 처리하고 후속 paragraph 의 y_offset 만 보정 (typeset 의 current_height 는 자연 누적 그대로 유지).

### 옵션 3 — case-specific 가드 (HWP3 만 적용)

본 의도가 HWP3 fixture 만이므로 HWP3 (또는 paper-relative) 분기에서만 보정 적용:
```rust
if matches!(cm.vert_rel_to, VertRelTo::Para) && is_hwp5 {
    // HWP5/HWPX 의 Para-relative 는 wrap-around 흘러감으로 자연 처리
    // 보정 적용 안 함 → exam_science 회귀 회피
} else {
    // HWP3 또는 paper/page-relative
    st.wrap_around_pic_bottom_px = body_y + pic_h_px;
}
```

## 권위 메모리 정합

- **`feedback_v076_regression_origin`**: bisect 로 회귀 origin 정확히 식별 (단일 commit `82e41ba` 까지 좁힘)
- **`feedback_visual_regression_grows`**: 광범위 fixture sweep + 작업지시자 시각 판정 게이트
- **`feedback_hancom_compat_specific_over_general`**: 일반화 가드 (옵션 C col 경계) 가 효과 없을 때 case-specific revert 가 정합

## 광범위 영향 점검 시 잡지 못할 수 있는 specific 조합

본 결함이 PR #506 의 광범위 회귀 검증에서 미검출된 이유:

| 조합 요소 | 영향 |
|----------|------|
| Square wrap 그림 부재 fixture | typeset 분기 trigger 안 됨 |
| Square wrap 있어도 wrap-around 흘러감이 짧은 케이스 | bottom_px 와 current_height 차이 작음 → 영향 작음 |
| **2단 + 그림이 단 0 끝 + 풍부한 wrap-around (= exam_science)** | **결함 trigger** |

→ Square wrap + 다단 layout fixture sweep 을 회귀 검증 게이트에 추가 필요. 권장 fixture: `samples/exam_science.hwp`.

## 관련 task / PR / 이슈

- **PR #506** (HWP 3.0 파서 + Square wrap 어울림 렌더링, @jangster77 51 commits) — 회귀 origin commit 포함
- **Task #460 보완5** (`82e41ba`) — 회귀 origin commit
- **Task #546** — 본 결함 정정 (옵션 A revert)
- **이슈 #546** — 회귀 추적 이슈 (closes Task #546 정정으로)

## 권장 향후 절차

1. HWP3 fixture 의 Square wrap 그림 아래 텍스트 시각 결함 재발 점검 (`rhwp export-svg` + 한컴 2010/2020 비교)
2. 재발 시 별도 task 등록 → 옵션 1/2/3 중 결정
3. 회귀 검증 게이트에 exam_science.hwp 의 페이지 수 검사 추가 (`tests/issue_546.rs` 참고)
