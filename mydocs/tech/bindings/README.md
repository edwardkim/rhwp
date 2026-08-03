---
kind: guide
status: active
canonical: mydocs/tech/bindings/README.md
last_verified: 2026-08-03
---

# 외부 바인딩 문서 지도 — 동등성 계약 축

`mydocs/tech/bindings/` 는 rhwp 의 **외부 언어 바인딩들이 서로 어긋나지 않게 하는 계약**을
보존한다. 바인딩 하나하나의 사용법이 아니라 **바인딩들 사이**를 다룬다.

로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M18~M20 축의 전제는
"같은 문서를 물으면 언어와 무관하게 같은 답이 나온다"이다. 바인딩이 하나일 때는 자명하고,
둘이 되는 순간 **아무도 강제하지 않으면 깨진다.** 실제로 깨져 있다 — 확인된 표류가
20건이다.

## 이 디렉터리의 문서

| 문서 | kind | 무엇을 답하나 |
|---|---|---|
| [`parity_contract.md`](parity_contract.md) | canonical | **새 바인딩이 지켜야 할 것.** 봉투 이름 규약·판정 vs 실패·버전 정합·노출 범위·강제 테스트 설계 |
| [`new_binding_guide.md`](new_binding_guide.md) | guide | **C#/Swift 를 추가하려는 사람이 읽는 12단계.** 기존 두 구현을 코드 경로로 인용 |
| [`python_node_comparison.md`](python_node_comparison.md) | reference | **현행 두 바인딩의 실측 차이 20건**과 의도/표류 판정 |

## 이 디렉터리와 다른 문서의 관계

```
bindings_foundation.md  (../)          ← 설계 전제: 왜 서브프로세스인가, IR 스키마 버저닝
        │                                (M18~M20 공통 기반, #3142 RFC 의 실행편)
        ▼
parity_contract.md  (여기, canonical)  ← 둘 이상이 됐을 때 무엇을 같게 유지하나
        ├── new_binding_guide.md       ← 그 계약을 새 언어에 적용하는 절차
        └── python_node_comparison.md  ← 그 계약에 비춘 현행 스냅샷
                │
                ▼
manual/python_binding_guide.md         ← 언어별 사용법 (이 축 밖)
manual/node_binding_guide.md
bindings/*/docs/DESIGN.md              ← 언어별 결정 기록 (버린 대안 포함)
```

**중복을 피하는 경계:**

- [`bindings_foundation.md`](../bindings_foundation.md) 는 **왜 이 표면인가**를 답한다.
  표면 판단 매트릭스(CLI 서브프로세스 / 장수명 서버 / C ABI / WASM), IR 스키마 버저닝
  전략, 마일스톤별 착수 조건이 거기 있다. 이 디렉터리는 그걸 다시 쓰지 않는다.
- 이 디렉터리는 **둘 이상일 때 무엇을 같게 유지하나**를 답한다.
- 언어별 사용법과 결정 기록은 `mydocs/manual/` 과 `bindings/*/docs/` 에 있다.

## 언제 무엇을 읽나

| 상황 | 읽을 것 |
|---|---|
| 새 언어 바인딩을 만든다 | [`new_binding_guide.md`](new_binding_guide.md) 를 순서대로. 각 단계가 [`parity_contract.md`](parity_contract.md) 의 해당 절을 가리킨다 |
| 기존 바인딩에 명령·옵션을 추가한다 | [`parity_contract.md`](parity_contract.md) §5(노출 범위) → 다른 바인딩에도 같이 넣었는지 확인 |
| 두 바인딩이 다르게 동작한다는 제보를 받았다 | [`python_node_comparison.md`](python_node_comparison.md) 에 이미 있는지 먼저 본다 |
| 오류·예외를 새로 만든다 | [`parity_contract.md`](parity_contract.md) §3.3 매핑표 — 이름은 언어별로 정하지 않는다 |
| 봉투에 필드를 더한다(본체 쪽) | [`parity_contract.md`](parity_contract.md) §2 — camelCase 이고, 바인딩은 자동으로 따라온다 |
| CI 에 패리티 가드를 붙인다 | [`parity_contract.md`](parity_contract.md) §6 의 A·B·C 3층 |

