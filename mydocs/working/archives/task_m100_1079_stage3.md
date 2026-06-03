# Stage 3 보고서 — Task #1079: 구현 (typeset + 렌더 gap-fill)

- 브랜치: `local/task1079`
- 수정: `typeset.rs`, `layout.rs`, `layout/picture_footnote.rs`

## 판정 조건 (양 레이어 공통)
```
gap_before = V[pi] - (V[pi-1] + line_height[pi-1])      // 그림 para 줄 앞 빈 공간
already_accounted = para_idx>0 && gap_before >= 그림높이 - 8px
```
- pr-149 pi=2: gap≈217 ≥ 209 → true. #409 계열(gap 작음) → false(현행 유지).

## 구현
1. **typeset** (`typeset.rs:1320~`): pushdown_h match 를 `(obj_h, extra)` 로 변경, already_accounted
   시 `current_height += extra` 생략. → pr-149 pi=7 cur_h 986.5 → 777.2px(파일 vpos), 1페이지.
2. **렌더** (`picture_footnote.rs:layout_body_picture`): `vpos_accounts_for_height` 인자 추가.
   - true 시 그림을 gap 안에: `pic_y = base_y + caption_top_offset - total_height` (그림 바닥이
     그림 para 줄 base_y 에 정렬).
   - 반환: true 시 `base_y`(추가 진행 없음), false 시 현행 `base_y + total_height`.
   - 호출부(`layout.rs` layout_shape_item)에서 동일 gap_before 산출 후 전달. paper-based 호출은
     false.

## 검증 (pr-149)
- **1페이지, overflow 0.** 렌더 위치(절대 px): 원본:164.9 / 그림1 174.9~384 / 회색조:416.9 /
  그림2 426.9~636 / 흑백:668.8 / 그림3 678.8~888 / 입니다:942.1. 모두 본문(1009) 내.
- 라벨→그림→라벨 순서 정확, 입니다. 존재(누락 없음), 그림이 라벨 아래 — **PDF p1 구조 정합**.

## 회귀 (전수 sweep)
- baseline 3057 lines / 382815px / 97파일 → **3056 / 382705 / 96파일**.
- 변화: **pr-149 만 overflow 소멸(해소)**. 신규 회귀 0, 타 파일 불변. #409 계열(gap<그림높이)
  현행 유지 확인.
- 골든 SVG **8/8**, lib **1324 passed**, clippy/fmt clean.

## 다음 (Stage 4)
회귀 가드(`tests/issue_1079_*.rs`: pr-149 1페이지 + 그림 라벨 아래 + 입니다 본문 내) + 최종 보고서.
