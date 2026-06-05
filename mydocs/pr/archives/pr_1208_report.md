# PR #1208 처리 보고서 — HWPX 수식 스크립트 토큰 처리 (root/sqrt glued, rm+bar, prime, cdots)

- **작성일**: 2026-06-01
- **PR**: #1208 → **MERGED** (devel, 로컬 `--no-ff` 머지 `77c7c27c` + cdots 보정 후속 커밋)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터)
- **연결 이슈**: #1204 → **CLOSED** (`closes #1204`)
- **판단**: **머지** ✅ (작업지시자 시각 판정 통과)

## 결정 사유

20쪽 등 수식 스크립트 토큰 leak(root3/rm bar/primeF)을, tokenizer glued-split 확장 +
parser decoration 위임으로 해결. allowlist(GLUE_SAFE) 기반 케이스별 가드 — 실문서 23쪽 정량
회귀 0, 전부 leak→정상 렌더. 메인테이너가 cdots 연접 leak 까지 동일 메커니즘으로 보정.

## 변경 요약 (9 파일, +484/−5 + cdots 보정)

| 파일 | 변경 |
|------|------|
| `src/renderer/equation/tokenizer.rs` | root/sqrt·GEQ/LEQ digit-split, prime split, `longest_keyword_prefix`(GLUE_SAFE), over/atop ≤2자 글자 분리, 대문자 글꼴 |
| `src/renderer/equation/parser.rs` | `parse_single_or_group` Command 분기 → `parse_command` 위임(bar leak), DECORATIONS/FONT_STYLES 대소문자 fallback |
| mydocs ×7 | 컨트리뷰터 작업 문서 |
| **(메인테이너 보정)** tokenizer.rs | GLUE_SAFE 에 `cdots`/`ldots`/`vdots`/`ddots` 추가 + `test_dots_glued_split` |

## PR 본문 ↔ 실제 diff 불일치 (고지)

PR 본문은 "A/B/C 최소 수정"으로 기술했으나 실제로는 **#1204-E**(over/atop 글자 분리 +
범용 `longest_keyword_prefix`), 관계연산자 digit-split, 대문자 변형이 포함됨 — 수식 토크나이저
일반 동작을 바꾸는 변경. allowlist 로 제한되어 회귀 0 이나, 컨트리뷰터에 본문 범위 정정 요청 예정.

## 검증 결과

| 항목 | 결과 |
|------|------|
| merge `--no-ff` | ✅ CLEAN (equation 모듈, 충돌 0) |
| fmt / build | ✅ |
| 전체 테스트 `cargo test --tests` | ✅ **1907 passed, 0 failed** (수식 회귀 9건 + dots 1건) |
| **20쪽 leak 정량** | ✅ root3 7→0, bar 5→0, prime 1→0 |
| **전체 23쪽 텍스트 diff** | ✅ 변화 13페이지 전부 leak→정상 렌더(개선) |
| **오분리 회귀** | ✅ 0건 (sintheta→sin θ, angleOPC→∠ OPC, root3→√3 …) |
| **cdots 연접 leak** | ✅ `cdotscdots` 14·18·19·20쪽 → 0, ⋯ 정상 렌더 |
| **시각 판정** | ✅ **통과** (작업지시자, 20쪽 √·overline·∠·⋯ ↔ PDF) |
| WASM | (빌드 진행) |

## 처리 절차

1. PR 정보 — MERGEABLE/BEHIND, CI green(Canvas visual diff 포함), equation 2파일. 컨트리뷰터 사이클(#1208).
2. tokenizer/parser diff 정밀 검토 — 본문보다 넓은 범위(#1204-E) 식별. 트러블슈팅 검색(equation/Task#576/#1122).
3. 로컬 `pr1208-verify` 머지 시뮬: fmt/build/test 1906 / 20쪽 leak 정량 / 전체 23쪽 정량 회귀 / 오분리 탐침(secant류 실문서 부재).
4. 작업지시자 cdots leak 지적 → 메인테이너 보정(GLUE_SAFE dots 4종) → 1907 passed, cdots 0.
5. **작업지시자 시각 판정 통과**.
6. devel `--no-ff` 머지(`77c7c27c`) + cdots 보정 후속 커밋 + push. 이슈 #1204 클로즈.

## 비고

- GLUE_SAFE 가 다글자 변수와 충돌하는 새 샘플 발생 시 allowlist 조정 필요(현 실문서 무회귀).
- 모호 4자 `dots` 는 변수 충돌 우려로 제외(케이스별 가드 — `feedback_hancom_compat_specific_over_general`).
