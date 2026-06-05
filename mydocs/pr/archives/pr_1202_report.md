# PR #1202 처리 보고서 — HWPX 미주/각주 prefixChar 파싱 (마커 접두문자 '문' 복원)

- **작성일**: 2026-06-01
- **PR**: #1202 → **MERGED** (devel, 머지커밋 `73034de9`)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터 — #1182/#1164/#1148 머지, 미주/수식 시리즈)
- **연결 이슈**: #1199 → **CLOSED** (`closes #1199`, cross-repo `--no-ff` 라 수동 클로즈)
- **판단**: **머지** ✅ (작업지시자 시각 판정 통과)

## 결정 사유

HWPX 미주/각주 마커 "문N)" 의 접두문자 '문' 이 "N)" 로 탈락하는 결함을, 파서가 prefixChar
(코드포인트 숫자)를 suffixChar 와 대칭으로 읽도록 분기 추가하여 해결. 파서 단일 파일, 순수 추가,
무회귀, 한글 2022 PDF 정합. 위험 낮음.

## 변경 요약 (7 파일, +353 / −0)

| 파일 | 변경 |
|------|------|
| `src/parser/hwpx/section.rs` (+118) | `parse_ctrl_endnote`/`parse_ctrl_footnote` 에 `b"prefixChar"` 분기 추가 → `before_decoration_letter` 매핑. 회귀 테스트 2건 |
| mydocs (plans/report/working ×6) | 컨트리뷰터 작업 문서 |

## 검증 결과

| 단계 | 결과 |
|------|------|
| merge `--no-ff` | ✅ CLEAN (충돌 0) |
| fmt / build | ✅ |
| 전체 테스트 `cargo test --tests` | ✅ **1896 passed, 0 failed** (머지 후 재검증 동일) |
| 신규 회귀 2건 (prefix 매핑 / 누락 시 0) | ✅ |
| **미주 '문' 접두 복원 (정량)** | ✅ 보정 전/후 9쪽 **0→7**, 10쪽 **0→6** |
| **한글 2022 PDF 정답지 정합** | ✅ 9쪽 7개 = "문1)~문7)" 일치 |
| **시각 판정** | ✅ **통과** (작업지시자, SVG↔PDF 비교) |
| WASM | ✅ pkg 빌드 (14:24) |
| 머지 검증 | ✅ PR head(`826dacb7`) 조상 확인 |
| CI(PR) | ✅ Build&Test / Analyze ×3 / CodeQL 전부 SUCCESS |

## 처리 절차

1. PR 정보 확인 — MERGEABLE/BEHIND, CI green, 파서 단일 파일. 컨트리뷰터 사이클 점검(#1202/1203/1208 동시).
2. prefixChar 분기 diff + 회귀 테스트 검토. 트러블슈팅 사전 검색(미주/직렬화).
3. 로컬 `pr1202-verify` 머지 시뮬: fmt/build/test --tests 1896 / 보정 전·후 '문' 정량 대조 / PDF 대조 → `pr_1202_review.md`.
4. **작업지시자 시각 판정 통과** (SVG↔PDF).
5. 메인테이너 로컬 `--no-ff` 머지(`84819f99..73034de9`) + 재검증 + push. GitHub 자동 MERGED 인식.
6. WASM 빌드. 이슈 #1199 클로즈 + 머지 코멘트 + 보고서.

## 비고

- 우측 단 "다른 풀이" 미표시는 별개(#1200, curve 도형 외곽선 — PR #1203 진행).
- @planet6897 미주/수식 시리즈 중 본 PR(미주 prefix)은 독립적 — #1203/#1208 과 무충돌.
- cross-repo `--no-ff` 이나 이번엔 GitHub 자동 MERGED 인식(#1159 동일 패턴). 이슈는 수동 클로즈.
