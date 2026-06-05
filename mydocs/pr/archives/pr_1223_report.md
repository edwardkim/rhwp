# PR #1223 처리 보고서 — 수식 포함 줄 본문 한글 압축·겹침 해소 (거짓 오버플로우)

- **작성일**: 2026-06-01
- **PR**: #1223 → **MERGED** (devel, 로컬 `--no-ff` 머지)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터)
- **연결 이슈**: #1219 → **CLOSED** (`closes #1219`)
- **판단**: **머지** ✅ (작업지시자 SVG 시각 판정 통과)

## 결정 사유

인라인 수식(TAC) 포함 줄의 본문 한글이 거짓 오버플로우로 음수 자간 압축되던 결함을, 측정을 렌더
규칙(line_tac_offsets)에 통일 + 선두 미주 이중계상 제거로 해결. golden SVG 8건 무회귀, 한글 advance
8.96→11.93px(≈PDF) 정량 정합, 정상 줄 불변.

## 변경 요약 (7 파일, +576/−15)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` | TAC 측정을 `line_tac_offsets`(렌더와 동일 `pos < 다음 줄 시작` 엄격 미만)로 통일(est_x 루프 + total_tac_width_in_line). footnote_positions 측정에서 start_line==0 선두 Endnote 제외 |
| `tests/issue_1219_equation_line_hangul_advance.rs` | 문26 줄 한글 advance ≥ 11.0px 회귀 가드 |
| mydocs ×5 | 컨트리뷰터 작업 문서 |

## 근본 원인 (둘 다 측정/렌더 불일치)

1. 줄 경계 오포함: `pos <= line_end`(포함) 필터가 줄 끝 위치(=다음 줄 선두) 수식을 현재 줄 폭에
   오포함(문26 라인0 에 다음 줄 `a₁=b₁=1` 55px). → 거짓 오버플로우 → Left 줄 음수 자간 압축.
2. 선두 미주 이중계상: endnote_marker_x_advance(선두 마커 풀사이즈, available_width 차감) +
   footnote_positions 위첨자. 렌더는 인라인 위첨자 안 그림.

## 검증 결과

| 항목 | 결과 |
|------|------|
| merge `--no-ff` | ✅ CLEAN (충돌 0) |
| fmt / build | ✅ |
| 전체 테스트 `cargo test --tests` | ✅ **1925 passed, 0 failed** |
| **golden SVG 스냅샷** | ✅ **8 passed** (인라인 TAC/각주/목차/우측탭 회귀 0 — 핵심 가드) |
| issue_1219 회귀 테스트 | ✅ 1 passed (한글 advance ≥ 11.0px) |
| **한글 advance 정량** | ✅ 압축 줄 8.96px→11.93px(≈PDF 12px) 12글자, 정상 줄 12.0px 보정 전후 동일(무관 줄 불변) |
| **SVG 시각 판정** | ✅ **통과** (작업지시자, 6쪽 문26 한글 겹침 해소 ↔ 한글 2022 PDF) |
| CI(PR) | ✅ 전부 PASS |
| WASM | (머지 후 빌드 — 측정 경로 변경) |

산출물: `output/poc/pr1223/{before,after}/3-09월_교육_통합_2023_006.svg`.

## 처리 절차

1. PR 정보 — MERGEABLE/BEHIND, CI green, paragraph_layout.rs. @planet6897.
2. diff + 진단 코드 확인(tac_offsets_for_line 엄격 미만, est_x/total_tac_width 오포함). 트러블슈팅 검색.
3. 로컬 `pr1223-verify`: 머지 / fmt / test 1925 / **golden SVG 8건** / issue_1219 / 한글 advance 8.96→11.93 정량(정상 줄 불변).
4. **작업지시자 SVG 시각 판정 통과**.
5. devel `--no-ff` 머지 + push + WASM 빌드. 이슈 #1219 클로즈.

## 비고

- 글리프 크기 차이(Noto Sans CJK ↔ 한컴 돋움 폰트 대체)는 본 건과 별개 — 별도 이슈(PR 본문 명시).
  본 PR 은 advance(자간) 압축 해소이지 글리프 크기 아님.
- #1220(같은 샘플 4쪽 wrap=Square)과 독립.
