# Task #525 최종 보고서 — exam_science 8번 문제 인라인 수식+한글 중첩 정정

## 요약

`layout_wrap_around_paras` 가 비-TAC Picture wrap=Square host paragraph 에 대해 호스트 자기 텍스트를 중복 emit 하던 결함 정정. 영향 코드 경로 두 곳 (`layout.rs:3106 layout_shape_item` + `layout.rs:3534 layout_column_shapes_pass`) 의 호출 제거. Table Square wrap 의 호출 (layout.rs:2555) 은 유지 (Table 호스트 = 빈 텍스트 + 표 구조의 의도된 동작).

이슈 본문 가설 (Shape advance 폭 / ParaShape spacing) 은 부정확. 실제 본질은 **호스트 paragraph 텍스트 중복 emit** — 정상 PageItem::FullParagraph 경로 + wrap_around 경로 두 곳에서 같은 줄을 다른 col_w 정렬로 distinct x 위치에 emit → 시각 중첩.

**상태**: 완료 (회귀 0, 시각 정합 작업지시자 승인).

## 1. 본질 (Stage 1)

### 1-1. 3 호출 재현 — backtrace 진단

`para_index == 37` 임시 로깅 + `RHWP_LAYOUT_DEBUG=1 export-svg samples/exam_science.hwp -p 1` 결과:

| # | col_w | 출처 | 의도 |
|---|-------|------|------|
| 1 | 422.6 | `layout_column_item → layout_partial_paragraph` (PageItem::FullParagraph) | 정상 paragraph 처리 |
| 2 | 233.9 | `layout_column_item → layout_shape_item:3106` (Picture Square wrap) | wrap-around 텍스트 |
| 3 | 233.9 | `build_single_column → layout_column_shapes_pass:3534` (typeset fallback) | wrap-around 텍스트 |

→ 호출 2 와 3 가 호스트 paragraph 의 텍스트를 중복 emit. 호출 1 의 `has_picture_shape_square_wrap` 분기 (paragraph_layout.rs:822, 973-982) 가 이미 `LINE_SEG.cs/sw` 기반으로 그림 옆 (좁은) + 그림 아래 (넓은) 모두 처리하므로 호출 2, 3 모두 redundant.

### 1-2. `layout_wrap_around_paras` 의 호스트 텍스트 처리

`layout.rs:3206 layout_partial_paragraph(table_para, ...)` — Task #295 자가 wrap host 다중 줄 처리. **Table Square wrap 케이스 의도**: 호스트 = 빈 텍스트 + 표 구조 → wrap_around_paras 가 호스트 텍스트도 처리. **Picture Square wrap 케이스 부작용**: 호스트 = 본문 텍스트 → 정상 PageItem 경로와 중복.

### 1-3. ls 별 emit 분포 (pi=37)

| ls | sw (HU) | 호출 1 effective col_w | 호출 2,3 effective col_w | distinct x |
|----|---------|------------------------|--------------------------|-----------|
| 0~5 | 17546 | 233.9 (좁은) | 233.9 (좁은) | 1 위치 (시각 동일) |
| **6~7** | **31692** | **422.6 (전체)** | **233.9 (좁게 압축)** | **2 위치 → 시각 중첩** |

이슈 본문 보고 "1.3~9px 중첩" = 본 4.46~13px 누적 오프셋과 일치.

## 2. 정정 (Stage 2)

A안 + B안 동시 적용 — Picture Square wrap 의 wrap-around 호출 두 곳 모두 제거.

| 파일 | 라인 | 변경 |
|------|------|------|
| `src/renderer/layout.rs` | 3096-3116 (호출 2) | `layout_shape_item` 안의 Picture Square wrap `layout_wrap_around_paras` 호출 + 컨텍스트 if 블록 제거. 대체 주석으로 PageItem::FullParagraph 경로가 동일 처리 수행함을 명시. |
| 동 | 3499-3546 (호출 3) | `layout_column_shapes_pass` 안의 Picture Square wrap `layout_wrap_around_paras` fallback 호출 블록 전체 제거. |

**유지 항목**:
- Table Square wrap 호출 (`layout.rs:2555 layout_table_item` + `layout.rs:2811 layout_partial_table_item`): Table 호스트 = 빈 텍스트 + 표 구조의 의도된 wrap 처리.
- 호출 1 (정상 PageItem::FullParagraph): `has_picture_shape_square_wrap` 분기 (paragraph_layout.rs:822, 973-982) 로 ls[0..5] 좁은 영역 + ls[6..7] 넓은 영역 모두 적절히 처리.

총 변경: 1 파일 (`layout.rs`), +14 / -69 라인 (`35c6c00`).

## 3. 검증 (Stage 3)

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✓ |
| `cargo test --lib --release` | ✓ 1122 passed / 0 failed / 2 ignored |
| `cargo clippy --release --lib -- -D warnings` | ✓ warning 0 |
| `scripts/svg_regression_diff.sh build HEAD~1 HEAD` | ✓ 168 / 170 byte-identical |

**SVG 회귀**:

