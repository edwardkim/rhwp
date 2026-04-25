# TAC 표의 ParaShape align 반영 누락

## 증상

`samples/basic/KTX.hwp` 1페이지 2단 구성에서 오른쪽 단의 TAC 표 (pi=31 16x15, pi=32 10x9) 가 단의 좌측 끝에 붙어 렌더링됨. 한컴 PDF 는 단의 우측에 정렬.

다음 패턴의 다른 샘플에서도 동일 증상:
- `samples/aift.hwp` — 18페이지 분량의 align=Center/Right TAC 표가 좌측 붙음
- `samples/biz_plan.hwp` 1페이지 — TAC 표 align=Center 가 좌측 붙음

## 원인

`src/renderer/layout.rs::layout_columns` 의 표 위치 계산 로직 (`tbl_inline_x`):

```rust
} else if !is_tac && tbl_is_square {
    // Square wrap (비-TAC) — Task #295 에서 horz_align 반영 추가
    let x = match t.common.horz_align { ... };
    Some(x)
} else if is_tac {
    // TAC 분기 — ParaShape alignment 반영 누락 ❌
    let leading = ...;
    Some(col_area.x + effective_margin + leading)  // 좌측 강제
}
```

**비-TAC + Square wrap 분기는 `t.common.horz_align` 반영하지만 TAC 분기는 ParaShape `alignment` 무시.**

## 수정

`src/renderer/layout.rs:1991~2013` (Issue #291) — TAC 분기에 ParaShape `alignment` 매치 추가:

```rust
} else if is_tac {
    let leading = ...;
    let base_x = col_area.x + effective_margin + leading;
    let aligned_x = match para_style.map(|s| s.alignment) {
        Some(Alignment::Right) => {
            let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
            let avail_right = col_area.x + col_area.width - margin_right;
            (avail_right - tbl_w).max(base_x)
        }
        Some(Alignment::Center) => {
            let tbl_w = hwpunit_to_px(t.common.width as i32, self.dpi);
            let center = col_area.x + (col_area.width - tbl_w) / 2.0;
            center.max(base_x)
        }
        _ => base_x,
    };
    Some(aligned_x)
}
```

### 핵심 설계

1. **`base_x` 분리**: 기존 leading 계산을 그대로 두고 그 위에 align 분기 추가
2. **`.max(base_x)` 안전장치**: leading 이 있는 경우 align 정렬로 위치가 후퇴하지 않도록 방어
3. **`_ => base_x`**: Justify/Left/Distribute/Split 등 기존 동작 보존

## 검증

| 샘플 | 변경 페이지 | align 분포 | 평가 |
|------|-------------|-----|------|
| KTX.hwp | 1/1 | TAC+Right | 목표 달성 (24.46px 우측 이동) |
| exam_math | 0/20 | - | 무영향 |
| 21_언어 | 0/19 | - | 무영향 |
| aift | 18/74 | TAC+Center/Right | 모두 의도된 개선 |
| biz_plan | 1/6 | TAC+Center | 의도된 개선 |

cargo test --lib: 992 passed / 0 failed.

## 우선순위 정의

본 수정은 **ParaShape `alignment`** 만 고려. `t.common.horz_align` 도 영향을 줄 수 있는 표라면 다음 우선순위 검토 필요:

1. (현재) ParaShape `alignment`
2. (별도 이슈 후보) `t.common.horz_align`

KTX.hwp 의 TAC 표는 `horz_align = 문단(0)` 으로 무지정 → ParaShape 만 고려해도 충분. 두 값이 충돌하는 케이스 발견 시 본 문서 갱신.

## 참조

- 이슈: [#291](https://github.com/edwardkim/rhwp/issues/291)
- 관련 작업: Task #295 (PR #298) — 비-TAC Square wrap 표의 horz_align 반영 추가
- 발견일: 2026-04-25
