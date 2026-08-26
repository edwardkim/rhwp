---
kind: working-note
status: completed
issue: 4962
stage: W4-1
last_verified: 2026-08-22
---

# Task M100 #4962 W4 Stage 1 — 조판 위험 계약·호환 projection·RED

- **이슈**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **계획**: [`task_m100_4962_w4.md`](../plans/task_m100_4962_w4.md)
- **선행 계획 commit**: `91149b634`
- **브랜치**: `task_m100_4962`
- **단계 상태**: W4-1 완료, W4-2 승인 대기

## 1. 결론

W3가 보존한 aggregate만으로 W4 조판 위험 순위를 계산할 수 있도록 입력 동결값, usage row 필드,
document-face identity, fixed-frame context proxy, stored/fresh LineSeg lane, 같은 행의 risk mass와 privacy
경계를 machine-readable 계약과 JSON Schema로 고정했다.

공개 fixture 기반 ranker test를 먼저 작성해 구현 모듈이 아직 없다는 정확한 이유로 RED를 확인했다. W4-1은
ranker를 구현하거나 private 10k를 다시 실행하지 않는다. metric DB·fallback·renderer·font asset도
변경하지 않았다.

## 2. 입력 호환성 동결

### 2.1 W3 결정성 쌍

보존된 r2와 r3는 다음 값이 exact다.

| 항목 | r2 | r3 | 판정 |
| --- | ---: | ---: | --- |
| file mode | `0600` | `0600` | exact |
| JSON bytes | 110,097,106 | 110,097,106 | exact |
| file SHA-256 | `24eb1d15…49352a1` | `24eb1d15…49352a1` | exact |
| aggregate SHA-256 | `37867105…bc2888` | `37867105…bc2888` | exact |
| source | `c1ec759f9` | `c1ec759f9` | exact |

계약은 primary를 r2 하나로 지정하고 r3는 결정성 peer로만 보존한다. W4 ranker가 두 파일을 합산하거나
새 모집단으로 해석해서는 안 된다.

### 2.2 최신 devel ingress

기존 Stage 4-B 32건과 최신 devel 통합 뒤 같은 32건은 모두 32/32 성공했고 failure·count·category·join,
legacy/decision usage row가 exact였다. `aggregateHash`와 `checkpoint` 실행 계보를 제외한 의미 본문은 양쪽
모두 SHA-256 `ba0d69bb…f3f43d47`이다. source와 worker가 바뀌어야 달라지는 실행 identity를 semantic
drift로 오인하지 않도록 이 projection을 계약에 명시했다.

### 2.3 historical POC 정의

commit `631287d4708f144011162179d61f8272cf072ff6`의
`examples/poc_font_layout_habits.rs` 원문 SHA-256은 `76d394de…a41b62e`다. `fixed_frame_context`와
`analyze_paragraph`에서 다음 정의를 다시 확인하고 계약에 동결했다.

- compressed: `ratio < 100 || spacing < 0`
- extreme compressed: `ratio <= 90 || spacing <= -5`
- fixed context: table cell, text box, caption, header, footer, master page bit 중 하나 이상
- stored LineSeg: 비어 있지 않고 모든 항목이 missing placeholder는 아닌 상태

fixed context는 geometry가 아니므로 출력 이름을 `fixedFrameContextProxy`로 제한했다. stored LineSeg도
유효성·최신성 판정이 아니므로 두 lane을 나누는 데만 사용한다.

## 3. 고정한 계약

| 경계 | 계약 |
| --- | --- |
| risk category | `char-miss`, `face-miss`, `heuristic`만 포함 |
| candidate identity | exact document `font`; format·language·style로 identity를 쪼개지 않음 |
| cause cluster | `metricRequestedFace`; document face를 합치지 않음 |
| risk mass | 같은 row의 장평·자간 indicator와 fixed-context proxy만 반영 |
| category weight | 없음 |
| LineSeg | stored/fresh lane별 mass를 분리하되 multiplier 없음 |
| 분모 | risk category 합, category별 합, lane mass 합, HWP+HWPX 가산성을 독립 대사 |
| 문서 영향도 | usage row `documentCount`를 face별 affected document로 합산하지 않음 |
| privacy | path·filename·문서 hash·raw row·문자·token·stack을 공개 projection에서 거부 |

fixture의 동일 행 계산은 다음 기대값으로 잠갔다.

| candidate | risk chars | stored mass | fresh mass | base mass |
| --- | ---: | ---: | ---: | ---: |
| Face A | 17 | 18 | 50 | 68 |
| Face B | 20 | 0 | 20 | 20 |
| 합계 | 37 | 18 | 70 | 88 |

Face A의 `table-cell`, 장평 90, 자간 -5인 5자는 compression factor 5와 proxy factor 2를 같은 row에서
적용해 mass 50이 된다. `footnote+header`는 허용 token `header`가 있으므로 proxy가 참이지만 단순
`footnote`만으로는 참이 아니다.

## 4. RED와 검증

### 4.1 의도된 RED

```bash
node --test scripts/tests/font_typesetting_risk_rank.test.mjs
```

결과:

```text
Error [ERR_MODULE_NOT_FOUND]: Cannot find module
'scripts/font_typesetting_risk_rank.mjs'
tests 1, pass 0, fail 1
```

이는 제품 결함이 아니라 Stage W4-2 구현 대상이 아직 없음을 확인한 RED다. test는 contract JSON 외에
private aggregate를 읽지 않으며, 같은 행 risk mass·document face/metric cluster 분리·format 가산성·row
order 결정성·입력 drift·민감정보 실패 경계를 요구한다.

### 4.2 계약·회귀 검증

| 검증 | 결과 |
| --- | --- |
| JSON 두 파일 parse | 통과 |
| JSON Schema Draft 2020-12 Ajv strict compile·instance validate | 통과 |
| 신규 test syntax | 통과 |
| W3 contract·checkpoint finalizer focused test | 12/12 통과 |
| `cargo fmt --all` + `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |

첫 strict compile은 tuple schema에 고정 길이가 빠진 작성 결함을 검출했다. 각 tuple에 동일한
`minItems`·`maxItems`를 명시한 뒤 strict compile과 instance validation을 다시 통과시켰다.

## 5. 보호 불변식

- private 10k 재실행 없음
- r2/r3와 기존 POC를 덮어쓰거나 권한 변경하지 않음
- corpus path·filename·document hash·raw 문자 공개 없음
- POC 9,948건과 W3 9,909건 직접 join 없음
- fixed-context proxy를 geometry나 overflow로 승격하지 않음
- stored LineSeg를 valid/invalid로 부르지 않음
- source·Cargo·WASM·Studio 제품 변경 없음
- 원격 push·PR·GitHub 본문 변경 없음

## 6. 다음 승인 지점

Stage W4-2는 이 RED를 GREEN으로 만드는 최소 결정론적 streaming ranker를 구현한다. 보존된 W3 r2 한 건만
읽어 document-face와 metric-request cluster를 집계하고, 위험 문자 2,061,732자와 category·lane·format
분모를 대사한 뒤 local 공개 projection을 두 번 생성해 bytes/hash exact를 확인한다.

메인테이너 승인 전에는 W4-2 구현, 110 MB 실제 입력 실행, ranking 공표, W4-3 evidence promotion,
원격 push와 PR을 시작하지 않는다.