| 샘플 | 결과 |
|------|------|
| 2010-01-06, aift, exam_eng, exam_kor, exam_math, synam-001 | byte-identical (회귀 0) |
| exam_science | 002 + 001 변경 = 의도된 정정만 |

**pi=37 직접 측정 (사용자 보고 케이스)**:

```
y≈639.41 / 662.35 / 753.55 / 775.01 / 796.48: dup chars (offset<15px) = 0
```

ls[0..7] 모든 줄에서 distinct x 위치 dup 0 — 사용자 보고 핵심 행 (y=775.01, y=796.48) 완전 해소.

**페이지별 dup-instances**:

| 페이지 | BEFORE | AFTER | byte 상태 |
|--------|--------|-------|-----------|
| 002 (pi=37) | 19 | 5 | CHANGED (-74%) |
| 001 | 6 | 6 | CHANGED |
| 003, 004 | 4, 6 | 4, 6 | IDENTICAL |

잔존 5 (p2) 와 6 (p1) 는 본 task 와 다른 본질의 결함 — 별도 task 영역.

**시각 정합**: 작업지시자 승인.

## 4. Stage 1 가설 정정

Stage 1 §4 에서 "광범위 37 페이지 영향" 가설 → 실제로는 exam_science 2 페이지만 byte-changed.

근거:
- Stage 1 의 dup-instances detection (한 글자가 0.1~15px 오프셋의 다중 x 위치 emit) 이 본 결함 외 다른 layout 결함도 매칭 (false positive)
- exam_kor 130 dup, exam_eng 25 dup 등은 본 task 와 다른 본질
- 본 정정으로 byte 변경 = exam_science 2 페이지만

이는 본 task 의 회귀 위험을 크게 감소시킨 긍정적 발견. 작업지시자 시각 검증 부담도 작음 (2 페이지 한정).

**잔존 dup (189 instances) 는 별도 본질 — 향후 별도 task 분리 권고**.

## 5. 영향 범위 확정

| 영향 범위 | Stage 1 가설 | Stage 3 실제 |
|-----------|-------------|-------------|
| byte-changed 페이지 | 37 페이지 (광범위) | **2 페이지** (exam_science_001, _002) |
| dup-instances 감소 | 205 → ~0 예상 | 205 → 189 (-16, 그 중 -14 는 exam_science p2) |
| 본 task 영향 단락 | 모든 비-TAC Picture wrap=Square host | 동일 (단, 다른 샘플은 byte 동일 결과) |

본 task 의 실제 영향은 좁고, 대부분 dup 감소는 사용자 보고 케이스 (exam_science p2 pi=37) 에 집중.

## 6. 위험·실측

| 위험 (구현 계획서 §4) | 결과 |
|----------------------|------|
| typeset 경로 wrap-around 텍스트 미렌더 | 호출 1 (PageItem::FullParagraph) 가 typeset 경로에서도 활성화되어 정상 처리 — 회귀 0 확인. |
| 본 결함 외 페이지 회귀 | 6 샘플 byte-identical 검증 — 회귀 0. |
| 사용자 시각 검증 부담 (37 페이지 예상) | 실제 2 페이지로 축소 — 부담 작음. |
| 호출 1 만으로 그림 옆 텍스트 미렌더 | `has_picture_shape_square_wrap` 분기 정상 동작 — pi=37 ls[0..5] 모두 정확 위치 emit 확인. |

## 7. 메모

- `layout_wrap_around_paras` 자체 (Task #295 자가 wrap host) 는 Table Square wrap 의 의도된 동작 — Picture Square wrap 에는 부적합한 일반화. 본 task 가 이 일반화의 부작용을 정정.
- 호출 출처 4 곳 중 본 task 정정 = 2 곳 (3106, 3534). 유지 = 2 곳 (2555 Table, 2811 Partial Table).
- 잔존 dup-instances (189) — 본 task 와 다른 결함 (예: 같은 글자 자연 등장, 다른 layout 결함). 별도 task 분리.

## 8. 커밋 이력 (local/task525)

| Commit | 단계 | 내용 |
|--------|------|------|
| `d24a896` | 사전 | 수행 계획서 (사전 진단 결과 반영) |
| `ba680bc` | Stage 1 | 진단 — 광범위 layout_wrap_around_paras 중복 호출 확정 |
| `68f109b` | 사전 | 구현 계획서 — A안 (3534 제거) |
| `35c6c00` | Stage 2 | 비-TAC Picture Square wrap 호스트 텍스트 중복 emit 정정 (A+B안) |
| `78af341` | Stage 3 | 회귀 검증 — 168/170 byte-identical |

다음 commit: 본 보고서 + orders 갱신.

## 9. 종료 조건 충족

- [x] 본질 식별 (Stage 1 — 사용자 가설 정정 포함)
- [x] 구현 계획서 승인 (A안 → A+B안 확장)
- [x] 코드 변경 + 빌드 + 단위 테스트 + Clippy 통과 (Stage 2)
- [x] 회귀 검증 통과 (168/170 byte-identical, 의도된 정정 2페이지만) (Stage 3)
- [x] 시각 정합 작업지시자 승인
- [ ] merge (local/task525 → local/devel → devel + push)
- [ ] gh issue close 525
