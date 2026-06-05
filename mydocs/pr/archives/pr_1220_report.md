# PR #1220 처리 보고서 — HWP5 wrap=Square 호스트 본문 커서 전진 (답안↔문제 겹침)

- **작성일**: 2026-06-01
- **PR**: #1220 → **MERGED** (devel, 로컬 `--no-ff` 머지)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터)
- **연결 이슈**: #1218 → **CLOSED** (`closes #1218`)
- **판단**: **머지** ✅ (작업지시자 시각 판정 통과)

## 결정 사유

wrap=Square 인라인 표 호스트 본문이 표보다 길 때 다음 단락이 표 하단에서 시작해 겹치던 결함을,
호스트 본문 텍스트 높이만큼 커서를 전진시켜 해결. 가드(`> y_offset`)로 기존 케이스 불변. 핵심
위험인 double advance 트러블슈팅(exam_science 회귀)을 보정 전/후 동일값으로 무재발 확증.

## 변경 요약 (8 파일, +295/−0)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` (+19, 단일 hunk) | Square 호스트 Table 처리에서 `layout_wrap_around_paras` 직후 `host_text_bottom = table_y_before + (text_h - last_ls)` 까지 커서 전진. `host_text_bottom > y_offset` 가드 |
| mydocs ×7 | 컨트리뷰터 작업 문서 |

## double advance 트러블슈팅 정합 (핵심)

`square_wrap_pic_bottom_double_advance.md` 경고: Square wrap 커서 advance 가 wrap-around 누적과
결합하면 exam_science.hwp 4→6쪽 회귀. 본 PR 은 **호스트 본문 단락 자기 텍스트 높이 기준**이라
wrap-around 누적과 결합 안 함(함정은 그림+누적). 표 호스트 한정 + 가드로 안전.

## 검증 결과

| 항목 | 결과 |
|------|------|
| merge `--no-ff` | ✅ CLEAN (충돌 0) |
| fmt / build | ✅ |
| 전체 테스트 `cargo test --tests` | ✅ **1924 passed, 0 failed** (svg_snapshot / issue_546 wrap_around 회귀 포함) |
| **exam_science 회귀(double advance)** | ✅ 4쪽/37items/45items 보정 전후 동일 |
| **문26 겹침 해소(정량)** | ✅ 최하단 ① y 906.3(문제끝줄 904.7 겹침) → 922.8(18px 분리) |
| **시각 판정 (SVG)** | ✅ **통과** (작업지시자, 4쪽 ①~⑤ 분리 ↔ 한글 2022 PDF) |
| **시각 판정 (rhwp-studio)** | ✅ **통과** (WASM 재빌드 후 편집기 canvas 경로 — SVG/편집기 양쪽 정합) |
| CI(PR) | ✅ 전부 PASS |
| WASM | ✅ pkg 빌드 (5,443,309 bytes) — 렌더 경로 변경 노출 |

산출물: `output/poc/pr1220/{before,after}/3-09월_교육_통합_2023_004.svg`.

## 처리 절차

1. PR 정보 — MERGEABLE/BEHIND, CI green, layout.rs 단일 hunk. @planet6897.
2. diff + 트러블슈팅 정독(square_wrap double advance) — 회귀 위험 식별.
3. 로컬 `pr1220-verify`: 머지 / fmt / test 1924 / **exam_science 페이지·item 보정 전후 동일(무회귀)** / 문26 겹침 해소 정량 / before·after SVG.
4. **작업지시자 시각 판정 통과**.
5. devel `--no-ff` 머지 + push + WASM 빌드. 이슈 #1218 클로즈.

## 비고

- z-표 행 압축(셀 내부 세로정렬/줄높이)은 별도 서브시스템 — 별도 이슈(PR 본문 명시).
- 트러블슈팅 옵션 3(HWP5 case-guard)은 그림+wrap-around 결합 함정 대비책이나, 본 PR 은 호스트 본문
  자기 높이 기준이라 해당 함정과 무관함이 실측 확인.
