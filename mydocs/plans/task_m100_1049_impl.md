# 구현 계획서 — #1049 본문 하단 잔여 overflow 수정

- 타스크: #1049 (M100), 브랜치 `local/task1049`
- 수행계획서: `task_m100_1049.md`
- 작성일: 2026-05-21 (v2 갱신: Stage 1 진단 반영)
- 단계: 3단계 (진단 격리 → 수정 → 회귀 검증)

> **v2 갱신 사유 (Stage 1 결과 반영, `task_m100_1049_stage1.md`)**: Stage 1 진단으로
> #1046 의 "줄높이 과대 계산" 가설이 **반증**됨. 렌더러 줄높이는 정확(20.0px)하며, 잔여
> 4.6px 의 진짜 원인은 `height_cursor.rs::vpos_adjust` 의 **lazy_base 오산출(+12.8px 전진)**.
> Stage 2 수정 대상을 줄높이 모델 → **VPOS_CORR lazy_base** 로 재정의한다.

## 단계 구성 개요

| 단계 | 제목 | 소스 수정 | 핵심 산출 |
|------|------|----------|-----------|
| Stage 1 | 근본 원인 격리 (진단) | ✕ (무수정) | **완료** — 원인=vpos_adjust lazy_base (줄높이 무죄) |
| Stage 2 | VPOS_CORR lazy_base 수정 | ○ (최소 침습) | pi=760 +12.8px 점프 제거, pi=781 overflow 해소 |
| Stage 3 | 회귀 검증 + 시각 대조 | ✕ (검증) | 골든 SVG 전수 + PDF 대조 |

---

## Stage 1 — 근본 원인 격리 (소스 무수정)

**목표**: 렌더러가 pi=760 줄높이를 34.8px(≈23.7pt)로 계산하는 **정확한 산출 경로와 입력값**을
확정한다. 의심 4가지 중 실제 원인을 데이터로 격리.

**작업**:
1. baseline 빌드(`cargo build`) + `LAYOUT_OVERFLOW` 로 `aift.hwp` 현재 overflow 5건 재확인.
2. pi=760 의 입력값 덤프:
   - `rhwp dump samples/aift.hwp -s {sec} -p 760` → ParaShape(line_spacing_type/value,
     indent), LINE_SEG(lh/th/bl/ls), run별 char_style_id.
   - 각 char_style_id 의 실제 font_size(끝마커 id=450 포함) 확인.
3. 줄높이 산출 지점 계측(임시 eprintln 또는 기존 DRIFT 플래그):
   - layout `paragraph_layout.rs:1145-1166` 의 max_fs 와 `corrected_line_height` 입력/출력.
   - `corrected_line_height`(mod.rs:545-561) 분기·결과.
   - `recompute_lh` 여부와 `line_spacing_px` 가산값(typeset 경로 사용 시 `typeset.rs:1485-1493`).
4. 34.8px = (어떤 max_fs) × (어떤 배율) + (어떤 line_spacing 가산) 으로 **분해**.
   - 가설 검증: 끝마커 폰트가 max 에 끼는가 / line_spacing 이중 가산인가 /
     small-% 분기 오류인가 / 3경로 불일치인가.

**완료 기준**: 34.8px 의 산출식을 입력값 단위로 분해해 1개 근본 원인으로 좁힌다.
→ `task_m100_1049_stage1.md` 작성 + 승인 요청 (소스 커밋 없음).

---

## Stage 2 — 줄높이 모델 수정 (최소 침습)

**목표**: Stage 1에서 확정한 원인(`height_cursor.rs::vpos_adjust` lazy_base 오산출)을
**최소 범위**로 수정해 pi=760 의 +12.8px forward 점프를 제거하고 pi=781 overflow 를 해소.

**수정 대상 (확정)**: `src/renderer/height_cursor.rs::vpos_adjust` 의 lazy_base 산출
(line 120-144). 줄높이 모델(`corrected_line_height`, max_fs, 끝마커, hanging indent,
small-%)은 **무죄로 확정 — 미변경**.

