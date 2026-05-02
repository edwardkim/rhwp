# Task #533 Stage 4 — 광범위 샘플 회귀 검증

**작성일**: 2026-05-02
**이슈**: [#533](https://github.com/edwardkim/rhwp/issues/533)
**브랜치**: `local/task533`

## 1. 결론

> 8 샘플 192 페이지 중 **190 페이지 byte-identical**, **2 페이지 변경** (exam_kor p14 + p17). 변경 페이지 모두 본 결함의 동일 본질 (Square wrap host text > table height) 케이스로, 의도된 정정. 회귀 0 건.

## 2. 측정 명령

```bash
scripts/svg_regression_diff.sh build b848d00 61415cf \
    exam_kor exam_eng exam_science exam_math \
    synam-001 aift 2010-01-06 exam_math_no
```

`b848d00`: Stage 2 (변경 전) / `61415cf`: Stage 3 (변경 후).

## 3. 결과

| 샘플 | total | same | diff | 변경 페이지 |
|------|-------|------|------|------------|
| 2010-01-06 | 6 | 6 | 0 | — |
| aift | 77 | 77 | 0 | — |
| exam_eng | 8 | 8 | 0 | — |
| **exam_kor** | **20** | **18** | **2** | p14, p17 |
| exam_math_no | 20 | 20 | 0 | — |
| exam_math | 20 | 20 | 0 | — |
| exam_science | 6 | 6 | 0 | — |
| synam-001 | 35 | 35 | 0 | — |
| **합계** | **192** | **190** | **2** | |

## 4. 변경 페이지 분석

### 4-1. exam_kor p14 (의도된 정정 — 본 task 의 직접 대상)

본 결함의 1차 발견 페이지. dump-pages 측정:

| pi | wrap=Square host | inner_h | table_h | host > table | 결함 |
|----|-----------------|---------|---------|--------------|------|
| 33 (col 0) | 4 lines × 1838 + lh = 6664 HU = 88.85 px | 88.85 px | 75.6 px | +13.25 px | ✓ shift |
| 37 (col 0) | 3 lines + lh = 4826 HU = 64.35 px | 64.35 | 51.4 px | +12.95 px | ✓ shift |
| 40 (col 0) | 2 lines + lh = 2988 HU = 39.84 px | 39.84 | 26.4 px | +13.44 px | ✓ shift |
| 47 (col 0) | 3 lines + lh = 4826 HU = 64.35 px | 64.35 | 51.4 px | +12.95 px | ✓ shift |
| 50 (col 1) | 3 lines + lh = 4826 HU = 64.35 px | 64.35 | 51.4 px | +12.95 px | ✓ shift |

→ 좌측/우측 단 모두 5 곳의 Square wrap host 가 모두 12-13 px 만큼 호스트 텍스트가 표보다 길어 본 fix 의 max() 분기에 진입.

### 4-2. exam_kor p17 (의도된 정정 — 동일 본질 추가 발견)

dump-pages 측정:

```
PartialParagraph  pi=2  lines=0..9  vpos=19715..34419
Table          pi=2 ci=0  3x2  23.0x198.8px  wrap=Square tac=false  vpos=19715..34419
FullParagraph  pi=3  vpos=36257..49123  "사잇소리 표기에서는, 󰡔용비어천가󰡕는 ..."
```

| 항목 | 값 |
|------|-----|
| pi=2 inner_h | (34419 + 1150) - 19715 = 15854 HU = 211.39 px |
| pi=2 table height | 198.8 px |
| host > table | 211.39 - 198.8 = **12.59 px** ★ |

→ pi=3 이하 12.59 px 만큼 아래로 이동. SVG 측정 검증:

| 텍스트 | y (수정 전) | y (수정 후) | shift |
|--------|-----------|-----------|-------|
| "사" (pi=3 첫 글자) | 680.41 | 692.81 | **+12.40** ✓ |
| body clip rect y | 1059.48 | 1071.88 | +12.40 |
| 외곽 rect height | 721.56 | 733.96 | +12.40 |

본 결함과 동일 패턴, 동일 정정 효과. **별도 결함 아님 — 본 task 흡수**.

## 5. 회귀 차단 검증

### 5-1. 동작 동일 케이스 (이론 + 측정)

- TopAndBottom wrap (대다수): wrap != Square 가드 → 변경 없음 ✓
- TAC 인라인 표: !is_tac 가드 → 변경 없음 ✓
- 호스트 텍스트 < 표 영역: max() → 변경 없음 ✓ (실 케이스 측정 가능)
- 다단 (Multi-column): paragraph 내부 offset 만 사용 → 다단 영향 없음 ✓

### 5-2. 큰 데이터셋 측정

- aift.hwp 77 페이지: 0 변경 — 다양한 표/그림 wrap 패턴 회귀 차단 확인
- exam_math/exam_math_no 40 페이지: 0 변경 — 수식 + 인라인 표 회귀 차단 확인
- synam-001.hwp 35 페이지: 0 변경 — 다단 + 표 분할 회귀 차단 확인

## 6. 단위 게이트 (Stage 3 기록)

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | 1116 passed |
| `cargo test --test svg_snapshot` | 6/6 |
| `cargo test --test issue_418` | 1/1 |
| `cargo test --test issue_501` | 1/1 |
| `cargo clippy --lib` | 2 pre-existing errors (본 task 미관여) |

## 7. 산출물

| 산출물 | 위치 |
|--------|------|
| 회귀 측정 데이터 | `/tmp/svg_diff_before/` + `/tmp/svg_diff_after/` (8 샘플 × ~24 페이지 평균) |
| 본 보고서 | `mydocs/working/task_m100_533_stage4.md` |
| 코드 변경 | **0** (Stage 3 변경 검증만) |

## 8. 다음 단계

작업지시자 시각 판정 (Stage 5) 후 최종 보고서 + close.

작업지시자 검증 영역:
- exam_kor p14 우측 단 줄간격 일관성 (본 결함 직접 대상)
- exam_kor p14 좌측 단 pi=37/40/47 직후 줄간격 일관성 (동일 패턴)
- exam_kor p17 (Task #528 PUA 옛한글 영역 + 본 fix 의 12.4 px shift) — 옛한글 자모 합자 / 페이지 외곽 박스 정합 확인

## 9. 승인 게이트

- [x] 광범위 샘플 회귀 측정 (192 페이지)
- [x] 변경 페이지 (2 건) 모두 동일 본질 + 의도된 정정 확인
- [x] 회귀 차단 가드 측정 검증 (190 페이지 byte-identical)
- [x] 호스트 텍스트 영역 < 표 영역 케이스 회귀 0
- [x] 단위 테스트 1116 통과 (Stage 3)
