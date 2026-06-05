# PR #1208 검토 — HWPX 수식 스크립트 토큰 처리 (root/sqrt glued, rm+bar, prime glued)

- **작성일**: 2026-06-01
- **PR**: #1208 (OPEN)
- **컨트리뷰터**: @planet6897 (핵심 컨트리뷰터 — #1203/#1202/#1182/#1164/#1148/#1095 머지)
- **연결 이슈**: #1204 (`closes #1204`)
- **base/head**: `devel` ← `fix/1204-equation-tokens` (cross-repo)
- **규모**: 9 파일, +484/−5 (소스 `equation/tokenizer.rs` + `equation/parser.rs`, 나머지 docs ×7)
- **mergeable**: **MERGEABLE / BEHIND** (충돌 없음 — equation 모듈, #1203 의 section.rs 와 무간섭)
- **CI**: Build & Test / Canvas visual diff / Analyze ×3 / CodeQL 전부 SUCCESS. WASM skip.
- **라벨**: enhancement / 마일스톤 v1.0.0

## 1. 문제 (코드 확인 완료)

`samples/3-09월_교육_통합_2022.hwpx` 20쪽 등에서 일부 수식 스크립트 토큰이 literal 텍스트로
leak (문15·24·25·26·30). 전체 2207 수식 중 일부.

근본 원인(이슈/PR 진단 — 코드 확인):
- **A** `root3` 등 root/sqrt 가 숫자 피연산자에 붙으면 한 토큰 → leak (√3 미렌더). tokenizer
  glued 분리 목록(`bold/it/rm/times/sim`)에 root/sqrt 없음.
- **B** `rm bar {...}` 의 bar(overline)가 글꼴 명령 body 에서 Text 로 소비 → leak. parser
  `parse_single_or_group` 의 Command 분기가 symbol/function 외를 Text 처리.
- **C** `primeF` prime 이 글자에 붙으면 leak.

## 2. 수정 내용 — ⚠️ PR 본문보다 범위가 넓음 (정밀 검토)

PR 본문은 "A/B/C 최소 수정"으로 기술했으나 **실제 diff 는 광범위**:

| 항목 | 본문 | 실제 변경 |
|------|------|-----------|
| A root/sqrt digit-split | ○ | ○ **+ 관계연산자 GEQ/LEQ/GE/LE 도 digit-split 추가**(본문 미고지) |
| B parser bar leak | ○ | ○ (parse_command 위임 + 대소문자 fallback) |
| C prime split | ○ | ○ |
| **E** | **미고지** | **신규**: over/atop 를 **숫자뿐 아니라 ≤2자 글자 피연산자**에도 분리(`overa^2`), 그리고 **범용 `longest_keyword_prefix`** 도입 — GLUE_SAFE allowlist 키워드가 글자에 붙으면 분리(`tanx`→tan x, `barMH`→bar MH, `trianglePQR`→△ PQR) |
| 글꼴 대문자 | 미고지 | RM/IT/BOLD 대문자 추가 |

`#1204-E` 가 핵심 신규 메커니즘. **무분별한 일반화가 아니라 GLUE_SAFE allowlist**(sin/cos/tan…,
bar/vec/hat…, leq/geq/triangle… 명시 목록)로 제한 + whole-keyword 제외(`is_eq_keyword`) +
longest-match(`cosh`≠cos+h) + 모호 prefix 제외(greek `alphabet`, over `overlap`, root `rootn`,
arg `argmax`, 2자 `ge/le`). 방향은 `feedback_hancom_compat_specific_over_general`(케이스별 명시
가드) 와 정합.

## 3. 위험 평가 — 오분리(over-split) 가능성 탐침

allowlist glued-split 의 본질적 위험: **함수/키워드로 시작하는 일반 식별자**가 쪼개짐.
직접 탐침(tokenize 동작):

| 입력 | 결과 | 비고 |
|------|------|------|
| `secant` | `sec`+`ant` | 함수 prefix 변수면 오분리 (이론적) |
| `tank` | `tan`+`k` | hwpeq 전제(`tanx`→tan x)면 정상, 변수 `tank`면 오분리 |
| `barx` | `bar`+`x` | 동일 |
| `overa` | `over`+`a` | ≤2자 글자 분리 (Task#1122 는 숫자만이었음) |

→ **추정으로 단정 불가**(`feedback_no_inference_authoritative_spec`). 실제 한컴 hwpeq 가
다글자 변수를 공백 없이 쓰는지 여부에 달림. 따라서 **실제 문서 23페이지 정량 회귀로 판정**.

## 4. 검증 결과 (로컬 머지 시뮬 `pr1208-verify`)

| 단계 | 결과 |
|------|------|
| merge | ✅ CLEAN (equation 모듈, 충돌 0) |
| fmt / build | ✅ |
| 전체 테스트 `cargo test --tests` | ✅ **1906 passed, 0 failed** (수식 회귀 9건 추가) |
| equation lib 테스트 | ✅ 136 passed |
| **20쪽 leak 정량** | ✅ `root3` 7→0, `bar` 5→0, `prime` 1→0 (PR 본문 일치) |
| **전체 23쪽 텍스트 diff (보정 전↔후)** | ✅ 변화 13페이지 — **전부 leak→정상 렌더(개선)** |
| **오분리 회귀** | ✅ **0건** (아래) |

전체 23페이지 보정 전/후 텍스트 비교 — 사라진 것은 모두 leak 토큰, 새로 생긴 것은 정상 기호:
- `sintheta`→`sin θ`, `sin2theta`→`sin 2θ`, `cosx`→`cos x` (함수)
- `angleOPC`→`∠ OPC`, `triangleDAE`→`△ DAE` (도형)
- `GEQ5`/`GE0`→`≥ 5`/`≥ 0` (관계연산자), `root3`/`root5`→`√3`/`√5`, `vec`/`bar`→장식, `primeF`→`′F`
- **탐침 우려 케이스(`secant`/`tank` 류 오분리)는 23페이지 실문서 어디에도 없음** → 이 문서의
  실제 hwpeq 스크립트는 PR 전제(키워드+피연산자 glued)대로 작성됨.

산출물: `output/poc/pr1208/{before_all,after_all}/` (23쪽 ×2), `after_/...020.svg`.
정답지: `pdf/3-09월_교육_통합_2022.pdf` 20쪽 (한글 2022).

## 5. 판단 — 머지 권고 (시각 판정 게이트 + 본문 범위 고지 권고)

- 진단 정확, B 는 좁고 정확, A/C/E 는 allowlist 로 제한된 케이스별 가드.
- 실문서 23페이지 정량 회귀 0, 전체 테스트 1906 passed, leak 13페이지 전부 개선.
- 다만 **PR 본문이 #1204-E(범용 glued-prefix)·관계연산자·대문자 변형을 고지하지 않음** —
  실제로는 수식 토크나이저의 일반 동작을 바꾸는 변경. 컨트리뷰터에 본문 범위 정정 요청.
- 수식은 한컴 핵심 경로이고 자체 검증 ≠ 한컴 호환(`feedback_self_verification_not_hancom`) →
  **작업지시자 직접 시각 판정**(20쪽 √·overline·∠, PDF 대조)을 게이트로.
- 승인 + 시각 판정 통과 시 메인테이너 로컬 `--no-ff` 머지 + push.

## 6. 메인테이너 추가 보정 — cdots 연접 leak

작업지시자 지적: `cdotscdots`(⋯⋯, 두 생략기호 공백 없이 연접)가 보정 후에도 잔존(14·18·19·20쪽).
PR #1208 이 다루지 않은 별개 leak — GLUE_SAFE allowlist 에 dots 계열 미포함이라 분리 안 됨
(`lookup_symbol` 은 대소문자 무시라 `cdots` 단독은 이미 ⋯ 인식, 연접만 leak).

메인테이너 보정(머지에 포함): `GLUE_SAFE` 에 `cdots`/`ldots`/`vdots`/`ddots` 추가
(5자 명시 키워드만; 모호 4자 `dots` 는 일반 변수 충돌 우려로 제외 — 케이스별 가드 취지).
회귀 테스트 `test_dots_glued_split` 추가.

| 항목 | 결과 |
|------|------|
| `cdotscdots` 분리 | ✅ `["cdots","cdots"]` |
| 전체 23쪽 `cdotscdots` leak | ✅ 0 (보정 전 14·18·19·20쪽 → 0) |
| 14쪽 `⋯` 정상 렌더 | ✅ cdots 잔여 0, ⋯ 2개 |
| 전체 테스트 (cdots 포함) | ✅ **1907 passed, 0 failed** |

## 7. 비고

- GLUE_SAFE 가 향후 다글자 변수와 충돌하는 새 샘플이 나오면 allowlist 조정 필요(현재 실문서 무회귀).
- #1199/#1200 과 무관한 별개 사안(PR 본문대로).
