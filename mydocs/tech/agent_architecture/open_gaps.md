---
kind: reference
status: active
canonical: mydocs/tech/agent_architecture/layer_model.md
last_verified: 2026-08-12
---

# 미해결 공백 대장

> **v0.8.4 현행성 주의:** Python·Node 바인딩 관련 공백은 #4655의 철회 결정으로
> 닫혔다. 관련 경로와 테스트 언급은 당시 진단 기록이며 현재 백로그가 아니다.

> 오늘까지 여러 이슈·PR·조사 문서에서 나온 **미해결 항목을 한 곳에** 모은다.
> 흩어져 있으면 두 번 조사하거나 영영 잊힌다.
>
> **남의 보고를 그대로 옮기지 않았다.** 각 항목을 이 PC 에서 직접 재현했고,
> 재현되지 않은 것은 [§9](#9-재현-실패--확인되지-않음) 에 **재현 실패**로 적었다.

관련 — 규칙 [`invariants.md`](invariants.md) · 결정 [`decision_log.md`](decision_log.md) · 층
[`layer_model.md`](layer_model.md) · 로드맵 [`roadmap_atlas.md`](roadmap_atlas.md) · 지도
[`README.md`](README.md) · 보안 [`agent_security/`](../agent_security/README.md) · 상위 이슈
[#3880](https://github.com/edwardkim/rhwp/issues/3880).

---

## 0. 읽는 법

**항목 형식** — *증상* 무엇이 잘못 보이는가 / *재현* 이 PC 에서 실제로 돌린 명령과 출력 (못 돌렸으면 **재현 실패**) / *층* L1 표면·L2 도구·L3
세션·L4 계획·바인딩·인프라·문서 / *막는 것* 이 공백 때문에 못 하는 일 / *이슈* 번호, 없으면 **"없다"**.

**측정 환경** — `rhwp v0.8.2`(`<저장소>/target/release/rhwp.exe`) · 2026-08-03 · 명령 61 · `--json` 31 ·
MCP 무상태 39 · 세션 12(서버 총 노출 51). 이 바이너리의 커밋은 **확인되지 않음**([§9](#9-재현-실패--확인되지-않음)).

**이 문서에 적지 않는 것** — 미제출 취약점의 위치·재현·패치. `SECURITY.md` 가 공개 등록을 금지한다([`decision_log.md`
D-20](decision_log.md#d-20--취약점은-pr-이-아니라-비공개-경로로-간다)). 그런 항목이 있다는 사실만 [§7](#7-인프라--운영) 끝에 적는다.

---

## 1. 공백 색인

| ID | 증상 | 층 | 재현 | 이슈 |
| --- | --- | --- | --- | --- |
| [G-01](#g-01--dump--diag--core-pages-가---json-과-미지-옵션을-침묵-무시한다) | 진단 3종이 `--json`·미지 옵션 침묵 무시 | L1 | **O** | #3880 T2 |
| [G-02](#g-02--run-의-실패-경로-예외를-자기서술이-적지-않는다) | `run` 예외를 `jsonContract.failure` 가 안 적음 | L1 | **O** | #3880 T4 |
| [G-03](#g-03--info---json-에-warnings-가-없다) | `info --json` 에 `warnings` 없음 | L1 | **O** | #3880 T1 · PR #3882 |
| [G-04](#g-04--봉투에-snake_case-키가-하나-남아-있다) | `structure.node_count` | L1 | **O** | #3880 T3 · PR #3882 |
| [G-05](#g-05--같은-논리-오류가-진입로마다-다른-판정을-받는다) | 없는 필드: `edit` exit 0+파일 생성 / `run` exit 2 | L1·L4 | **O** | 없다 |
| [G-06](#g-06---o-와---json-의-우선순위가-명령마다-다르다) | 같은 플래그 조합이 세 가지 뜻 | L1 | **O** | 없다 |
| [G-07](#g-07--render-diff---json-은-이미-있는데-현황판이-잔여로-둔다) | 현황판 드리프트 | L1 | **O** | #3719 §3-1 |
| [G-08](#g-08--export-png---json-게이트-정합) | feature 게이트 봉투 미정합 | L1 | 부분 | #3357 |
| [G-09](#g-09--무상태-도구가-어느-프로필에도-없을-수-있다) | 프로필 등재 가드 미머지 | L2 | 간접 | PR #3838 |
| [G-10](#g-10--무상태-hwp_search-에-상한이-없다) | 컨텍스트 상한 미적용 | L2 | **O** | 없다 |
| [G-11](#g-11--l6-매크로가-멈춰-있고-판정-1건이-남아-있다) | `hwp_form_autopilot` 미판정 | L6 | — | #3719 §6 |
| [G-12](#g-12--세션-도구에-undoredo-가-없다) | `hwp_doc_undo/redo` 부재 | L3 | **O** | #3719 §3-3 |
| [G-13](#g-13--sessiondoc-에-원본-경로가-없다) | 감시·중복제거·재적재가 막힘 | L3 | 코드 | PR #3878 |
| [G-14](#g-14--세션-저장이-원자-교체를-쓰지-않는다) | CLI 와 비대칭 | L3 | 코드 | PR #3878 |
| [G-15](#g-15--hwp_doc_text-를-page-없이-부르면-여는-것보다-비싸다) | 393쪽 374 ms | L3 | 인용 | PR #3878 |
| [G-16](#g-16--계획-스키마에-dryrun-과-preview-skipped-가-빠져-있다) | 전제 소멸 후 미반영 | L4 | **O** | PR #3808 |
| [G-17](#g-17--파이썬-바인딩의-치명-결함-3건) | `convert(out=)` 항상 실패 등 | 바인딩 | **O**(1건) | PR #3879 |
| [G-18](#g-18--csharpswiftnative-봉투에-판정-어휘가-없다) | `schemaVersion`·종료코드 없음 | 바인딩 | 코드 | #3608 M20 |
| [G-19](#g-19--batch-fill-이-행마다-서식을-재파싱한다) | 137 ms/행 | 인프라 | 인용 | PR #3878 |
| [G-20](#g-20--퍼징이-ci-에서-돌지-않는다) | `grep -ril fuzz .github/` = 0 | 인프라 | **O** | PR #3877 |
| [G-21](#g-21--fuzzregressions-가-규정돼-있는데-없다) | README 규정 ↔ 부재 | 인프라 | **O** | PR #3877 |
| [G-22](#g-22--모델-미탑재를-강제하는-장치가-없다) | 의존성 허용목록 부재 | 인프라 | **O** | 없다 |
| [G-23](#g-23--이-pc-에서-rhwp-를-빌드할-수-없다) | 계약 테스트 실행 불가 | 인프라 | **O** | 없다 |
| [G-24](#g-24--레시피-03pii-마스킹이-비어-있다) | 상호 참조만 있고 본문 없음 | 문서 | **O** | PR #3835 |
| [G-25](#g-25--동시-열린-pr-이-22건이다) | 볼륨 규약 위반 | 운영 | **O** | #3719 §7 |

이슈·PR 링크는 `https://github.com/edwardkim/rhwp/issues|pull/<번호>`.

---

## 2. L1 — 표면·봉투

### G-01 · `dump` · `diag` · `core-pages` 가 `--json` 과 미지 옵션을 침묵 무시한다

*층* L1 — *이슈* [#3880](https://github.com/edwardkim/rhwp/issues/3880) T2

**증상.** `capabilities` 에 `json`·`flags` 를 선언하지 않은 명령이 `--json` 을 받고 아무 말 없이 사람용 텍스트를 낸다. **모르는 옵션도
똑같이 무시한다.**

```
$ rhwp dump samples/field-01.hwp --json            → exit=0  stdout=18,643B  stderr=0B (사람용 텍스트)
$ rhwp dump samples/field-01.hwp --존재하지않는옵션   → exit=0  stdout=18,643B  (완전히 동일)
$ rhwp diag samples/field-01.hwp --json            → exit=0  stdout=615B
$ rhwp core-pages samples/field-01.hwp --json      → exit=0  stdout=150B
$ rhwp info samples/field-01.hwp --없는옵션          → exit=2  stdout=0B  (대조군: 제대로 거부)

$ rhwp capabilities | jq '.commands[] | select(.name=="dump")'
{"category":"diagnostic","name":"dump","summary":"문서 조판부호 구조 덤프"}   ← json·flags 키 없음
```

**왜 가드가 못 잡는가.** 검사가 **단방향**이다. `tools/agent_preflight.py:523` `check_declared_flags_real` 은 *선언한*
플래그를 넣어 보고 거부당하는지 본다(선언 → 실물). **역방향은 아무도 안 본다.** 그래서 **아무것도 선언하지 않은 명령은 검사 대상에서 사라진다** — 선언을 안 하는
것이 가드를 피하는 가장 쉬운 길이 되는 구조다.

**막는 것.** [`invariants.md` INV-09](invariants.md#inv-09--미지-옵션을-침묵-무시하지-않는다) 위반. 에이전트가 오타를 내면 성공으로
읽힌다. 그리고 [`decision_log.md` D-32](decision_log.md#d-32--진단-30종을---json-대상에서-명시적으로-제외한다) 의 "명시적
제외"가 **의도한 형태로 구현되지 않았음**을 드러낸다 — 제외의 올바른 형태는 "`--json` 을 exit 2 로 거부한다"이지 침묵 무시가 아니다.

**정책 결정이 먼저다** — 거부할 것인가, `json:false` 를 명시 선언할 것인가, 봉투를 만들 것인가.
[#3882](https://github.com/edwardkim/rhwp/pull/3882) 가 이 항목을 뺀 이유다.

---

### G-02 · `run` 의 실패 경로 예외를 자기서술이 적지 않는다

*층* L1 — *이슈* [#3880](https://github.com/edwardkim/rhwp/issues/3880) T4

**증상.** `capabilities.jsonContract.failure` 는 "단건 명령 실패 시 stdout 0바이트"라고 말하는데 `run` 은 예외다. 자기서술만
읽는 소비자가 깨진다.

**재현 — 예외의 경계까지 확정했다.**

```
$ rhwp run <입력 문서가 없는 계획> --json     → exit=1  stdout=192B  {"error":"입력을 읽을 수 없습니다 …"}
$ rhwp run <없는 필드를 채우는 계획> --json    → exit=2  stdout=382B  {"invalid":[{"step":0,"reason":…}]}
$ rhwp run                                 → exit=2  stdout=0B    ← 규약 준수
$ rhwp run <비 JSON 파일> --json             → exit=2  stdout=0B    ← 규약 준수

$ rhwp capabilities | jq -r '.jsonContract.failure'
단건 명령 실패 시 stdout 0바이트; batch 는 error 레코드 + 최종 exit 1
```

> [#3876](https://github.com/edwardkim/rhwp/pull/3876) 은 "exit 1 + 200 B",
> [#3880](https://github.com/edwardkim/rhwp/issues/3880) 은 "exit 2 + 137바이트"로 적었다.
> **둘 다 맞다 — 서로 다른 실패 경로다.** 이 문서가 경계를 확정한다:
> **계획 파싱 성공 이후의 실패는 봉투를 낸다.**

**막는 것.** [`invariants.md` INV-03](invariants.md#inv-03--실패-경로는-stdout-에-0바이트를-쓴다) 의 예외가 문서화되지 않은
상태다. [#3869](https://github.com/edwardkim/rhwp/issues/3869) 의 봉투 동등성 계약을 세울 때 **무엇을 기준으로 삼을지**를
흔든다 — WASM 표면이 이 예외를 따라야 하는가.

**고치는 방법 두 갈래.** ① 자기서술에 예외를 적는다(문구 변경 → 계약 테스트가 걸린다) ② `run` 을 규약에 맞춘다(봉투를 stderr 로 → L4 저널 계약이
깨진다). **①이 옳다** — 예외는 설계이고, 설계를 자기서술이 숨기는 것이 결함이다.

---

### G-03 · `info --json` 에 `warnings` 가 없다

*층* L1 — *이슈* [#3880](https://github.com/edwardkim/rhwp/issues/3880) T1 · **PR
[#3882](https://github.com/edwardkim/rhwp/pull/3882) 로 인플라이트**

```
$ rhwp info --json samples/field-01.hwp | jq 'has("warnings")'
false
$ rhwp capabilities | jq -r '.commands[] | select(.name=="info") | .recordFields | join(" ")'
schemaVersion source format sizeBytes version sections pageCount paraCount fonts title
                                                  ← warnings 없음. 선언도 함께 틀렸다
```

**원인 (코드 확인).** `src/main.rs:7370` `show_info()` 의 JSON 분기가 `return EXIT_OK` 로 끝나(`:7418` 부근)
`:7442` 의 `println!("warnings: …")` 에 도달하지 못한다.

**막는 것.** 리소스가 조용히 잘린 HML 문서가 **exit 0 + 완전해 보이는 봉투**를 낸다. `fonts` 가 부분 목록인데 봉투는 그렇다고 말하지 않는다 —
[`invariants.md` INV-05](invariants.md#inv-05--부분-목록을-내지-않는다--확신-없으면-null) 위반. 보안 축이 "봉투를 믿어라"라고
말하는 근거를 약하게 만든다.

**수정 후에도 남는 한계.** 경고 원천은 **HML 파서 하나**다. 배열이 비었다고 "문서가 온전하다"는 뜻이 아니며, #3882 가 그 사실을 코드 주석에 남겼다.

---

### G-04 · 봉투에 `snake_case` 키가 하나 남아 있다

*층* L1 — *이슈* [#3880](https://github.com/edwardkim/rhwp/issues/3880) T3 · **PR
[#3882](https://github.com/edwardkim/rhwp/pull/3882) 로 인플라이트**

```
$ rhwp export-structure samples/field-01.hwp --json    (전체 재귀 순회)
top keys:  mode, nodeCount, schemaVersion, source, structure, untrustedContent, untrustedFields
snake_case 키: ['.structure.node_count']        ← 이것 하나
$ rhwp info --json samples/field-01.hwp          snake_case 키: []
```

**막는 것.** 별칭 조회 계층이 없는 **정적 매핑 언어(C#·Swift)에서 필드가 사라진다** — 예외도 없이 기본값이 된다. M20 이 시작되는 순간 부딪힌다. 그리고
[`decision_log.md` D-10](decision_log.md#d-10--바인딩은-원문-키를-바꾸지-않는다) ("원문 키는 어떤 바인딩도 바꾸지 않는다")의 전제 —
**원문 키가 일관되다** — 를 흔든다.

---

### G-05 · 같은 논리 오류가 진입로마다 다른 판정을 받는다

*층* L1·L4 — *이슈* **없다** ([#3876](https://github.com/edwardkim/rhwp/pull/3876) 조사에서 발견)

"문서에 없는 필드를 채우라"는 **같은 논리 오류**가 진입로에 따라 정반대 판정을 받는다.

```
$ rhwp edit fill-fields samples/field-01.hwp --data '{"없는필드":"값"}' -o <임시>/o.hwp --json
exit=0   출력 파일 생성=YES   {"filledCount":0,"notFound":["없는필드"], …}

$ rhwp run <같은 내용의 계획> --json
exit=2   출력 파일 생성=NO    {"invalid":[{"step":0,"reason":"필드 '…' 이(가) 없거나 …"}]}
```

**막는 것.** **종료 코드만 보는 게이트가 조용히 통과하는 경로**다. CI 가 `rhwp edit … || exit 1` 로 짜여 있으면 아무것도 안 채워졌는데 성공으로
읽고, 만들어진 출력 파일이 "채워진 문서"로 다음 단계에 들어간다.

**어느 쪽이 옳은가 — 판정이 필요하다.** `edit` 은 **부분 성공이 정상인 축**이고(10개 중 9개 채움) `run` 은 **계획 전체가 검증 가능해야 하는
축**이다. 둘 다 자기 층에서는 일관된다. 문제는 **그 차이가 어디에도 적혀 있지 않다**는 것이다.

**최소 조치.** [`agent_knowledge_map.md`](../../manual/agent_knowledge_map.md) 에 "`notFound` 는 실패가 아니다
— 실패로 다루려면 `run` 을 쓰거나 봉투를 검사하라"를 명시.

---

### G-06 · `-o` 와 `--json` 의 우선순위가 명령마다 다르다

*층* L1 — *이슈* **없다**

같은 플래그 조합이 한 바이너리 안에서 **세 가지 뜻**을 갖는다.

```
$ for c in export-text export-structure export-markdown export-tables; do
    rhwp $c samples/table-001.hwp --json -o <임시>/$c.f; done
export-text:      exit=0 stdout=756B 첫문자='{' 파일생성=NO
export-structure: exit=0 stdout=189B 첫문자='{' 파일생성=NO
export-markdown:  exit=0 stdout=468B 첫문자='{' 파일생성=YES
export-tables:    exit=0 stdout=140B 첫문자='표' 파일생성=YES
                                     ↑ "표 추출 완료: 1개 → <임시>/out.json"
```

| 명령 | 해석 |
| --- | --- |
| `export-text` · `export-structure` | `--json` 이 이긴다. `-o` 는 **조용히 무시**(디렉터리조차 안 생긴다) |
| `export-markdown` | 둘 다 한다 — 봉투 + 파일 |
| `export-tables` | `-o` 가 이긴다. **stdout 에 사람 문장** |

**막는 것.** `export-tables` 는 [`invariants.md`
INV-04](invariants.md#inv-04--stdout-은-데이터만-담는다)(stdout 순수성)와
[INV-13](invariants.md#inv-13--자기서술은-실물과-같다)(자기서술 = 실물)을 **동시에** 어긴다 — 레코드는 `json:true` +
`recordFields` 4개를 선언한다. [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가 "바인딩이 옵션을 닫은 건 회피일
뿐 수정이 아니다"라고 지목한 항목이다. 나머지 둘의 불일치도 **문서화되지 않은 방언**이다 — `export-text` 의 무시는 바인딩 가이드에만 근거가 있다("받아 주면
'저장했다'는 거짓말").

**정합 방향.** `export-markdown` 형태가 가장 정직하다(봉투에 `output` 필드로 어디에 썼는지 말해 준다). 다만 `export-text` 에 파일
생성을 되살리는 것은 **동작 변경**이라 `schemaVersion` 판단이 필요하다.

---

### G-07 · `render-diff --json` 은 이미 있는데 현황판이 "잔여"로 둔다

*층* L1(현황판 드리프트) — *이슈* [#3719](https://github.com/edwardkim/rhwp/issues/3719) §3-1

```
$ rhwp render-diff samples/field-01.hwp samples/field-01.hwp --json
exit=0  stdout=839B  {"hardStructPages":0,"maxDisp":0.0,"mode":"pair","pageCountA":3, …}

$ rhwp capabilities | jq -c '.commands[] | select(.name=="render-diff") | {json, flags}'
{"json":true,"flags":["--json","--batch","--via","-p","--max-disp","-o"]}
```

`tests/render_diff_json_contract.rs` 도 존재하고, 그 안의
`exit_code_dictionary_names_render_diff_without_dropping_the_others` 가 종료 코드 사전 등재까지 확인한다. 그런데
#3719 §3-1 은 이것을 "잔여 공백 3"의 첫 항목으로 든다.

**막는 것.** 로드맵이 실물보다 뒤처지면 **우선순위 판단이 틀린 전제 위에서 이뤄진다.**
#3719 §6 "다음 12조각" 2번은 이미 끝난 일이다. 같은 부류가 하나 더 있다 —
[#3877](https://github.com/edwardkim/rhwp/pull/3877) 이 지적한 #3608 M21 체크리스트: 첫 항목("cargo-fuzz 타깃
4종")이 **이미 머지됐는데 미체크**다. #3608 본문이 "체크 = 진행률의 유일 기준"이라고 선언하므로 실제 진행률이 왜곡된다.

---

### G-08 · `export-png --json` 게이트 정합

*층* L1 — *이슈* [#3357] · [#3719](https://github.com/edwardkim/rhwp/issues/3719) §3-1

```
$ rhwp export-png samples/field-01.hwp --json
exit=2  stdout=0B  stderr: 오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.

$ rhwp capabilities | jq -c '.commands[] | select(.name=="export-png")'
{"available":false,"category":"export","name":"export-png","requiresFeature":"native-skia", …}
```

**부분 재현인 이유.** 자기서술은 **잘 돼 있다** — `available:false` + `requiresFeature` 로 "이 빌드에는 없다"를 기계가 읽을 수 있고
exit 2 도 #3719 요구대로다. 확인 못 한 것은 **`available:true` 빌드에서 `--json` 봉투가 나오는가**다 →
[G-23](#g-23--이-pc-에서-rhwp-를-빌드할-수-없다).

**남은 실제 공백.** `json`·`flags` 키가 **레코드에 없다.** 게이트가 열린 빌드에서 이 명령이 `--json` 을 내는지, 낸다면 어떤
`recordFields` 인지 자기서술이 말하지 않는다 — [G-01](#g-01--dump--diag--core-pages-가---json-과-미지-옵션을-침묵-무시한다)
과 같은 부류다.

---

## 3. L2 — 무상태 도구

### G-09 · 무상태 도구가 어느 프로필에도 없을 수 있다

*층* L2 — *이슈* **PR [#3838](https://github.com/edwardkim/rhwp/pull/3838) 로 인플라이트**

도구를 추가하고 역할 프로필에 등재하지 않으면 **프로필로 필터링해 붙은 에이전트에게는 없는 것과 같다.**

```
$ grep -rn "every_stateless_tool_belongs_to_some_specific_profile" tests/
(결과 없음)                      ← 가드가 이 워크트리에 없다
$ echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | rhwp mcp-serve
총 51 (무상태 39 + 세션 12)
```

#3838 이 전수 대조로 찾은 누락은 **14건**이었다 — `hwp_digest`·`hwp_batch_fill`·
`hwp_replace_text`·`hwp_insert_image`·`hwp_run_plan`·`hwp_table_to_csv`·`hwp_csv_to_table`·
`hwp_export_doclang`·`hwp_sanitize`·`hwp_redact`·`hwp_inspect_hidden_text`·
`hwp_inspect_injection`·`hwp_inspect_unicode`·`hwp_render_diff`.

**막는 것.** [`invariants.md` INV-15](invariants.md#inv-15--무상태-도구는-최소-한-프로필에-속한다) 가 가드 없이 선언으로만
존재한다. 그 사이 새 도구를 추가하면 같은 부채가 다시 쌓인다. [`decision_log.md`
D-39](decision_log.md#d-39--프로필-등재-누락을-별도-pr-로-분리한다) 가 보여 준 대로 **"하나 빠졌다"는 대개 "여럿 빠졌다"의 첫 증상**이다.

---

### G-10 · 무상태 `hwp_search` 에 상한이 없다

*층* L2 — *이슈* **없다** ([#3802](https://github.com/edwardkim/rhwp/pull/3802) S7 이 "남긴 공백"으로 명시)

세션 `hwp_doc_search` 와 CLI `search` 는 `maxMatches` 를 갖는데 무상태 `hwp_search` 는 없다 (`capabilities
--mcp` 의 `inputSchema` 확인). CLI 쪽 상한은 정상 동작한다:

```
$ rhwp export-text samples/field-01.hwp --json --max-chars 5 | jq -c '{truncated, omittedCount}'
{"truncated":true,"omittedCount":149}
```

**원인.** `tests/mcp_arg_validation_contract.rs` 가 `--` 배선 순서를 못 박고 있어 인자를 끼워 넣을 자리가 없다. 그 계약은 `-` 로
시작하는 검색어를 표현하기 위해 존재한다 ([#3748](https://github.com/edwardkim/rhwp/pull/3748)) — **다른 기능의 정당한
계약**이다.

**막는 것.** 거대 문서의 검색 결과가 에이전트 컨텍스트를 밀어낸다. [`invariants.md`
INV-06](invariants.md#inv-06--조용히-자르지-않는다) 이 이 표면만 못 덮는다.

---

### G-11 · L6 매크로가 멈춰 있고 판정 1건이 남아 있다

*층* L6 — *이슈* [#3719](https://github.com/edwardkim/rhwp/issues/3719) §6 "판정 대기 2건" 중 남은 하나

`hwp_form_autopilot` 을 도메인 매크로로 만들 것인가, 계획서 템플릿으로 배포할 것인가가 **결정되지 않았다.** 짝이던 `hwp_doc_transaction`
은 [#3826](https://github.com/edwardkim/rhwp/pull/3826) 이 닫았다. `capabilities --mcp` 39개에
`hwp_form_autopilot` 은 없고, `run` 의 `steps` 4종 + `assertions` 2종이 조사→채움→체크→검증을 덮는다.

**막는 것.** 결정이 없으면 **누군가 매크로를 만들 수 있다.** 그러면 같은 일을 두 곳에서 해 [`invariants.md`
INV-14](invariants.md#inv-14--상위-층은-새-편집조판-로직을-만들지-않는다) 위반이다. 사실상 "템플릿으로 배포"로 가고 있으나 **결정으로 기록되지
않았다.** **최소 조치**: #3826 과 같은 형태의 판정 문서 1편, 코드 변경 0건.

---

## 4. L3 — 세션

### G-12 · 세션 도구에 undo/redo 가 없다

*층* L3 — *이슈* [#3719](https://github.com/edwardkim/rhwp/issues/3719) §3-3

```
$ echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | rhwp mcp-serve   (세션 도구 추출)
세션 12: hwp_close, hwp_doc_fields, hwp_doc_fill_fields, hwp_doc_info, hwp_doc_render_page,
         hwp_doc_replace_text, hwp_doc_save, hwp_doc_search, hwp_doc_set_cell,
         hwp_doc_tables, hwp_doc_text, hwp_open       ← undo / redo 없음
```

**막는 것.** 대화형 에이전트가 잘못 편집하면 **문서를 다시 열어야 한다.** 393쪽이면 130 ms 를 다시 쓰고 그 사이 편집 내용이 전부 날아간다.

**프리미티브는 이미 있다.** [#3878](https://github.com/edwardkim/rhwp/pull/3878) 이 찾았다:
`save_snapshot_native`/`restore_snapshot_native` 가 **코어에 있고 WASM 에만 노출**돼 있다. 즉 L3 undo 는 신규 로직이
아니라 **노출 문제**다 — [`decision_log.md` D-33](decision_log.md#d-33--m24-는-기능-추가가-아니라-봉투-층-이동이다) 과 같은
구조.

---

### G-13 · `SessionDoc` 에 원본 경로가 없다

*층* L3 — *이슈* **없다** (PR [#3878](https://github.com/edwardkim/rhwp/pull/3878) 부수 발견)

`src/mcp_serve.rs:41-49` 의 `SessionDoc` 에 원본 파일 경로 필드가 없다. **코드 인용으로만 확인했다** — 세션 상태 내부는 프로토콜 밖이라
실행으로 재확인하지 못했다.

**막는 것.** **감시·중복 제거·재적재 셋이 전부 이 한 필드에서 막힌다.** 같은 파일을 두 번 열면 핸들이 둘 생기고, 파일이 디스크에서 바뀌어도 세션이 모른다(스테일
— 사용자 편집을 덮어쓰는 경로). [`decision_log.md` D-34](decision_log.md#d-34--m25-는-증분-재파싱을-채택하지-않는다) 가 "감시의
값은 속도가 아니라 스테일 방지"라고 정한 그 값이 지금 실현 불가다.

---

### G-14 · 세션 저장이 원자 교체를 쓰지 않는다

*층* L3 — *이슈* **없다** (PR [#3878](https://github.com/edwardkim/rhwp/pull/3878) 부수 발견)

세션 `hwp_doc_save` 는 `std::fs::write` 직행(`mcp_serve.rs:1391`)인데 CLI 는 `write_atomically` 를 쓴다.
**비대칭이다.** 저장 중간에 프로세스를 죽여야 재현되므로 이 문서에서는 **코드 인용으로만** 확인했다.

**막는 것.** 저장 중 중단(디스크 가득·프로세스 종료)에서 **원본이 깨진 파일로 남는다.** 같은 저장소가 CLI 경로에서는 이미 막고 있는 위험을 세션 경로에서만
진다. 그리고 [`invariants.md` INV-14](invariants.md#inv-14--상위-층은-새-편집조판-로직을-만들지-않는다) 의 반례다 — 같은 저장을 두
방식으로 한다.

---

### G-15 · `hwp_doc_text` 를 `page` 없이 부르면 여는 것보다 비싸다

*층* L3(성능) — *이슈* **없다** (PR [#3878](https://github.com/edwardkim/rhwp/pull/3878) 실측 **인용**)

393쪽 문서에서 `page` 없이 부르면 **374 ms** — `hwp_open`(126~132 ms)보다 비싸다. 전수 순회다. 같은 393쪽 표본을 특정하지 못해
**재실측하지 않았다.**

**막는 것.** 세션의 존재 이유가 "재파싱 0"인데 이 한 호출이 이득을 지운다. 에이전트는 `page` 를 줘야 한다는 것을 모른다 — **도구 설명이 비용을 말하지
않는다.** **최소 조치**: `hwp_doc_text` description 에 "쪽을 지정하지 않으면 전수 순회" 명시.
[#3876](https://github.com/edwardkim/rhwp/pull/3876) 의 `cost_model.md` 가 이 축의 자리다.

---

## 5. L4 — 계획 실행기

### G-16 · 계획 스키마에 `dryRun` 과 `preview` `skipped` 가 빠져 있다

*층* L4 — *이슈* **PR [#3808](https://github.com/edwardkim/rhwp/pull/3808)** (열림)

**전제가 사라졌는데 아직 반영되지 않았다.**

```
$ cat plan.json
{"planVersion":"1.0","dryRun":true,"input":"samples/field-01.hwp","output":"<임시>/dr.hwp",
 "steps":[{"action":"fill_fields","data":{"회사명":"검증사"}}]}
$ rhwp run plan.json --json
exit=0   출력 파일 생성=NO   {"dryRun":true,"invalid":[],"preview":[{"step":0, …}]}

$ rhwp capabilities | jq -r '.commands[] | select(.name=="run") | .flags | join(" ")'
--json --plan-json --dry-run
```

즉 **계획서에 실린 `dryRun` 이 실제로 존중된다.** #3808 이 필드를 뺀 근거였던 "`--dry-run`([#3759]/[#3761])이 아직 devel 에
없다"가 사라졌다.

**막는 것.** 스키마가 실물보다 좁으면 에이전트가 **쓸 수 있는 기능을 못 쓴다.** 그리고 `export-agent-manifest` 의
`missingAxes:["planSchema"]`([#3843])가 채워지지 않아 부트스트랩 왕복이 계속 불완전하다.

**해야 할 일 (두 PR 의 약속).** #3808 본문이 "머지 시 필드 추가 = minor" 로 예고했고, dry-run `preview[]` 의 `{step,
action, skipped, reason}` 반영도 **"두 PR 중 나중에 머지되는 쪽"** 몫으로 남겼다. **이제 나중이 #3808 로 확정됐다.**
[`decision_log.md` D-04](decision_log.md#d-04--계획-스키마에-dryrun-을-일부러-넣지-않았다).

---

## 6. 바인딩 (M18~M20)

### G-17 · 파이썬 바인딩의 치명 결함 3건

*층* 바인딩 — *이슈* **PR [#3879](https://github.com/edwardkim/rhwp/pull/3879)** (열림, 문서만)

#3879 가 표류 **20건**(12건이 파이썬 뒤처짐)을 찾았고 그중 치명 3건이다.
**하나는 실행으로 재현**했고 둘은 코드 경로만 확인했다.

**D-1 · `convert(out=)`·`export_hwpx(out=)` 가 항상 실패한다 — 재현 O.**

```
$ rhwp convert samples/field-01.hwp -o <임시>/conv.hwpx --json
exit=2  stdout=0B  stderr: 알 수 없는 옵션: -o
        사용법: rhwp convert <입력> <출력> [--verify] [--verify-pages] [--json]
```

바인딩은 `-o` 를 붙인다 — `bindings/python/src/rhwp/commands.py:369` `_flag(args, "-o", out)`. 따라서
`rhwp.convert(out=…)` 는 **항상 `UsageError`(exit 2)** 다. Node 는 위치 인자로 넘기고 그 사실을
`commands.ts:534-536` 주석에 남겼다.

**D-4 · 검사인 줄 알았는데 편집한다 — 재현 실패(전제 부재).**

`bindings/python/src/rhwp/plan.py:194` 의 `check()` 는 `_execute(self.to_dict(dry_run=True))` 직행이고
**지원 여부 게이트가 없다.** Node 는 `bindings/node/src/plan.ts:239-278` 에서 `capabilities` 를 읽어 `--dry-run`
지원을 캐시하고 미지원이면 거부한다. **이 PC 에서는 재현할 수 없었다** — `#3761` 이전 바이너리가 없다. 현재 바이너리는 계획서 `dryRun` 을
존중하므로([G-16](#g-16--계획-스키마에-dryrun-과-preview-skipped-가-빠져-있다)) 검사가 편집하지 않는다. **위험은 "미지원 바이너리와
짝지어졌을 때"** 이고, `RHWP_BIN` 으로 임의 바이너리를 가리킬 수 있으므로 가설적 위험이 아니다.

**D-19 · 구조적 원인 — 부분 재현 실패.**

#3879 는 "파이썬에 **선언 → 래퍼** 패리티 테스트가 없다"고 적었다. **파일은 있다** —
`bindings/python/tests/test_envelope_parity.py`(테스트 10건). 다만 범위가 좁다.

| 테스트 | 무엇을 보나 |
| --- | --- |
| `test_declared_fields_actually_appear` | `recordFields` ↔ 실물 봉투. **조회 계열 4개만** |
| `test_declared_flags_are_accepted_by_the_tool` | `info` 의 **`--json` 하나만** |

**즉 "래퍼가 조립하는 argv 를 CLI 가 받아들이는가"를 보는 테스트가 없다.** 그래서 D-1 이 머지까지 살아남았다. #3879 의 결론은 맞고 표현이 넓었다 — 이
문서는 **"패리티 테스트 파일은 있으나 래퍼 argv 조립을 대조하지 않는다"** 로 좁힌다.

**그 외 17건.** `_quote` 역슬래시 미이스케이프(`errors.py:160-165` — `"` 만 치환하고 `\` 는 그대로라 `C:\my dir\` 가
`"C:\my dir\"` 로 깨진다. 대상이 **오류 메시지의 재현 명령 문자열**이라 실행에는 영향 없음) · ReDoS 정규식 · **`VerifyReport` 진리값이
언어마다 반대 뜻**(`if result.verify` = 통과 / `if (saved.verify)` = 요청함) · `inspect` 인자 순서 반대 등.

**막는 것.** 바인딩이 늘수록 표류가 는다. M20(C#·Swift) 착수 전에 **패리티 계약 자체를 테스트로** 만들어야 한다.

---

### G-18 · `csharp`/`swift`/`Native` 봉투에 판정 어휘가 없다

*층* 바인딩 — *이슈* [#3608](https://github.com/edwardkim/rhwp/issues/3608) M20

```
$ wc -l bindings/Native/src/lib.rs bindings/csharp/RhwpNative.cs
  376 bindings/Native/src/lib.rs        ← cdylib C ABI
   63 bindings/csharp/RhwpNative.cs     ← P/Invoke
$ ls bindings/swift/Sources/Rhwp/
Rhwp.swift  RhwpDocumentTextView.swift
```

#3879 §8 에 따르면 노출 함수 4개, 봉투는 `{"ok":true,…}` 로 **`schemaVersion` 도 종료
코드도 판정 표현도 없다.** (봉투 형태는 빌드 불가로 **확인되지 않음** — [G-23](#g-23--이-pc-에서-rhwp-를-빌드할-수-없다).)

**막는 것.** "M20 미착수"라는 전제로 새 바인딩을 설계하면 **디렉터리 이름을 이미 다른 계약이 점유**하고 있다는 사실에 부딪힌다. 이 봉투는
[`invariants.md` INV-08](invariants.md#inv-08--필드-추가는-자유-변경삭제는-schemaversion-범프)·
[INV-20](invariants.md#inv-20--출처-표지는-항상-실린다) 을 전혀 따르지 않는다 — 마이그레이션인가 폐기인가를 먼저 정해야 한다.

---

## 7. 인프라 · 운영

### G-19 · `batch fill` 이 행마다 서식을 재파싱한다

*층* 인프라(성능) — *이슈* **없다** (PR [#3878](https://github.com/edwardkim/rhwp/pull/3878) 실측 **인용**)

`fill_fields_core` 가 진입마다 `fs::read` + `from_bytes` 를 한다 → **137 ms/행**. 1,000행 메일머지면 137초가 파싱이다.
이 문서에서는 재실측하지 않았다.

**막는 것.** 메일머지의 실용 규모를 제한한다. 해결책의 프리미티브도 이미 있다 —
`save_snapshot_native`/`restore_snapshot_native`(코어, WASM 에만 노출). fork 방식으로 한 번 파싱하고 N 번 복제하면 된다.
[`invariants.md` INV-14](invariants.md#inv-14--상위-층은-새-편집조판-로직을-만들지-않는다) 를 지키면서 고칠 수 있는 항목이다.

---

### G-20 · 퍼징이 CI 에서 돌지 않는다

*층* 인프라 — *이슈* **PR [#3877](https://github.com/edwardkim/rhwp/pull/3877)** (문서만)

```
$ grep -ril fuzz .github/
(결과 0건)
$ ls .github/workflows/
build-nextest-archives.yml  cache-generation-sweep.yml  cancel-stale-pr-runs.yml  ci.yml
close-issues-on-devel-push.yml  codeql.yml  deploy-pages.yml  full-renderer-sweep.yml
npm-publish.yml  release-binary.yml  render-diff.yml  run-nextest-archives.yml
$ ls fuzz/fuzz_targets/
parse_hml.rs  parse_hwp.rs  parse_hwp3.rs  parse_hwpx.rs  parse_ooxml_chart.rs  parse_wmf.rs
```

**막는 것.** 인프라가 있는데 아무도 안 돌린다. #3877 조사: `git log -- fuzz/` 커밋 3개가 전부 인프라 도입이고 **크래시 유입 0건**, 반대로
**퍼저가 잡을 수 있었던 손수정 결함이 커밋 로그에 12건**이다.

**제안된 경로.** nightly 대신 **stable `cargo test` 로 도는 코퍼스 재생 계약 스위트**를 먼저 만든다([`decision_log.md`
D-36](decision_log.md#d-36--퍼징을-ci-에-바로-넣지-않고-코퍼스-재생-계약-스위트를-먼저-제안한다)). **아직 없다.**

---

### G-21 · `fuzz/regressions/` 가 규정돼 있는데 없다

*층* 인프라 — *이슈* **PR [#3877](https://github.com/edwardkim/rhwp/pull/3877)**

```
$ grep -n "regressions" fuzz/README.md
94:   `fuzz/regressions/<타깃>/` 에 커밋합니다(코퍼스와 회귀 케이스를 분리).
$ ls fuzz/
Cargo.toml  README.md  corpus  fuzz_targets        ← regressions 없음
```

**막는 것.** 규정과 실물이 다르면 규정이 무시된다. 크래시를 찾아도 **어디에 넣어야 하는지 실물 예시가 없다.**
[G-20](#g-20--퍼징이-ci-에서-돌지-않는다) 과 한 묶음으로 처리하는 것이 자연스럽다 — 재생 스위트가 읽을 디렉터리가 곧 이것이다.

---

### G-22 · 모델 미탑재를 강제하는 장치가 없다

*층* 인프라 — *이슈* **없다**

[`invariants.md` INV-18](invariants.md#inv-18--rhwp-에-모델을-넣지-않는다) 은 정책 문서에만 있다. `Cargo.toml` 에
모델·임베딩 크레이트가 들어오는 것을 막는 자동 검사가 없다 (위 [G-20](#g-20--퍼징이-ci-에서-돌지-않는다) 의 워크플로 목록 참조 — `codeql.yml` 은
코드 스캔이지 의존성 허용목록이 아니다).

**막는 것.** 이 불변식은 **유혹이 실제로 있는 자리**에서 반복 시험받는다 —
[#3836](https://github.com/edwardkim/rhwp/pull/3836)(`--search` 를 임베딩으로?) ·
[#3832](https://github.com/edwardkim/rhwp/pull/3832)(`explain` 을 LLM 으로?) ·
[`detection_policy.md`](../agent_security/detection_policy.md) ③. 지금까지는 매번 사람이 막았다. **최소 조치**:
`cargo-deny` 류 허용목록. 산출물 크기 회귀 임계도 같은 축이다 (`crate-type = ["rlib","cdylib"]` 이라 로컬 모델은 WASM 산출물에
그대로 실린다).

---

### G-23 · 이 PC 에서 rhwp 를 빌드할 수 없다

*층* 인프라(환경) — *이슈* **없다**

MSVC `dbghelp.lib` 손상 + GNU `dlltool` 부재로 링크가 안 된다(`CVT1107`/`LNK1123`).
[#3877](https://github.com/edwardkim/rhwp/pull/3877) 이 최소 크레이트로 재현했다.

**이 문서에 미치는 영향 (정직하게).**

- **계약 테스트 총 건수를 세지 못했다.** `tests/` 의 `*contract*.rs` 는 **66개 파일**
(실측)이지만 `#[test]` 총합은 **확인되지 않음**. #3719 가 2026-08-01 에 기록한 값은 계약 215건 / 전체 1,486건이다
- **`native-skia` 게이트가 열린 빌드**를 확인하지 못했다 → [G-08](#g-08--export-png---json-게이트-정합)
- **Node 바인딩을 실행 검증하지 못했다** — `bindings/node/node_modules` 부재(실측).
#3879 도 같은 이유로 Node 주장을 코드 인용으로만 남겼다
- 모든 실측은 **미리 빌드된 `target/release/rhwp.exe`(v0.8.2)** 로 했고,
**이 바이너리의 커밋은 확인되지 않았다**

**막는 것.** **CI 가 유일한 검증 수단**이다. red→green 을 로컬에서 못 돌리므로 PR 마다 CI 왕복이 필요하고, 그것이 리뷰 큐를
늘린다([G-25](#g-25--동시-열린-pr-이-22건이다)와 간접 연결).

**비공개 경로 항목.** [#3802](https://github.com/edwardkim/rhwp/pull/3802) 는 조사 중 "문서 내용이 파일 경로로 해석되는 실물
경로 하나"를 찾아 재현·수정했고, `SECURITY.md` 규약에 따라 **위치·재현·패치를 PR 에서 제외하고 비공개 경로로 제보**했다. 그 내용은 이 문서에 적지
않는다.

---

## 8. 문서 · 운영

### G-24 · 레시피 03(PII 마스킹)이 비어 있다

*층* 문서 — *이슈* **PR [#3835](https://github.com/edwardkim/rhwp/pull/3835)** (열림)

```
$ ls mydocs/manual/recipes
ls: cannot access 'mydocs/manual/recipes': No such file or directory
```

레시피 묶음 전체가 아직 devel 에 없고(#3835 미머지), 그중 **03 은 PR 안에도 없다.**

**왜 안 썼는가 — 원칙 준수 사례다.** 작업 워크트리가 `edit redact` 병합 이전 스냅샷이라 명령을 실행할 수 없었고, **"실행 없이 출력을 지어내는 것은 이
묶음의 원칙('지어낸 값 없음')을 정면으로 어긴다"** 는 이유로 쓰지 않았다. [`invariants.md`
INV-26](invariants.md#inv-26--증적은-두-종류다).

**막는 것.** 01·04 본문에 03 을 가리키는 **상호 참조가 이미 있어** 그 자리가 비어 있다. `edit redact` 는 되돌릴 수 없는 명령이라 서사형 안내의
필요가 가장 큰 축이다.

**지금은 쓸 수 있다.** 이 바이너리에 `edit redact` 가 있다 — `capabilities.commands[edit].flags` 에
`--kind`·`--mask`·`--in-place`·`--keep-preview` 가 있고 `capabilities --mcp` 에
`hwp_redact`·`hwp_sanitize` 가 있다. 전제가 해소됐다.

---

### G-25 · 동시 열린 PR 이 22건이다

*층* 운영 — *이슈* [#3719](https://github.com/edwardkim/rhwp/issues/3719) §7

```
$ gh pr list --repo edwardkim/rhwp --author kevin9327 --state open --limit 100 --json number -q 'length'
22
```

2026-07-22 메인테이너가 열린 PR 폭주로 30건을 일괄 close 하며 동시 열린 PR 을 **10건 내외**로 요청했다. #3719 가 2026-08-01 에
16건으로 기록했고, 지금 **22건**이다.

| 성격 | 건수 | 예 |
| --- | --- | --- |
| 문서·설계 (코드 0) | 6 | #3876 #3877 #3878 #3873 #3879 #3826 |
| 신규 표면 | 7 | #3843 #3842 #3841 #3836 #3835 #3832 #3808 |
| 결함 수정 | 6 | #3875 #3872 #3870 #3839 #3838 #3882 |
| 기타 | 3 | #3871 #3867 #3827 |

**막는 것.** 리뷰 대역폭을 넘으면 전부가 느려진다. **결함 수정 6건이 신규 표면 7건과 같은 큐에 있다** — #3719 §7 이 경고한 "리뷰 부하 편중"이다.
`#3875`(WMF 패닉)· `#3839`(암호 문서 못 여는 명령 7건)·`#3870`(표 셀 검색 불가)은 사용자가 지금 겪는 결함이다. 규약은 유효한데 지켜지지 않고
있다 — [`decision_log.md` D-40](decision_log.md#d-40--동시-열린-pr-을-10건-내외로-유지한다).

---

## 9. 재현 실패 · 확인되지 않음

**이 문서가 남의 보고를 그대로 옮기지 않았다는 증거다.**

| 항목 | 상태 | 왜 |
| --- | --- | --- |
| #3879 **D-4** — "미지원 바이너리에서 검사 호출이 실제 편집을 수행한다" | **재현 실패** | 이 PC 에 `#3761` 이전 바이너리가 없다. 현재 바이너리는 계획서 `dryRun` 을 존중한다(실측). **코드 경로만 확인** — `plan.py:194` 게이트 없음 / `plan.ts:239-278` 게이트 있음 |
| #3879 **D-19** — "파이썬에 선언 → 래퍼 패리티 테스트가 **없다**" | **부분 재현 실패** | 파일은 **있다**(`test_envelope_parity.py`, 10건). 다만 래퍼 argv 조립을 대조하지 않는다 → [G-17](#g-17--파이썬-바인딩의-치명-결함-3건) 에서 주장을 좁혔다 |
| #3879 — Node 바인딩 동작 | **확인되지 않음** | `bindings/node/node_modules` 부재(실측). `vitest`/`tsc` 미실행 |
| #3878 — `hwp_doc_text` 374 ms · `batch fill` 137 ms/행 | **재실측 안 함** | 393쪽 표본을 특정하지 못했다. #3878 실측 인용 |
| #3878 — `SessionDoc` 경로 부재 · 세션 저장 비원자성 | **코드 인용만** | 세션 내부 상태는 프로토콜 밖 |
| `export-png` `available:true` 빌드의 `--json` 봉투 | **확인되지 않음** | `native-skia` 빌드 불가 → [G-23](#g-23--이-pc-에서-rhwp-를-빌드할-수-없다) |
| 계약 테스트 총 건수 | **확인되지 않음** | 빌드 불가. 파일 수 66개만 실측 |
| `target/release/rhwp.exe` 의 커밋 | **확인되지 않음** | 워크트리에서 git 조회를 하지 않았다. 표면으로 시점만 좁혔다 |
| `csharp`/`swift`/`Native` 봉투가 `{"ok":true,…}` 라는 것 | **확인되지 않음** | 빌드 불가. #3879 §8 인용 |

**한 번 재현 실패로 오인한 것 (기록).** `export-tables -o --json` 을 처음에는 **디렉터리** 경로로 시험해 `exit 1` + `stdout
0B`(쓰기 권한 거부)를 받아 "재현 실패"로 볼 뻔했다. **파일** 경로로 다시 시험하니 exit 0 + 사람 문장 140B 가 나왔다 →
[G-06](#g-06---o-와---json-의-우선순위가-명령마다-다르다). 재현 실패를 선언하기 전에 **입력 형태를 바꿔 한 번 더** 보는 것이 이 대장의 규약이다.

---

## 10. 우선순위 제안

[#3880](https://github.com/edwardkim/rhwp/issues/3880) §3 의 순서를 이 대장의 실측으로 갱신한다. **아래 층이 위 층의
전제다.**

| 순서 | 무엇 | 왜 먼저인가 | 크기 |
| --- | --- | --- | --- |
| 1 | [G-03](#g-03--info---json-에-warnings-가-없다)·[G-04](#g-04--봉투에-snake_case-키가-하나-남아-있다) 머지(#3882) | 이미 인플라이트. L3 도달의 전제 | 완료 대기 |
| 2 | [G-02](#g-02--run-의-실패-경로-예외를-자기서술이-적지-않는다) 자기서술에 예외 기재 | #3869 봉투 동등성 계약의 기준점 | 문구 + 테스트 |
| 3 | [G-01](#g-01--dump--diag--core-pages-가---json-과-미지-옵션을-침묵-무시한다) 정책 결정 + 역방향 가드 | 가드의 구조적 구멍. 새 명령마다 재발 | 정책 + 스윕 |
| 4 | [G-16](#g-16--계획-스키마에-dryrun-과-preview-skipped-가-빠져-있다) #3808 리베이스 | 전제가 이미 사라졌다 | 필드 추가 |
| 5 | [G-09](#g-09--무상태-도구가-어느-프로필에도-없을-수-있다) #3838 머지 | 부채 14건 + 재발 가드 | 완료 대기 |
| 6 | [G-06](#g-06---o-와---json-의-우선순위가-명령마다-다르다) `-o`↔`--json` 정합 | M20 정적 언어 진입 전 | 동작 변경 주의 |
| 7 | [G-17](#g-17--파이썬-바인딩의-치명-결함-3건) 바인딩 D-1·D-4 수정 | 바인딩이 늘기 전에 | 수정 + 패리티 테스트 |
| 8 | [G-20](#g-20--퍼징이-ci-에서-돌지-않는다)·[G-21](#g-21--fuzzregressions-가-규정돼-있는데-없다) 코퍼스 재생 스위트 | 인프라가 있는데 안 돈다 | 신규 |
| 9 | [G-07](#g-07--render-diff---json-은-이미-있는데-현황판이-잔여로-둔다) 현황판 갱신 | 우선순위 판단의 전제 | 문서 |
| 10 | [G-25](#g-25--동시-열린-pr-이-22건이다) 큐 정상화 | 위 전부의 처리 속도를 정한다 | 운영 |

**L3·L4·L6 항목(G-11 · G-12~G-15)은 위 1~5 가 닫힌 뒤에 본다** — 봉투 계약이 흔들리는
상태에서 상위 층을 쌓으면 나중에 전부 다시 만진다.

---

## 11. 이 문서를 고칠 때

1. **공백을 추가하면** 다섯 칸(증상·재현·층·막는 것·이슈)을 채운다.
**이슈 번호가 없으면 "없다"라고 쓴다** — 빈칸은 "확인 안 함"과 구별되지 않는다
2. **재현은 직접 한다.** 못 하면 [§9](#9-재현-실패--확인되지-않음) 에 이유와 함께 올린다.
남의 보고를 옮기면 출처를 명시하고 "인용"이라고 적는다
3. **닫힌 항목은 지우지 말고 이슈·PR 번호와 함께 "닫힘"으로 표시**한다 —
왜 공백이었는지가 다음 사람에게 필요하다
4. 규칙은 [`invariants.md`](invariants.md), 결정은 [`decision_log.md`](decision_log.md)
