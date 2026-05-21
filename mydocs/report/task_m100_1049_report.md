# 최종 결과보고서 — #1049 본문 하단 잔여 overflow (VPOS_CORR lazy_base)

- 타스크: #1049 (M100), 브랜치 `local/task1049` (← `local/task1046`)
- 부모: #1046 (PR #1048, 측정 통일 B — overflow 18→5)
- 대상: 비공개 185p "재정통합 제안요청서" hwpx, sec0 pi=781 본문 4.6px overflow
- 검증 권위: `pdf/2. 인공지능(AI) … 제안요청서-2022.pdf` (한컴 2022, 179p)
- 작성일: 2026-05-21

## 1. 개요

#1046 에서 별도 이슈로 분리한 잔여 1건(pi=781, 4.6px)을 해소. #1046 진단의 "렌더러 줄높이
과대 계산" 가설을 Stage 1 에서 **반증**하고, 진짜 원인이 `vpos_adjust` 의 lazy_base 오산출임을
격리한 뒤 1지점 수정으로 해결했다.

## 2. 결과 요약

| 지표 | baseline | 최종 |
|------|----------|------|
| pi=781 본문 overflow | 4.6px | **0 (해소)** |
| 대상 overflow 총건 | 3 | **2** (page-larger pi=323/567만) |
| `cargo test --release` | 1516 / 0 | **1516 passed / 0 failed** |
| `cargo clippy --lib` | — | 경고 0 |
| 대상 페이지 수 | 185 | **184** (한컴 179p 에 1쪽 근접) |
| aift.hwp 페이지 수 | 74 | 74 (불변) |

## 3. 근본 원인 (Stage 1 격리)

- **줄높이 무죄**: 렌더러 `corrected_line_height`(pi=760)=20.0px, 줄 advance 22.03px 로 정확.
  #1046 진단(`_v2_stage3_781diag.md`)의 34.8px 과대 줄높이 가설은 메커니즘이 틀림.
- **진짜 원인**: `src/renderer/height_cursor.rs::vpos_adjust` 가 인라인 1×1 TAC 표(pi=758)
  직후 `vpos_page_base` 리셋(layout.rs:2538) → pi=760 lazy 재산출 시 Task #1022 v2
  trailing-ls bridge(`+trailing_ls_hu`)가 직전 제목(pi=759) trailing 줄간격 960HU(=12.8px)를
  base 에서 또 빼 lazy_base 과소(5959663, 정답 5961289 부근) → pi=760 부터 lazy 문단 전부
  +12.8px 전진 → pi=781 본문 4.6px 초과. (페이지네이터는 인라인 TAC 에 리셋 안 해 정확했음 —
  두 엔진 발산이 본질.)

## 4. 수정 (Stage 2, 1파일·1지점)

`vpos_adjust` lazy_base 산출에서 trailing-ls bridge 를 **vpos 연속 + 직전 실텍스트 문단**일 때
끔(나머지는 종전 유지):
```rust
let prev_has_text = prev_para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
let vpos_continuous = matches!(curr_first_vpos, Some(v) if v <= prev_vpos_end);
let trailing_ls_hu = if vpos_continuous && prev_has_text { 0 } else { /* 종전 #1022 v2 */ };
```
- 연속(curr_first_vpos==prev_vpos_end)이고 실텍스트면 trailing_ls 가 이미 연속 vpos·
  sequential y 에 포함 → bridge off (#1049 해소).
- gap(top-box 후 본문·footnote-01 p1) 또는 빈 문단 prev(복학원서 page1, 빈줄 높이 억제)는
  bridge 유지 → 무회귀.

## 5. 검증 (Stage 3)

- 회귀: 전체 1516 passed/0 failed, 골든 SVG 전수 통과, clippy 0. footnote-01·복학원서·
  exam_kor·#874/#991 무회귀.
- 시각: 보안 서약서(대표자용) 폼이 한컴 2022 PDF p110 과 레이아웃 일치 — 하단 "생 년 월 일 :"
  줄(pi=781) 본문 내 배치 정합.

## 6. 잔여 (범위 외)

- **page-larger 2건** (pi=323 단독 표, pi=567 nested) — 단일 항목이 본문보다 큼, #1046 부터 범위 외.
- **한컴 179p 대비 +5p 과다 분할** — pi=758 류 인라인 TAC 표 과대렌더(+8.9px 잔여 드리프트) 등
  별도 누적 축. 본 타스크는 그 중 +12.8px(vpos_adjust bridge)만 정정. 추가 정합은 별도 이슈 권장.

## 7. 커밋
- f58f77ec (Stage 1 진단) · e2577517 (Stage 2 수정 + Stage2 보고서) · 본 커밋(Stage3 + 최종보고서).

## 8. 상주 인프라 (env 게이트, 동작 불변)
- `RHWP_VPOS_DEBUG`: VPOS_CORR(path/base/end_y/applied) — lazy/page base 산출 대조.
- `RHWP_TYPESET_DRIFT(_LINES)` / `RHWP_TABLE_DRIFT`(LAYOUT_Y) — 페이지네이터↔렌더러 y 대조.
