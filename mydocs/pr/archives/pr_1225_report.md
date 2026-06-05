# PR #1225 처리 보고서 — HWP5 수식-only 셀 z-표 행 압축 수정 (tac 순서매핑)

- **작성일**: 2026-06-01
- **PR**: #1225 → **MERGED** (devel, 로컬 `--no-ff` 머지)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터)
- **연결 이슈**: #1221 → **CLOSED** (`closes #1221`)
- **판단**: **머지** ✅ (작업지시자 시각 판정 통과)

## 결정 사유

수식-only 셀 문단의 char_start degenerate(모든 줄 0)로 tac→줄 char-범위 매핑이 붕괴해 z-표 두
수식이 한 줄에 겹치던 결함을, 4조건 가드 하에 순서(index) 1:1 매핑으로 해결. 전체 23쪽 중 4쪽
z-표 1곳만 변화(세로 분리), 22쪽 무영향. golden 8건·1925 passed.

## 변경 요약 (7 파일, +182/−4)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs` (+23, 빈-runs tac 블록) | `index_based_tac` 가드(다중 줄 + 줄수==tac수 + 모든 줄 빈 runs + char_start degenerate) 충족 시 tac→줄 순서 1:1 매핑. 그 외 char-범위 유지 |
| mydocs ×6 | 컨트리뷰터 작업 문서 |

## 근본 원인 (계측 확정)

z값=인라인 수식. 수식-only 셀 문단은 텍스트 없어 모든 LINE_SEG.text_start=0 → 모든 줄 char_start=0.
char-범위 매핑: line0 `[0,0)` 빈 범위(행 빔) / line1 `[0,MAX)` 모든 수식 흡수("1.01." 가로 겹침).

## 검증 결과

| 항목 | 결과 |
|------|------|
| merge `--no-ff` | ✅ CLEAN (#1223 과 같은 파일 다른 블록, 충돌 0) |
| fmt / build | ✅ |
| 전체 테스트 `cargo test --tests` | ✅ **1925 passed, 0 failed** |
| golden SVG 스냅샷 | ✅ 8 passed (표/수식 회귀 0) |
| **전체 23쪽 SVG diff (devel vs devel+#1225)** | ✅ **4쪽 1페이지만 변화(4줄), 22쪽 무영향** |
| **z-표 행 분리(정량)** | ✅ cell-clip-178 수식 2개 보정 전 둘 다 y=925.3(겹침) → 후 y=913.3/925.3(세로 분리)+x 305.2 정렬 |
| **시각 판정** | ✅ **통과** (작업지시자, 4쪽 z=1.0/1.1/1.2/1.3 각 행 분리 ↔ 한글 2022 PDF) |
| CI(PR) | ✅ 전부 PASS |
| WASM | (머지 후 빌드 — 셀 렌더 경로 변경) |

## 처리 절차

1. PR 정보 — MERGEABLE/BEHIND, CI green, paragraph_layout.rs. @planet6897. #1220 z-표 후속.
2. diff + 진단 코드 확인(index_based_tac 4조건 가드). 트러블슈팅 검색(tac/줄매핑).
3. 로컬 `pr1225-verify`: 머지 / fmt / test 1925 / golden 8 / 전체 23쪽 diff(4쪽만 변화) / z-표 수식 y 925.3→913.3·925.3 분리.
   - 초기 stash 꼬임으로 byte 동일 오판 → devel 단독 vs 머지본 전체 diff 재측정으로 정정·확정.
4. **작업지시자 시각 판정 통과**.
5. devel `--no-ff` 머지 + push + WASM 빌드. 이슈 #1221 클로즈.

## 비고

- #1220(wrap=Square 답안 겹침)·#1223(수식 줄 한글 압축)과 같은 샘플의 다른 결함 — 독립.
- 셀 valign/line-height 클램프/table_layout/shape 경로는 z값 미통과(PR 진단 배제 확인).
