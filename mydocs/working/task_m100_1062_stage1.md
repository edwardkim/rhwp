# Stage 1 보고서 — Task #1062: 원인 정밀화 + 구분 신호 측정

- 브랜치: `local/task1062` (소스 무변경, 진단만)
- 도구: `RHWP_VPOS_DEBUG`, `RHWP_TYPESET_DRIFT` (devel+#1048 빌드 `/tmp/rhwp_pr`)

## 1. drift 정량 = trailing line_spacing 확정

시험지 단일 줄 빈 문단(`lh=900hu, ls=452hu`):
- vpos-delta(렌더러 전진) = `lh + ls` = 1352hu = **18.03px**
- formatter `total_height` = lh+ls = 18px, `height_for_fit` = lh = 12px
- `ls=452hu = 6.03px` = 관측된 문단당 drift

## 2. 메커니즘 — 페이지네이터↔렌더러 누적 불일치

| 엔진 | 문단당 전진 | 비고 |
|------|-----------|------|
| 페이지네이터 (typeset.rs:1834-1841, 다단) | `height_for_fit` = total − ls = **12px** | Task #391 |
| 렌더러 (height_cursor vpos→y) | vpos-delta = lh+ls = **18px** | 소스 vpos가 ls 포함 |

문단당 페이지네이터가 **6px 과소 계상** → 빈 문단 다수 페이지(88문단)에서 ~530px 과밀 →
페이지네이터 "fit", 렌더러 본문 밖 적층. (`VPOS_CORR` 실측: pi 500~508 등 end_y 1103→1294px > col_bottom 1092.3)

두 하위 패턴 모두 동일 과소 계상에서 발생:
- `path=lazy base=0`: lazy_base 미확립(0) → 절대 vpos 매핑
- `path=page base=N`: page_base 설정됐으나 vpos_end − base 가 컬럼 초과

## 3. ⚠️ 단순 구분 신호(vpos 연속성)는 실패

수행계획서가 가정한 "빈 문단 연속 vs 후행 표" 구분을 vpos 연속성으로 시도했으나 **반증**:

| 파일 | vpos 연속(INCL_ls) | EXCL_ls | other |
|------|------|------|------|
| 시험지 3-09 2022 | 1115 | 0 | 9 |
| exam_eng (#391 비회귀) | 200 | 0 | 80 |
| 복학원서 (#1049 비회귀) | 24 | 0 | 0 |

→ 셋 다 vpos가 trailing_ls 포함(INCL_ls). `vpos_continuous`/`prev_has_text` 류 신호로
exam_eng(ls 제거 필요)과 시험지(ls 유지 필요)를 가를 수 없음. **height_cursor 가드 확장 노선 폐기.**

## 4. 핵심 발견 — vpos_h 가 일관된 정답 (누적 통일)

`TYPESET_DRIFT`의 `diff = fmt_total − vpos_h` 누적:

| 파일 | 문단수 | 누적 (fmt − vpos) |
|------|------|------|
| 시험지 3-09 2022 | 457 | +2767px (전건 diff>0) |
| exam_eng (#391) | 301 | +2822px |
| exam_kor (#1022) | 718 | +7506px |
| k-water-rfp (#359 단단) | 304 | +2570px |

**모든 케이스에서 formatter total_height > vpos_h (항상 과대).** 렌더러는 vpos 기반이므로
**vpos_h(LINE_SEG 실제 세로 span)가 렌더러와 일치하는 단일 정답.**

→ 페이지네이터가 formatter `total_height`/`height_for_fit` 대신 **`vpos_h`를 누적**하면
exam_eng·exam_kor·k-water-rfp·시험지가 **한 모델로 통일**(케이스별 trailing_ls 땜질 불요).
`height_for_fit`(= 마지막 항목 fit 시 trailing_ls 제외)는 fit 판정에만 유지.

## 5. 결론 — 수정 노선 재조정

- **수정 위치 변경**: `height_cursor.rs` 가드 확장(폐기) → **`typeset.rs` 다단(필요 시 단단) 누적을 vpos_h 기반으로 통일**. (수행계획서 범위 "필요 시 typeset.rs" 내)
- 모델: 누적 = vpos_h (line_segs 있을 때), fit = vpos_h − trailing_ls. line_segs 부재 시 formatter fallback.
- 스코프: FullParagraph 우선. 표/도형/Partial 별도 검토.

## 6. 리스크

- 누적 핵심 경로 변경 → 회귀면 넓음. 단 vpos_h 통일은 drift를 **전반적으로 감소**시키는
  방향(렌더러 일치)이라 악화 가능성 낮음. Stage 2 페이퍼 검증 + Stage 4 전 251샘플 회귀 필수.
- 비회귀 확인 대상: exam_eng 8p, exam_kor p5, k-water-rfp p3, 복학원서 p1, footnote-01 p1.

## 다음 (Stage 2)

vpos_h 누적 모델을 5개 비회귀 케이스 + 대상 4종에 대해 예상 동작 표로 페이퍼 검증 →
모순 없으면 Stage 3 구현.