## 동등성을 강제하는 3층 (요약)

계약을 문서로만 두면 지켜지지 않는다. 표류 20건이 그 증거다. 설계 전문은
[`parity_contract.md`](parity_contract.md) §6.

| 층 | 무엇을 대조하나 | 현재 |
|---|---|---|
| **A** 자기서술 대조 | `capabilities` ↔ 바인딩의 공개 표면 (선언→래퍼, 래퍼→선언, 옵션→플래그, 종료 코드 사전) | Node 에 2개, 파이썬에 1개. **A-1 이 파이썬에 없는 것이 표류 12건의 원인** |
| **B** 교차 실행 골든 | 같은 케이스를 CLI·각 바인딩으로 돌려 **봉투 원문 바이트 비교** | 없음 (신규 설계) |
| **C** 오류·판정 매핑 | 9개 실패 상황의 (종료 코드, 예외 계열, **예외인가 값인가**) | 파이썬에 절반 |

A층만으로는 부족하다 — 각 바인딩이 *혼자서* 도구와 어긋나지 않았는지만 보므로, 두
바인딩이 서로 다른 방식으로 어긋나 있으면 둘 다 통과시킨다. 그래서 B층이 있다.

## 현재 상태 (2026-08-03 실측)

| 마일스톤 | 언어 | 디렉터리 | 소스 | 표면 |
|---|---|---|---|---|
| M18 | Python | `bindings/python` | `src/rhwp` 11파일 3,148줄 | CLI 서브프로세스 — 1층 28명령 + 세션 + 계획 |
| M19 | Node/TS | `bindings/node` | `src` 14파일 5,377줄 | 동일 + 생성 타입 + 브라우저(WASM) 어댑터 |
| M20 | C#/Swift | `bindings/csharp`·`swift`·`Native` | 아래 참조 | **미착수** |

### M20 자리에 이미 다른 것이 있다

`bindings/csharp`·`bindings/swift`·`bindings/Native` 는 **비어 있지 않다.**

```
bindings/Native/src/lib.rs                        376줄   cdylib, C ABI
bindings/csharp/RhwpNative.cs                      63줄   P/Invoke
bindings/swift/Sources/Rhwp/*.swift                274줄
bindings/swift/Sources/CRhwpNative/*.h             17줄
```

노출 함수는 넷뿐이다 — `rhwp_export_text`, `rhwp_export_markdown`, `rhwp_read_text`,
`rhwp_string_free`. 봉투는 `{"ok":true,"pageCount":N,"files":[…]}` 형태로
**`schemaVersion` 도 종료 코드도 판정 표현도 없다.**

`bindings/README.md:5-9` 가 이 계열을 "Native ABI" 로, python·node 를 "CLI subprocess
bindings" 로 명시적으로 갈라 놓았다. **즉 M20(서브프로세스 계열의 C#/Swift)이 미착수인
것은 맞지만, 그 디렉터리 이름은 이미 다른 계약이 쓰고 있다.**

M20 이 먼저 답해야 할 세 질문과 그때까지의 최소 방어는
[`parity_contract.md`](parity_contract.md) §8 에 있다.

## 핵심 결정 세 줄 요약

세 결정의 근거와 버린 대안은 [`parity_contract.md`](parity_contract.md) 본문에 있다.

1. **봉투 원문 키는 어떤 바인딩도 바꾸지 않는다.** 언어 관례(snake_case 등)는 *별칭 조회
   계층*으로만 제공하고, 직렬화 가능한 `.raw` 는 CLI 가 낸 그대로 보존한다. 기존 두
   바인딩이 이미 그렇게 되어 있고, 일괄 변환 함수는 만들어 두었지만 내부에서 한 번도
   호출하지 않는다(§2).
2. **exit 3/4 는 판정이라 반환값, exit 1/2 는 고장이라 예외.** opt-in 스위치로만 3/4 를
   예외로 올릴 수 있고 기본값은 반드시 거짓. 계획 선검증 위반(exit 2 + `invalid`)만이
   유일한 정당한 예외의 예외다(§3).