**작업** (Stage 1 후보 (a)~(c) 중 회귀 최소안 선정):
- (a, 우선) **직전 항목이 인라인 TAC 표였던 lazy 재산출**에서 Task #1022 v2 trailing-ls
  bridge(`+ trailing_ls_hu`)를 미적용. lazy_base 가 제목 trailing 줄간격(960 HU)만큼
  과소산출되는 것을 차단. (가장 국소적 — TAC-리셋 직후 조건만 추가.)
- (b) 인라인(작은) TAC 표 후 `vpos_page_base` 리셋(layout.rs:2538) 억제하여 page base
  유지 — #539 TAC 처리 영향 크므로 (a) 실패 시 검토.
- (c) page-path 복원 경로 우선 — 직전 컬럼-top page_base 가 살아있으면 lazy 대신 사용.

**검증 (수정 정합)**:
- `RHWP_VPOS_DEBUG=1 -p 111`: pi=760 end_y 가 y_in(224.67) 부근(점프 ≤8px)으로, +12.8
  forward 점프 소멸 확인. lazy_base ≈ 5960622 (또는 page base 5961289 복원).
- `LAYOUT_OVERFLOW=1`: pi=781 4.6px overflow 소멸. page-larger 2건(323/567) 불변.

**제약**: 페이지네이터(typeset, vpos 계산) 미변경. 변경 파일만 `cargo fmt`(--all 금지).
기능/포맷 커밋 분리.

**완료 기준**: pi=781 overflow 해소, +12.8px 점프 제거, 신규 overflow 0.
→ 소스 + `task_m100_1049_stage2.md` 커밋, 승인 요청.

---

## Stage 3 — 회귀 검증 + 시각 대조

**목표**: VPOS_CORR(핵심 경로) 수정의 회귀 없음을 전수 검증.

**작업**:
1. `cargo test` 전체 — 골든 SVG 전수 통과(diff 0) 또는 의도된 변화만 식별·설명.
2. 대상 185p HWPX 전체 `LAYOUT_OVERFLOW` 3→2 확인 (page-larger pi=323/567만 잔존).
   aift.hwp 74p 불변(회귀 0) 확인.
3. 한글 2022 PDF(`pdf/2. ...-2022.pdf`) 와 page 112(보안 서약서 폼) SVG 시각 정합 대조.
4. VPOS_CORR 회귀 케이스 정밀 검증: footnote-01 p1(trailing-ls bridge 정상 필요),
   exam_kor p5(lazy_base 음수 over-correction 방지), #874/#991/#412 골든.
5. `cargo clippy` 경고 없음.

**완료 기준**: 골든 SVG diff 0(또는 설명된 의도 변화), 대상 overflow 3→2, aift 74p 불변,
PDF 시각 정합, VPOS 회귀 케이스 무회귀.
→ `task_m100_1049_stage3.md` 작성, 최종 결과보고서(`task_m100_1049_report.md`) 작성,
승인 요청.

---

## 검증 명령 요약 (대상 = 비공개 185p HWPX, 커밋 금지)

```bash
cargo build
F="<비공개 185p 재정통합 제안요청서 hwpx>"   # 비공개 샘플 — 커밋 제외
LAYOUT_OVERFLOW=1 rhwp export-svg "$F" -o output/poc/ 2>&1 | grep "^LAYOUT_OVERFLOW:"
RHWP_VPOS_DEBUG=1 rhwp export-svg "$F" -p 111 2>&1 | grep "VPOS_CORR.*col_y=105.81.*pi=76"
RHWP_TABLE_DRIFT=1 rhwp export-svg "$F" -p 111 2>&1 | grep "LAYOUT_Y.*pi=7[68]"
cargo test            # 골든 SVG 전수 (회귀 0 필수)
cargo clippy
```
