# 최종 결과보고서 — Task M100 #1204

**이슈**: [#1204](https://github.com/edwardkim/rhwp/issues/1204) HWPX 수식 스크립트 토큰 처리 — root/sqrt glued, rm+bar, prime glued
**마일스톤**: v1.0.0 (M100)
**브랜치**: `local/task1204` (← `stream/devel`, 독립)
**완료일**: 2026-06-01

---

## 1. 문제

`samples/3-09월_교육_통합_2022.hwpx` 20쪽 등에서 특정 수식이 렌더 안 되고 스크립트 토큰이 literal 텍스트로 출력 (문15·24·25·26·30). 전체 수식 2207개 중 일부.

## 2. 근본 원인 (실제 스크립트로 확정)

| 버그 | 내용 | 영향 |
|------|------|------|
| **A** | `tokenizer` glued-keyword 분리 목록에 `root`/`sqrt` 및 관계연산자(`GEQ`/`LEQ`/`GE`/`LE`) 미포함 → `root3`/`GEQ5`/`GE0` 한 토큰 → literal leak | 21+9 scripts |
| **B** | `parser.parse_single_or_group` 가 symbol/function 외 명령을 `Text` 처리 → `rm` body 의 `bar` leak | 28 scripts |
| **C** | `prime` 이 글자에 붙으면(`primeF`) 미분리 → leak | 소수 |
| **D** | `DECORATIONS`/`FONT_STYLES` lookup 이 대소문자 구분(소문자 키) → 대문자 명령 `RM` 등 miss → leak (`3-11월_실전_통합_2022.hwpx`) | 3 scripts |
| **E** | 키워드가 **글자**에 붙은 경우(`tanx`,`barMH`,`LEQb`,`trianglePQR`,`rmbarFF`) 미분리 → leak. digit-guard(A) 는 숫자만 처리 | 11월 다수 |
| **F** | 집합연산 `cap`/`cup` glued(`capB`→A∩B), `over`/`atop` 가 짧은 글자 분모에 붙음(`overa^2`→분수) 미분리 → leak | 11월 다수 |

## 3. 수정

`src/renderer/equation/`:
- **A**: `tokenizer.rs` — `root`/`sqrt`(+대문자) 및 관계연산자 `GEQ`/`LEQ`/`GE`/`LE` 가 **숫자**에 붙으면 분리 (digit-guard, letter 충돌 회피; GEQ/LEQ 를 GE/LE 보다 먼저 검사).
- **B**: `parser.rs` — `parse_single_or_group` Command 분기를 `parse_command` 재귀로 (fall-through Text 안전).
- **C**: `tokenizer.rs` — `prime`(+대문자) 가 alnum 에 붙으면 분리.
- **D**: `parser.rs` — `DECORATIONS`/`FONT_STYLES` lookup 에 소문자 fallback (hwpeq 대소문자 무시) → `RM`/`BAR` 등 대문자 변형 인식. `tokenizer.rs` glued-split 에 `RM`/`IT`/`BOLD` 대문자 추가.
- **E**: `tokenizer.rs` — glued run 에서 **allowlist 키워드의 최장 prefix** 분리 (함수 sin/cos/tan/sinh.., 장식 bar/vec/.., 관계 leq/geq, 도형 triangle/angle, 집합 cap/cup). chain(`rmbarFF`→rm bar FF) 처리. **회귀 가드**: greek(alphabet)·root(rootn)·arg(argmax) 는 allowlist 제외 — over-split 방지.
- **F**: `tokenizer.rs` — `over`/`atop` 가 **짧은(≤2자) 글자 분모**(`overa^2`,`overdx`,`overy`)에 붙으면 분리(분수). 긴 word(≥3, `overlap`)·keyword(`overline`/`overset`)는 유지 → #1122 보존.

코드 + 회귀 테스트 9, 모델/레이아웃/타 모듈 무변경.

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | 성공 |
| `cargo test --release` (전체) | **1902 passed / 0 failed** |
| 수식 모듈 테스트 | 통과 (신규 9건 — split·over-split 가드 포함) |
| **전수 sweep — 09월(2207) + 11월(1723) 스크립트** | 양쪽 모두 leak **0** (함수명·점라벨 false-positive 제외) |
| over-split 회귀 가드 | alphabet/overlap/overline/overset/argmax/rootn/cosh 모두 유지 |
| 한글 2022 PDF 대조 | 09월: 문15(≥)·24·25(overline·√)·26(vec·√)·21쪽(overline) / 11월: 14쪽(∩∪)·19쪽(분수)·21쪽(overline·△·∠)·지수 시각 정합 |
| rustfmt (변경 파일) | clean |

## 5. 산출물

- 소스: `src/renderer/equation/tokenizer.rs`, `parser.rs`
- 계획: `mydocs/plans/task_m100_1204{,_impl}.md`
- 단계 보고: `mydocs/working/task_m100_1204_stage{1,2,3,4}.md`
- 최종 보고: 본 문서