3. **버전은 semver 로 묻지 않고 `capabilities` 로 기능을 묻는다.** 봉투 `schemaVersion`
   major 불일치는 거부, minor·필드 추가는 통과, 바이너리 semver 는 확인하지 않는다.
   버전 범위 표를 바인딩마다 들고 있으면 그게 곧 수기 매핑이고 언어 수만큼 복제된다(§4).

## 지금 가장 급한 일

[`python_node_comparison.md`](python_node_comparison.md) 의 표류 20건 중 우선순위:

| 순위 | 항목 | 왜 |
|---|---|---|
| 1 | **D-19** 파이썬에 "선언 → 래퍼" 패리티 테스트 추가 | D-2·D-12 의 구조적 원인. 이걸 먼저 넣어야 나머지가 재발하지 않는다 |
| 2 | **D-1** 파이썬 `convert`/`export_hwpx` 의 `-o` → 위치 인자 | 두 API 가 **항상 exit 2** 로 죽어 있다(실행으로 확인) |
| 3 | **D-4** 파이썬 `Plan.check()` 에 `--dry-run` 게이트 추가 | 검사인 줄 알고 문서가 편집될 수 있다 |
| 4 | **D-9** `VerifyReport` 진리값 의미 통일 | 같아 보이는 코드가 언어마다 반대 결론을 낸다 |
| 5 | **D-2·D-12** 파이썬 `render_diff` 와 누락 옵션 4건 | 옵션이 없으면 기능이 없는 것과 같다 |

## 이 축이 드러낸 본체 쪽 결함 (실측)

바인딩 동등성을 계약으로 못 박으면 본체의 어긋남도 함께 드러난다. 둘 다 별도 이슈감이다.

| 결함 | 실측 | 영향 |
|---|---|---|
| `export-structure --json` 의 `structure.node_count` 가 snake_case | 최상위는 `nodeCount`. 봉투 전체에서 `_` 가 든 키는 이것 하나 | 별칭 계층이 없는 언어(정적 매핑)에서 필드가 사라진다 |
| `export-tables -o --json` 이 봉투 대신 사람 문장을 낸다 | `표 추출 완료: 12개 → …` | `--json` 봉투 계약 전체의 예외. 바인딩이 옵션을 닫은 것은 회피일 뿐 수정이 아니다 |

## 실측 조건

이 디렉터리의 모든 수치·동작 주장은 아래에서 직접 실행해 얻었다. 근거를 대지 못하는
항목은 각 문서에 **확인되지 않음**으로 표시했다.

- 바이너리: `target/release/rhwp.exe`, `rhwp v0.8.2`, 봉투 `schemaVersion 1.0`
- `capabilities`: 명령 61개(그중 `json:true` 31개), 범주 분포
  `diagnostic 25 / export 18 / query 8 / internal 5 / edit 3 / serve 1 / batch 1`
- 표본 문서: `samples/2010-01-06.hwp` (hwp5, 6쪽, 87문단)
- 파이썬 바인딩: `bindings/python/src` 를 `PYTHONPATH` 로 임포트해 **실행 검증**
- Node 바인딩: **소스 정적 대조만** — 이 PC 에 `node_modules` 가 없어 `vitest`·`tsc` 미실행
- 실행일: 2026-08-03

## 관련 문서

- [`bindings_foundation.md`](../bindings_foundation.md) — M18~M20 공통 기반 (설계 전제)
- [`tech 문서 지도`](../README.md) — 상위 지도
- [`python_binding_guide.md`](../../manual/python_binding_guide.md) — 파이썬 사용법
- [`node_binding_guide.md`](../../manual/node_binding_guide.md) — Node 사용법
- [`agent_surface_playbook.md`](../../manual/agent_surface_playbook.md) — 표면 추가 절차
- [`cli_commands.md`](../../manual/cli_commands.md) — CLI 명령 레퍼런스
- 로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608) M18~M20
