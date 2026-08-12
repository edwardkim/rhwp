---
kind: canonical
status: active
canonical: mydocs/tech/agent_architecture/layer_model.md
last_verified: 2026-08-03
---

# 에이전트 표면 불변식 전수

> **v0.8.4 현행성 주의:** Python·Node 바인딩을 예로 든 항목은 철회 전 설계 이력이다.
> 두 공식 바인딩은 #4655에서 제거됐으며 현재 지원 표면이 아니다.

> 이 표면이 **지키기로 한 규칙**을 한 곳에 모은다. 지금까지 이 규칙들은 이슈 본문
> ([#3719](https://github.com/edwardkim/rhwp/issues/3719) §4)·PR 본문 수십 건·코드 주석·
> [`detection_policy.md`](../agent_security/detection_policy.md)·
> [`agent_surface_playbook.md`](../../manual/agent_surface_playbook.md) 에 **흩어져** 있었다.
> 흩어진 규칙은 새로 들어온 사람에게 없는 규칙이고, 없는 규칙은 다음 PR 에서 조용히 깨진다.

관련 문서 — 층 구조는 [`layer_model.md`](layer_model.md), 로드맵 7개의 지도는
[`roadmap_atlas.md`](roadmap_atlas.md), 결정의 역사는 [`decision_log.md`](decision_log.md), 미해결 공백은
[`open_gaps.md`](open_gaps.md), 축 지도는 [`README.md`](README.md). 보안 축의 전제는
[`agent_security/threat_model.md`](../agent_security/threat_model.md), 상위 이슈는
[#3880](https://github.com/edwardkim/rhwp/issues/3880).

---

## 0. 읽는 법

**항목 형식** — 불변식마다 네 가지를 붙인다. 넷 중 하나라도 못 채우면 그건 불변식이 아니라 관습이다. ① **무엇을 금지하는가**(구체적 행위. "잘 하자"가 아니라
"이걸 하지 마라") ② **왜**(근거가 없으면 6개월 뒤 뒤집힌다) ③ **어기면**(소비자에게 실제로 무슨 일이) ④ **무엇이 강제하는가**(계약 테스트 이름 또는
선검사 항목 — **비어 있으면 그것이 발견이다**).

**근거 규약** — 모든 주장에 **이슈·PR 번호 / 코드 경로(`파일:줄`) / 실측 출력** 중 하나가 붙는다. 못 대는 항목은 **"확인되지 않음"**. 추측을
사실처럼 적은 계약 문서는 반년 뒤 거짓말이 된다 — 보안 축이 [`threat_model.md`](../agent_security/threat_model.md) 에서 세운
규약을 이 축도 따른다.

**측정 환경** — `rhwp v0.8.2`(`<저장소>/target/release/rhwp.exe`) · 2026-08-03 · 명령 **61** · `--json`
**31** · MCP 무상태 **39** · 세션 12 · 표본 `samples/field-01.hwp`(HWP5·3쪽·누름틀 11) ·
`samples/table-001.hwp`.

> **이 바이너리가 어느 커밋에서 빌드됐는지는 확인되지 않았다** — 워크트리에서 git 조회를
> 하지 않았다. 표면으로 시점만 좁힌다: `run --dry-run`([#3761])·`inspect` 3종·`edit redact`·
> `table-to-csv` 는 **있고**, `capabilities --search`([#3836])·`explain`([#3832])·
> `export-agent-manifest`([#3843])·`export-plan-schema`([#3808])는 **없다**.
> #3719 의 2026-08-01 스냅샷(명령 54 · json 21 · 도구 23)보다 뒤다.

---

## 1. 불변식 색인

| ID | 불변식 | 층 | 강제 상태 |
| --- | --- | --- | --- |
| [INV-01](#inv-01--판정은-데이터고-종료-코드는-그-파생이다) | 판정은 데이터, 종료 코드는 파생 | 횡단 | 부분 |
| [INV-02](#inv-02--판정과-실패를-종료-코드로-가른다) | 판정(3·4) vs 실패(1·2) | L1 | **강제됨** |
| [INV-03](#inv-03--실패-경로는-stdout-에-0바이트를-쓴다) | 실패 경로 stdout 0바이트 | L1 | **약함 + 예외** |
| [INV-04](#inv-04--stdout-은-데이터만-담는다) | stdout 순수성 | L1 | **위반 실측** |
| [INV-05](#inv-05--부분-목록을-내지-않는다--확신-없으면-null) | 부분 목록 금지 | 횡단 | 부분 |
| [INV-06](#inv-06--조용히-자르지-않는다) | 절단은 봉투에 남긴다 | L1·L3 | **강제됨** |
| [INV-07](#inv-07--아무-일도-안-하는-플래그를-두지-않는다) | 무동작 플래그 금지 | L1 | 부분 |
| [INV-08](#inv-08--필드-추가는-자유-변경삭제는-schemaversion-범프) | 하위호환 | 횡단 | **강제됨** |
| [INV-09](#inv-09--미지-옵션을-침묵-무시하지-않는다) | 미지 옵션 침묵 무시 금지 | L1 | **미강제** |
| [INV-10](#inv-10--봉투-키는-camelcase-다) | 봉투 키는 camelCase | L1 | 인플라이트 |
| [INV-11](#inv-11--도구-선언은-한-곳에서만-나온다) | 단일 출처 | L2·L3 | **강제됨** |
| [INV-12](#inv-12--선언과-배선은-같이-간다) | 선언 ↔ 배선 | L2 | **강제됨** |
| [INV-13](#inv-13--자기서술은-실물과-같다) | 자기서술 = 실물 | 횡단 | **강제됨(단방향)** |
| [INV-14](#inv-14--상위-층은-새-편집조판-로직을-만들지-않는다) | 새 로직 금지 | L2~L6 | 미강제(관행) |
| [INV-15](#inv-15--무상태-도구는-최소-한-프로필에-속한다) | 프로필 등재 | L2 | 인플라이트 |
| [INV-16](#inv-16--조용히-고치지-않고-표시한다) | 표시하되 고치지 않는다 | 보안 | **강제됨** |
| [INV-17](#inv-17--오탐은-기능-실패다) | 오탐 0 | 보안 | **강제됨** |
| [INV-18](#inv-18--rhwp-에-모델을-넣지-않는다) | 모델 미탑재 | 횡단 | 미강제(정책) |
| [INV-19](#inv-19--문서에서-온-문자열은-데이터-자리에만-온다) | 문서 문자열 격리 | 보안 | **강제됨** |
| [INV-20](#inv-20--출처-표지는-항상-실린다) | 출처 표지 | 횡단 | **강제됨** |
| [INV-21](#inv-21--판정-기준은-스펙이-아니라-이-엔진이-그려-내는-결과다) | 렌더러 기준 판정 | 보안 | **강제됨** |
| [INV-22](#inv-22--되돌릴-수-없는-작업은-목적지를-명시하지-않으면-거부한다) | 파괴적 작업 방어 | L1 | **강제됨** |
| [INV-23](#inv-23--체크섬을-통과하지-못하면-마스킹하지-않는다) | 마스킹 보수성 | 보안 | **강제됨** |
| [INV-24](#inv-24--악성-표본을-저장소에-두지-않는다) | 표본 비커밋 | 보안 | **강제됨** |
| [INV-25](#inv-25--허용목록을-베끼지-않고-원본에서-읽는다) | 허용목록 단일 출처 | 횡단 | 관행(코드 구조) |
| [INV-26](#inv-26--증적은-두-종류다) | 증적 2종 | 절차 | 미강제(리뷰) |

**강제 상태 어휘** — `강제됨`: 실패하면 CI 가 막는 계약 테스트가 있다 / `부분`: 일부 표면만 덮는다 / `약함`: 가드가 있으나 전수가 아니다 / `미강제`:
문서에만 있다 / `인플라이트`: 가드가 열린 PR 에 있다.

---

## 2. 봉투와 종료 코드 (L1)

### INV-01 · 판정은 데이터고, 종료 코드는 그 파생이다

- **금지** — 성공·실패를 봉투 필드 없이 종료 코드나 산문 메시지로만 표현하는 것
- **출처** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) §4-2 · L5 설계 원리 ①
- **강제** — 명령별 계약 테스트가 봉투 필드를 단언 (`envelope_shape_is_stable`, `envelope_has_every_declared_field`, `journal_reports_steps_and_verify` 등)

**왜.** 종료 코드는 정보량이 3비트다. "왜 실패했는가"·"무엇이 바뀌었는가"·"다음에 무엇을 하면 되는가"를 담을 수 없다. 에이전트는 그 셋을 알아야 다음 호출을
만든다.

**어기면.** 소비자가 stderr 산문을 정규식으로 파싱한다. 그 정규식은 다음 릴리스에서 문구가 한 글자 바뀌면 조용히 깨진다 — 깨졌다는 신호도 없이 "성공"으로
읽힌다.

**실측.** `edit fill-fields` 는 없는 필드를 만나도 exit 0 이지만 봉투가 판정을 담는다.

```
$ rhwp edit fill-fields samples/field-01.hwp --data '{"없는필드":"값"}' -o <임시>/o.hwp --json
exit=0
{"filledCount":0,"notFound":["없는필드"],"changedPages":[],"verify":null, …}
```

**한계.** 이 불변식을 **전 명령에 대해** 확인하는 가드는 없다. 명령마다 자기 계약 테스트가 자기 필드를 볼 뿐이다. 새 명령이 봉투 없이 종료 코드만 내도 아무것도
실패하지 않는다 — [INV-13](#inv-13--자기서술은-실물과-같다) 의 `recordFields` 선언 의무가 간접적으로 막을 뿐이다.

---

### INV-02 · 판정과 실패를 종료 코드로 가른다

- **금지** — 검증 단언 실패(3·4)를 런타임 실패(1)나 사용법 오류(2)에 섞는 것. 그리고 exit 3 을 내는 표면을 늘리면서 사전을 갱신하지 않는 것
- **출처** — [#2707] 종료 코드 사전 · [#3719](https://github.com/edwardkim/rhwp/issues/3719) §4
- **강제** — `tests/cli_json_contract.rs:659` **`exit_code_dictionary_covers_every_verify_surface`** · `tests/render_diff_json_contract.rs:548` `exit_code_dictionary_names_render_diff_without_dropping_the_others`

**왜.** `1`·`2` 는 "요청을 수행하지 못했다"이고 `3`·`4` 는 "수행했는데 결과가 기대와 다르다"다. 전자는 재시도·인자 수정이 답이고 후자는 **되돌리기**가
답이다. 게이트 스크립트가 이 둘을 합치면 정상 실패를 회귀로, 회귀를 정상 실패로 읽는다.

**어기면.** exit 3 표면이 `convert/export-hwpx` 하나였다가 `edit` 3종([#3702])·`run` 계획
단언([#3703])·`render-diff` 로 넓어졌는데 자기서술이 옛 문구에 머문 일이 실제로
있었다([#3719](https://github.com/edwardkim/rhwp/issues/3719) §4 "발견된 드리프트"). 자기서술만 읽는 에이전트는 "편집에는 3이
안 나온다"고 오판한다.

**실측.** 사전이 네 표면을 전부 이름으로 든다.

```
$ rhwp capabilities | jq -r '.exitCodes["3"]'
검증 단언 실패 — convert/export-hwpx --verify IR 차이, edit 3종 --verify 저장본 불일치,
run 계획 assertions 미충족, render-diff --json 시각 회귀 검출(사람 모드는 종전대로 1)
```

가드는 `["convert","edit","run"]` 세 문자열이 사전 문구에 들어 있는지를 단언한다. **표면이 늘면 사전도 늘어야 통과한다** — 이 축에서 가장 잘
설계된 가드다.

---

### INV-03 · 실패 경로는 stdout 에 0바이트를 쓴다

- **금지** — 실패로 끝나는 경로에서 stdout 에 무엇이든 쓰는 것 (특히 반쪽 JSON)
- **출처** — `capabilities.jsonContract.failure` · [#3795](https://github.com/edwardkim/rhwp/pull/3795)
- **강제** — 명령별 `*_silent_stdout` / `failure_paths_keep_stdout_empty` **23개 파일** + `tools/agent_preflight.py:552` `check_failure_stdout_silent`

**왜.** 봉투를 절반 쓰고 죽으면 소비자는 두 가지 중 하나를 한다 — 파싱하다 예외를 내거나, **더 나쁘게는 잘린 값을 참으로 읽는다.** 후자는 조용하다.
`"pageCount": 3` 까지만 나온 JSON 을 관대한 파서가 받아들이면 에이전트는 3쪽짜리 문서를 다 봤다고 믿는다.

**어기면.** 위 시나리오가 그대로 일어난다. 실패했는데 실패한 줄 모르는 소비자가 생긴다.

**강제가 약하다 — 이게 발견이다.**

1. **선검사는 하드코딩 3경로만 본다.** `tools/agent_preflight.py:556-560`

   ```python
   cases = [
       ["info", "--json", "존재하지_않는_파일_preflight.hwp"],
       ["info", "--json"],
       ["export-text", "--json", "존재하지_않는_파일_preflight.hwp"],
   ]
   ```

명령 61개 중 **2개**를 본다. 새 명령은 자동으로 검사 대상이 되지 않는다.

2. **계약 테스트는 명령마다 따로다.** `grep -rn "silent_stdout\|keep_stdout_empty" tests/`
가 23개 파일을 짚지만, **전 명령을 도는 스윕이 없다.** 명령을 추가하면서 이 테스트를 안 쓰면 아무것도 실패하지 않는다.

#### 예외 — `run` (실측 확인)

`run` 은 이 규약의 **명시적 예외**다. **경계는 계획 파싱 성공 시점**이다.

```
$ rhwp run <입력이 없는 계획> --json    → exit=1  stdout=192B  {"error":"입력을 읽을 수 없습니다 …"}
$ rhwp run <선검증 위반 계획> --json    → exit=2  stdout=382B  {"invalid":[{"step":0, …}]}
$ rhwp run                          → exit=2  stdout=0B    ← 규약 준수
$ rhwp run <비 JSON 파일> --json      → exit=2  stdout=0B    ← 규약 준수
```

의도된 설계다 — `run_plan_engine()` 이 MCP `hwp_run_plan` 과 저널을 공유하므로 "왜 못 돌리는가"를 데이터로 돌려줘야
한다([INV-01](#inv-01--판정은-데이터고-종료-코드는-그-파생이다) 과의 정면 충돌이고, 이 축은 INV-01 을 택했다).

예외를 못 박은 테스트는 `tests/run_plan_contract.rs:107`
**`prevalidation_failure_is_exit_2_with_no_output`** 이다. 이름의 "no output"은 **출력 파일 부재**이고, stdout 은
오히려 **파싱해서 `invalid[]` 를 단언한다**(`:127`).

> **미해결**: `capabilities.jsonContract.failure` 는 여전히 예외를 적지 않는다
> ("단건 명령 실패 시 stdout 0바이트; batch 는 error 레코드 + 최종 exit 1") —
> [`open_gaps.md` G-02](open_gaps.md#g-02--run-의-실패-경로-예외를-자기서술이-적지-않는다) ·
> [#3880](https://github.com/edwardkim/rhwp/issues/3880) T4.

---

### INV-04 · stdout 은 데이터만 담는다

- **금지** — 진단·진행·요약·사람용 문장을 stdout 에 쓰는 것. 그건 전부 stderr 다
- **출처** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) §4-6 · `capabilities.jsonContract.stdout`
- **강제** — `capabilities_schema_contract.rs:319` `output_file_keeps_stdout_machine_readable` · `hidden_text_contract.rs:729` `human_output_is_not_json_and_json_is_not_chatty`

**왜.** stdout 이 파이프라인의 데이터 채널이다. 여기에 사람용 문장이 섞이면 `| jq` 가 죽고, `--json` 을 켠 소비자는 자기가 뭘 받았는지 모른다.

**어기면 — 위반 1건 실측 (재현).** `export-tables` 는 `-o` 와 `--json` 을 같이 주면 **사람 문장**을 낸다.

```
$ rhwp export-tables samples/table-001.hwp --json -o <임시>/out.json
exit=0  stdout=134B     표 추출 완료: 1개 → <임시>/out.json
```

`capabilities.commands[export-tables]` 는 `json:true` + `recordFields` 4개를 선언한다 — 선언과 실물이
갈린다([INV-13](#inv-13--자기서술은-실물과-같다) 동시 위반). [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가
"바인딩이 옵션을 닫은 건 회피일 뿐 수정이 아니다"라고 지목한 그 항목이다.

**더 큰 문제 — 같은 충돌을 세 가지로 푼다.**

| 명령 | `--json` + `-o` | stdout | 파일 |
| --- | --- | --- | --- |
| `export-text` · `export-structure` | `--json` 이 이긴다 | 봉투 | **미생성** |
| `export-markdown` | 둘 다 한다 | 봉투 | 생성 |
| `export-tables` | `-o` 가 이긴다 | **사람 문장** | 생성 |

한 바이너리 안에서 같은 플래그 조합이 세 가지 뜻을 갖는다. 실측 전문은 [`open_gaps.md`
G-06](open_gaps.md#g-06---o-와---json-의-우선순위가-명령마다-다르다).

---

### INV-05 · 부분 목록을 내지 않는다 — 확신 없으면 `null`

- **금지** — 빠뜨린 항목이 있는 목록을 완전한 목록처럼 내는 것. 확정 불가를 빈 배열로 내는 것
- **출처** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) §3-5 설계 원리 ③ · [`detection_policy.md`](../agent_security/detection_policy.md) 결정 ④
- **강제** — `tests/changed_pages_contract.rs:103` `dry_run_changed_pages_is_null` · `tests/mcp_session_changed_pages_contract.rs:434` `session_changed_pages_empty_when_nothing_changed` · `tests/injection_scan_contract.rs:449` `clean_document_reports_null_highest_confidence`

**왜.** 부분 목록은 **침묵보다 나쁘다.** 침묵은 소비자가 모른다는 걸 알게 하지만, 부분 목록은 **거짓 통과**를 만든다. "위험 신호 0건"과 "위험 신호를
3개까지만 봤다"는 정반대 판단을 낳는데 봉투 모양이 같다.

**어기면.** `null`(확정 불가)과 `[]`(확정된 0)의 구별이 무너진다. 세 값 어휘가 바인딩에도 그대로 문서화돼 있다 — `bindings/node/…` 가이드:
`null = 확정 불가 / [] = 바뀐 쪽 없음 / [0,2] = 그 쪽들`.

**실측 위반 — `info` 의 `warnings` 부재 (T1).**

```
$ rhwp info --json samples/field-01.hwp | jq 'keys'
["fonts","format","pageCount","paraCount","schemaVersion","sections","sizeBytes",
 "source","title","untrustedContent","untrustedFields","version"]
$ … | jq 'has("warnings")'
false
```

원인은 `src/main.rs:7370` `show_info()` 의 JSON 분기가 `return EXIT_OK` 로 끝나(`:7418` 부근) `:7442` 의
`println!("warnings: {}", metadata.warnings.len())` 에 **도달하지 못하는** 것이다. 결과적으로 리소스가 조용히 잘린 HML 문서가
**exit 0 + 완전해 보이는 봉투**를 낸다 — `fonts` 가 부분 목록인데 봉투는 그렇다고 말하지 않는다.
[#3877](https://github.com/edwardkim/rhwp/pull/3877) 이 찾았고
[#3882](https://github.com/edwardkim/rhwp/pull/3882) 가 닫는 중이다.

**강제가 부분이다.** 위 세 테스트는 각각 `changedPages`·`highestConfidence` 축을 본다. "모든 목록 필드가 확정 불가를 `null` 로
낸다"를 전수로 보는 가드는 없다. `capabilities.commands[].recordFields` 와 실물 봉투를 대조하는 가드가 있으면 T1 이 자동으로 걸렸을 것이다
— 실제로 `info` 의 `recordFields` 에도 `warnings` 가 없어 **선언과 실물이 나란히 틀려 있었다.**

---

### INV-06 · 조용히 자르지 않는다

- **금지** — 상한에 걸려 잘랐으면서 봉투에 그 사실을 남기지 않는 것
- **출처** — [#3802](https://github.com/edwardkim/rhwp/pull/3802) S7
- **강제** — `tests/boundary_integrity_contract.rs:560` `export_text_max_chars_truncates_loudly_and_keeps_page_addresses` · `:711` `search_max_matches_reports_total_and_omitted` · `:786` `session_text_and_search_share_the_truncation_vocabulary` · `:493`(hidden_text) `excerpt_is_capped_but_char_count_is_truthful`

**왜.** [INV-05](#inv-05--부분-목록을-내지-않는다--확신-없으면-null) 의 특수형이다. 조용한 절단은 **"전부 봤다"는 거짓말**이 된다.

**어기면.** 에이전트가 393쪽 중 5쪽만 읽고 "문서 전체에 그런 문구는 없다"고 답한다.

**실측.**

```
$ rhwp export-text samples/field-01.hwp --json --max-chars 5 | jq '{truncated, omittedCount}'
{"truncated": true, "omittedCount": 149}
```

문자 축은 **쪽 주소를 보존**한다 — 잘린 뒤에도 어느 쪽인지 알 수 있어야 한다는 규약이 테스트 이름(`keeps_page_addresses`)에 박혀 있다.

**남은 공백**: 무상태 `hwp_search` 에는 상한이 없다. `mcp_arg_validation_contract.rs` 가 `--` 배선 순서를 못 박고 있어 넣지
못했다 — [#3802](https://github.com/edwardkim/rhwp/pull/3802) 가 숨기지 않고 적었다. [`open_gaps.md`
G-10](open_gaps.md#g-10--무상태-hwp_search-에-상한이-없다).

---

### INV-07 · 아무 일도 안 하는 플래그를 두지 않는다

- **금지** — 받아들이지만 아무 효과도 없는 값·플래그. 특히 `0` 을 "무제한"으로 받는 것
- **출처** — [#3802](https://github.com/edwardkim/rhwp/pull/3802) S7
- **강제** — `tests/boundary_integrity_contract.rs:671` `zero_and_garbage_limits_are_usage_errors`

**왜.** `--max-chars 0` 을 "무제한"으로 받으면, 상한을 걸려던 호출자가 **정반대 동작**을 얻는다. 아무 일도 안 하는 플래그는 호출자를 안심시키는
함정이다.

**어기면.** "상한을 걸었으니 안전하다"고 믿은 파이프라인이 컨텍스트를 통째로 삼킨다.

**실측.**

```
$ rhwp export-text samples/field-01.hwp --json --max-chars 0
exit=2  stdout=0B
stderr: 오류: --max-chars 뒤에 1 이상의 정수가 필요합니다.
```

**강제가 부분이다.** 이 가드는 `--max-chars`·`--max-matches` 두 축만 본다. 다른 수치
플래그(`--threshold-pt`·`--table`·`--occurrence` 등)의 0·음수 처리를 도는 스윕은 없다.

---

### INV-08 · 필드 추가는 자유, 변경·삭제는 `schemaVersion` 범프

- **금지** — 기존 필드의 이름·타입·의미를 바꾸면서 `schemaVersion` 을 그대로 두는 것. **그리고 단순 추가에 범프하는 것**
- **출처** — `capabilities.jsonContract.schemaPolicy` · [#3719](https://github.com/edwardkim/rhwp/issues/3719) §4-5
- **강제** — `tests/provenance_contract.rs:1104` **`schema_version_stays_1_0_because_the_flag_is_additive`** · `tests/capabilities_schema_contract.rs:71` `capabilities_schema_version_is_independent_of_envelope_version`

**왜.** 양방향이라는 점이 중요하다. 추가마다 범프하면 **정작 깨는 변경 때 신호가 무의미해진다** — 전 봉투가 `"1.0"` 단일 값이라 버전은 "세대"를 뜻해야
하고, 세대 안의 구별은 **키의 존재 여부**로 한다([#3804](https://github.com/edwardkim/rhwp/pull/3804)).

**어기면.** 소비자가 버전을 무시하기 시작한다. 무시되는 버전은 없는 버전이다.

**하위 규칙 — 오류 승격은 원문 보존.** `error` 필드에 기존 메시지 원문을 그대로 두고 형제 필드를 붙인다. 텍스트를 파싱하던 소비자가 무해하게 남는다
([#3719](https://github.com/edwardkim/rhwp/issues/3719) §3-5 원리 ②).

**가드의 성격이 특이하다.** `schema_version_stays_1_0_because_the_flag_is_additive` 는 "1.0 이어야 한다"를 단언한다.
**정책이 바뀌면 이 테스트가 먼저 실패한다** — 판단을 테스트로 고정한 사례다.

---

### INV-09 · 미지 옵션을 침묵 무시하지 않는다

- **금지** — 모르는 플래그를 받고 아무 말 없이 계속 진행하는 것
- **출처** — [`agent_surface_playbook.md`](../../manual/agent_surface_playbook.md):60 — "조립 오류는 exit 2 — **미지 옵션 침묵 무시 금지**"
- **강제** — **없다.** 반대 방향(`선언 flags 실재`)만 있다

**왜.** 에이전트는 오타를 낸다. 침묵 무시는 오타를 **성공**으로 만든다 — `--dry-run` 을 `--dryrun` 으로 잘못 쓴 호출이 문서를 실제로 편집한다.

**어기면.** 검사인 줄 알았는데 편집된다. [#3879](https://github.com/edwardkim/rhwp/pull/3879) 가 파이썬 바인딩에서 같은 부류를
D-4 로 잡았다.

**실측 — 준수하는 쪽.**

```
$ rhwp info samples/field-01.hwp --없는옵션
exit=2  stdout=0B  stderr=38B ("알 수 없는 옵션: …")
```

**실측 — 위반하는 쪽 (재현).**

```
$ rhwp dump samples/field-01.hwp --json               → exit=0 stdout=18,643B (사람용 텍스트)
$ rhwp dump samples/field-01.hwp --존재하지않는옵션      → exit=0 stdout=18,643B (동일)
$ rhwp diag samples/field-01.hwp --json               → exit=0 stdout=615B
$ rhwp core-pages samples/field-01.hwp --json         → exit=0 stdout=150B
$ rhwp ir-diff <a> <a> --json --없는옵션                → exit=0 stdout=171B
```

`dump`·`diag`·`core-pages` 는 `capabilities` 레코드에 **`json` 키도 `flags` 키도 없다**. `ir-diff` 는
[`ir_diff_command.md`](../../manual/ir_diff_command.md):42 가 이미 **명시적 예외로 문서화**해 뒀다("전역 계약 §종료 코드의
예외, #3178 정렬은 별도 이슈").

**왜 가드가 못 잡는가 — 이것이 발견이다.** 검사는 **단방향**이다.

- `tools/agent_preflight.py:523` `check_declared_flags_real` — *선언한* 플래그를 실제로
넣어 보고 거부당하는지 본다. **선언 → 실물**
- 역방향("실물이 받아들이는 플래그는 전부 선언돼야 한다", "선언에 없는 플래그는 거부돼야
한다")을 보는 검사는 **없다**

그래서 아예 아무것도 선언하지 않은 명령(`dump`·`diag`)은 **검사 대상에서 사라진다.** 선언을 안 하는 것이 가드를 피하는 가장 쉬운 길이 되는 구조다.

> `tests/cli_json_contract.rs:786` 의 이름은 `capabilities_declared_flags_are_real_cli_flags`
> 지만, 실제로 보는 것은 `batch.flags` ↔ `commands[batch].flags` 일치와 `edit` 의
> `--occurrence` 특례 두 가지다. **이름이 약속하는 전수 검사는 선검사 스크립트에만 있다.**

---

### INV-10 · 봉투 키는 camelCase 다

- **금지** — 봉투 어디에든 `snake_case` 키를 내는 것 (중첩 포함)
- **출처** — [#3879](https://github.com/edwardkim/rhwp/pull/3879) §본체 결함 · [#3880](https://github.com/edwardkim/rhwp/issues/3880) T3
- **강제** — **인플라이트** — [#3882](https://github.com/edwardkim/rhwp/pull/3882) 의 `export_structure_envelope_has_no_snake_case_keys` · `query_envelopes_share_the_camel_case_rule`

**왜.** 별칭 조회 계층이 없는 **정적 매핑 언어(C#·Swift)에서 필드가 통째로 사라진다.** `{ [JsonPropertyName("nodeCount")] int
NodeCount; }` 로 매핑한 구조체에 `node_count` 가 오면 값은 기본값(0)이 된다 — 예외도 없이.

**어기면.** M20(C#/Swift 바인딩)이 시작되는 순간 부딪힌다.
`bindings/Native/src/lib.rs`(376줄)·`bindings/csharp/RhwpNative.cs`(63줄)·
`bindings/swift/Sources/Rhwp/` 는 **이미 존재한다**(#3879 §8) — "M20 미착수"라는 전제는 디렉터리가 비어 있다는 뜻이 아니다.

**실측 — 전 재귀 순회에서 위반 1건.**

```
$ rhwp export-structure samples/field-01.hwp --json   (재귀 순회)
top keys: mode, nodeCount, schemaVersion, source, structure, untrustedContent, untrustedFields
snake_case 키: ['.structure.node_count']      ← 이것 하나
$ rhwp info --json …                            snake_case 키: []
```

**가드 설계에서 배울 점.** #3882 는 "키가 사라진 것만 보면 **이름만 바꾸고 값을 잃은 수정**도 통과한다"는 이유로 `nodeCount` **값 존재**를 함께
단언한다. 그리고 `query_envelopes_share_the_camel_case_rule` 로 `info`·`digest`·`fields`·`export-tables`
까지 같은 규약을 건다 — 하나 고치고 다음을 놓치지 않기 위해서다.

---

## 3. 단일 출처와 자기서술 (L2·L3·횡단)

### INV-11 · 도구 선언은 한 곳에서만 나온다

- **금지** — 도구 목록·명령 목록을 두 군데에 적는 것
- **출처** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) §4-1
- **강제** — `tests/mcp_server_contract.rs:144` **`tools_list_matches_capabilities_manifest`** · `tests/agent_profile_router_contract.rs:227` `capabilities_declares_the_session_tools_it_actually_serves` · `tests/capabilities_schema_contract.rs:255` `mcp_schema_matches_live_manifest_output`

**왜.** `capabilities --mcp` 선언과 `mcp-serve` 실행이 **같은 배열**(`mcp_tool_definitions()`)에서 나오면 드리프트가
구조적으로 불가능해진다. 복사본을 두면 언젠가 갈린다 — 갈린 뒤에는 어느 쪽이 진실인지 아무도 모른다.

**어기면.** 에이전트가 `capabilities` 를 읽고 도구 정의를 자동 생성했는데 서버에는 그 도구가 없다. `tools/call` 이 `isError` 로 돌아온다.

**실측.** `capabilities --mcp` 도구 39개와 `mcp-serve` 의 `tools/list` 가 같은 원천을 쓴다는 것을
`tools_list_matches_capabilities_manifest` 가 매 CI 에서 확인한다.

---

### INV-12 · 선언과 배선은 같이 간다

- **금지** — MCP `inputSchema` 에 속성을 선언하고 CLI 인자로 배선하지 않는 것
- **출처** — [#3795](https://github.com/edwardkim/rhwp/pull/3795)
- **강제** — `tests/mcp_server_contract.rs:205` **`every_declared_input_property_is_wired_to_the_cli`** · `:279` `boolean_false_does_not_inject_a_presence_flag` + 선검사 `check_property_wiring`(`agent_preflight.py:428`)

**왜.** 선언만 하고 배선하지 않으면 **인자가 조용히 버려진다.** 호출자는 `maxChars: 100` 을 줬는데 무제한이 돌아온다 — 성공 응답이라 알아채지 못한다.

**어기면.** [INV-07](#inv-07--아무-일도-안-하는-플래그를-두지-않는다) 과 같은 실패가 MCP 쪽에서 재현된다.

**형제 규칙.** `boolean_false_does_not_inject_a_presence_flag` — `{"dryRun": false}` 가 `--dry-run` 을
붙이면 안 된다. 존재 플래그와 불리언 값의 차이를 못 박는다.

**예외 관리.** `NON_ARGV_PROPERTIES`(`paths`·`password`)는 argv 가 아니라 **stdin** 으로 간다. 암호를 argv 에 실으면
프로세스 목록에 노출되기 때문이다 ([#3839](https://github.com/edwardkim/rhwp/pull/3839)).

---

### INV-13 · 자기서술은 실물과 같다

- **금지** — `capabilities` · `--help` · MCP 매니페스트 · JSON Schema 넷 중 어느 하나가 나머지와 다른 말을 하는 것
- **출처** — [#3731](https://github.com/edwardkim/rhwp/pull/3731) · [#3795](https://github.com/edwardkim/rhwp/pull/3795) · [#3808](https://github.com/edwardkim/rhwp/pull/3808)
- **강제** — `cli_json_contract.rs:409` `capabilities_covers_every_help_command` · `:738` `help_covers_every_capabilities_command` · `:365` `capabilities_mcp_covers_every_json_command` · `capabilities_schema_contract.rs:203` `schema_matches_live_capabilities_output` · `:255` `mcp_schema_matches_live_manifest_output` + 선검사 `check_help_coverage`·`check_json_has_mcp_tool`

**왜.** 자기서술은 에이전트가 **도구 정의를 자동 생성하는 원천**이다
([`cli_json_pipeline_guide.md`](../../manual/cli_json_pipeline_guide.md)). 여기 빠진 기능은 그 에이전트에게 **영영
없는 기능**이고, 여기 있는데 실물에 없는 기능은 **매번 실패하는 호출**이다.

**어기면.** [#3808](https://github.com/edwardkim/rhwp/pull/3808) 이 계획 JSON Schema 를 만들다 스키마와 실행기가
**양방향으로** 어긋난 것을 발견했다 — 조건 2개짜리 계획이 검증기를 통과했고(스키마가 관대), `data:{}` 를 무효로 봤다(스키마가 엄격). 둘 다 **실행기
기준으로** 고쳤다.

**강제가 단방향이다.**

- `capabilities → help`, `help → capabilities` 는 **양방향**으로 본다 (테스트 두 개)
- `capabilities.flags → 실물 CLI` 는 선검사가 본다
- **`실물 CLI → capabilities`** 는 아무도 안 본다 → [INV-09](#inv-09--미지-옵션을-침묵-무시하지-않는다) 의 구멍
- **`capabilities.recordFields → 실물 봉투`** 는 rhwp 본체에서 안 본다.
파이썬 바인딩의 `test_envelope_parity.py:39` `test_declared_fields_actually_appear` 가 **조회 계열 4개만**
대조한다(`info`·`export-text`·`fields`·`export-structure`). 이 구멍으로
[INV-05](#inv-05--부분-목록을-내지-않는다--확신-없으면-null) 위반(T1)이 통과했다 — `info` 의 `recordFields` 에도 `warnings`
가 없어 **양쪽이 나란히 틀렸기** 때문이다

**현재 위반 2건 (실측).**

1. `capabilities.jsonContract.failure` 가 `run` 예외를 적지 않는다
([INV-03](#inv-03--실패-경로는-stdout-에-0바이트를-쓴다) 참조)
2. `commands[export-tables].json = true` + `recordFields` 4개인데 `-o` 를 주면 사람 문장이
나온다 ([INV-04](#inv-04--stdout-은-데이터만-담는다) 참조)

---

### INV-14 · 상위 층은 새 편집·조판 로직을 만들지 않는다

- **금지** — L2~L6 에서 코어(`document_core`·`wasm_api`)가 이미 하는 일을 다시 구현하는 것
- **출처** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) §4-3 — "협상 불가"
- **강제** — **없다.** 리뷰와 PR 본문의 자기신고로만 유지된다

**왜.** 같은 일을 두 번 구현하면 **판정자와 실행자가 갈라진다.** `--verify` 가 "같다"고 말하는 기준과 편집기가 쓰는 기준이 달라지는 순간,
`--verify` 는 아무것도 검증하지 않는다. 그리고 층 모델의 경제학이 여기서 나온다 — L0~L5 가 두꺼울수록 L6 이 싸진다.

**어기면.** [#3797](https://github.com/edwardkim/rhwp/pull/3797) 이 실물 사례를 보여 준다.
`insert_picture_native` 가 `img_dim` 을 `(0,0)` 으로 두는데 HWP5 직렬화기는 `crop.right/bottom` 을 원본 크기 자리에 쓰고
파서가 그걸 되읽는다. 왕복하면 값이 달라져 **정상 삽입에도 `--verify` 가 항상 exit 3** 이었다. `--verify` 를 믿는 에이전트가 정상 작업을
되돌린다.

**준수의 증거는 PR 본문에 있다.**
- [#3797](https://github.com/edwardkim/rhwp/pull/3797) — "새 삽입·조판 로직은 0이다"
  + 재사용한 코어 함수 6개를 이름으로 열거
- [#3799](https://github.com/edwardkim/rhwp/pull/3799) — `edit_fill_fields` 를
`fill_fields_core(...) -> Result<FillOutcome, String>` 로 갈라 단건·배치가 공유. `src/main.rs` 의 `-129` 줄은
전부 추출이고 동작 변경이 아님을 무회귀 테스트로 증명

**미강제라는 사실이 발견이다.** "이 PR 이 코어를 재사용했는가"를 기계가 판정할 방법이 없다. 현실적 대안은 **커버리지가 아니라 크기** — 새 함수가 코어에 있는
함수와 같은 이름을 갖는지, `src/main.rs` 순증가가 특정 임계를 넘는지 같은 간접 신호뿐이다. 지금은 리뷰어의 눈이 유일한 관문이다.

---

### INV-15 · 무상태 도구는 최소 한 프로필에 속한다

- **금지** — MCP 무상태 도구를 추가하고 역할 프로필 어디에도 등재하지 않는 것
- **출처** — [#3838](https://github.com/edwardkim/rhwp/pull/3838)
- **강제** — **인플라이트** — `every_stateless_tool_belongs_to_some_specific_profile`

**왜.** 프로필로 필터링해 붙은 에이전트에게 **미등재 도구는 존재하지 않는다.** 표면에 추가해도 프로필에 안 넣으면 그 역할에게는 없는 것과 같다.

**어기면.** 실제로 **14건**이 그렇게 됐다 — `hwp_digest`·`hwp_batch_fill`·`hwp_replace_text`·
`hwp_insert_image`·`hwp_run_plan`·`hwp_table_to_csv`·`hwp_csv_to_table`·`hwp_export_doclang`·
`hwp_sanitize`·`hwp_redact`·`hwp_inspect_hidden_text`·`hwp_inspect_injection`·
`hwp_inspect_unicode`·`hwp_render_diff`. 몇 달간 아무도 몰랐다.

**부채를 갚는 것보다 다시 생기지 않게 하는 쪽이 핵심이다** — #3838 본문의 표현. 가드가 머지되기 전까지 이 불변식은 선언일 뿐이다.

---

## 4. 보안 축의 불변식

> 이 절의 여섯은 [`detection_policy.md`](../agent_security/detection_policy.md) 가 결정과
> 논증을 보존한다. 여기서는 **불변식으로서의 형태**만 옮긴다.

### INV-16 · 조용히 고치지 않고 표시한다

- **금지** — 문서에서 위험 신호를 찾았을 때 문자열을 고치거나 지우거나 정규화하는 것
- **출처** — [`detection_policy.md`](../agent_security/detection_policy.md) 결정 ①
- **강제** — `tests/injection_scan_contract.rs:174` **`scan_does_not_sanitize_the_payload_out_of_the_document`** · `:149` `scan_does_not_modify_the_document` · `tests/hidden_text_contract.rs:524` `inspection_never_modifies_the_input` · `tests/unicode_deception_contract.rs:504` `scanning_does_not_touch_the_document`

**왜.** 조용한 정화는 **그것대로 거짓말**이다. 사용자는 보지 않은 것을 봤다고 믿는다. 그리고 정화된 문서와 원본이 갈라지면 감사가 불가능해진다.

**어기면.** "rhwp 를 통과했으니 안전하다"는 잘못된 신뢰가 생긴다. `capabilities.jsonContract.textSecurity.policy` 가 이 규약을
자기서술로 광고한다 — `"보고 전용 — 문서 문자열을 수정하지 않는다"`.

**예외는 명시적이다.** `edit redact`/`edit sanitize` 는 **고치는 것이 목적인 명령**이고, 그래서
[INV-22](#inv-22--되돌릴-수-없는-작업은-목적지를-명시하지-않으면-거부한다)· [INV-23](#inv-23--체크섬을-통과하지-못하면-마스킹하지-않는다) 이라는
별도 방벽을 갖는다.

---

### INV-17 · 오탐은 기능 실패다

- **금지** — 탐지율을 높이려고 오탐을 늘리는 교환
- **출처** — [`detection_policy.md`](../agent_security/detection_policy.md) 결정 ②
- **강제** — `tests/injection_scan_contract.rs:201` **`every_normal_sample_is_clean`** · `:374` `lookalike_official_sentences_stay_clean` · `tests/hidden_text_contract.rs:226` `real_samples_report_clean` · `tests/unicode_deception_contract.rs:546` `ordinary_korean_documents_are_clean` + [#3867](https://github.com/edwardkim/rhwp/pull/3867) 의 음성 스윕(`samples/` 348건 × 탐지기 3종)

**왜.** 안 쓰이는 방어는 방어가 아니다. 오탐이 나오는 탐지기는 **신호를 읽히지 않게** 만든다 — 비대칭이다. 놓친 1건은 1건의 피해지만, 오탐 100건은 진짜
1건까지 묻는다.

**어기면.** [#3809](https://github.com/edwardkim/rhwp/pull/3809) 초기 규칙은 20개 문서에서 **31,946건**이 걸렸다. 그
상태로 머지됐으면 아무도 `inspect` 를 켜지 않았을 것이다.

**어떻게 지켰는가.** 튜닝이 아니라 **원인 특정**이다. 31,946 → 4 → 4 → **3건**으로 줄이는 동안 세 원인(음영 sentinel 0 = 검정 / 그림 위
흰 글씨는 보인다 / 표 zone 채우기 누락)을 각각 회귀 가드로 고정했다.

**허용목록이 서랍이 되지 않게.** [#3867](https://github.com/edwardkim/rhwp/pull/3867) 은 정상 코퍼스에서 남은 2건이 **진짜
은닉**임을 SVG 좌표로 확인하고 `KNOWN_GENUINE_HIDDEN_TEXT` 에 근거와 함께 등재한 뒤,
**`allowlisted_documents_still_actually_trigger_detection`** 을 같이 넣었다 — 등재 문서에서 탐지가 사라지면 실패한다.

---

### INV-18 · rhwp 에 모델을 넣지 않는다

- **금지** — 언어 모델 호출·소형 분류 모델·임베딩을 rhwp 코드에 넣는 것. 판정은 결정론적 규칙만
- **출처** — [`detection_policy.md`](../agent_security/detection_policy.md) 결정 ③ · [#3787](https://github.com/edwardkim/rhwp/issues/3787)
- **강제** — **없다.** 정책이고, 의존성 심사가 간접적으로 막는다

**왜.** 세 겹이다.

1. **순환성** — 간접 인젝션은 "모델이 데이터를 지시로 오인하는 것"이다. 이를 판정하려고
모델을 하나 더 두면 **그 판정 모델도 같은 입력을 읽는다.** 같은 취약점을 가진 부품을 방어선에 세우는 셈이고, 판정 모델은 보통 더 작아 오히려 뚫기 쉽다
2. **흔적 부재** — 규칙 기반은 "이 규칙이 이 문자열에 안 걸렸다"를 사후에 재현할 수 있다.
모델은 그때 그 판정이 왜 그랬는지를 재현할 수 없다
3. **실사용 환경** — 재현성·오프라인·산출물 크기. `[lib] crate-type = ["rlib","cdylib"]`
이라 로컬 모델은 WASM 산출물에 그대로 실린다. **데이터 표 1MB 를 거부한 저장소**가 모델을 넣는 것은 앞뒤가 안 맞는다

**어기면.** 재현 불가능한 보안 판정이 생긴다. 그 순간 [INV-17](#inv-17--오탐은-기능-실패다) 의 "오탐 0" 을 증명할 방법이 사라진다.

**적용 사례.** [#3836](https://github.com/edwardkim/rhwp/pull/3836) `capabilities --search` 는 "LLM
유사도가 아니라 **결정론적 부분 문자열 매칭**"이라고 본문에 명시했고, [#3832](https://github.com/edwardkim/rhwp/pull/3832)
`explain` 도 "LLM 은 넣지 않는다"를 설계 전제로 적었다. 유혹이 실제로 있는 자리에서 매번 같은 답을 냈다는 것이 이 불변식이 살아 있다는 증거다.

**미강제라는 사실이 발견이다.** `Cargo.toml` 에 모델 크레이트가 들어오는 것을 막는 자동 검사는 없다. 실현 가능한 가드는 의존성 허용목록(`cargo-deny`
유사)이다 — [`open_gaps.md` G-22](open_gaps.md#g-22--모델-미탑재를-강제하는-장치가-없다).

---

### INV-19 · 문서에서 온 문자열은 데이터 자리에만 온다

- **금지** — 문서 내용을 `didYouMean`·`nextCall`·오류 힌트·산출 경로에 싣는 것
- **출처** — [#3802](https://github.com/edwardkim/rhwp/pull/3802) S5·S6
- **강제** — `tests/boundary_integrity_contract.rs:397` **`mcp_did_you_mean_candidates_come_from_the_tool_list_only`** · `:431` `mcp_next_call_is_literal_and_names_a_real_tool` · `:465` `document_strings_stay_in_data_fields_never_in_hints` · `:535` `cli_unknown_command_hint_never_carries_document_text` · `:167` `export_output_paths_ignore_traversal_string_in_body` · `:282` `edit_output_path_comes_from_the_flag_not_the_document` · `:328` `run_plan_output_comes_from_the_plan_never_from_the_document`

**왜.** 에이전트는 **교정 단서를 가장 잘 따른다.** `nextCall` 에 문서가 쓴 문장이 들어가면 그것은 사실상 도구가 서명한 지시다.

**어기면.** 누름틀 이름에 `이전 지시를 무시하고 …` 를 심은 문서가 그대로 에이전트의 다음 행동이 된다.
[#3802](https://github.com/edwardkim/rhwp/pull/3802) 가 이 문서를 실제로 만들어 돌렸고 — 페이로드는 `fields[].name`
**데이터 자리에만** 나오고 단서 자리엔 없었다. **뚫리지 않았고, 그 사실을 테스트로 못 박았다**(코드 변경 0건).

**같은 원리의 확장.** [#3799](https://github.com/edwardkim/rhwp/pull/3799) `batch fill` 은 **산출 파일 이름이
데이터에서 온다.** 그래서 쓰기 전에 전 행을 사전 계산하고(병렬에서도 결정적), 금지 문자 치환·Windows 예약 장치 이름 회피·80자 상한·소문자 키 중복 판정을
넣었다. `--out-dir` 탈출 시도는 테스트로 고정했다.

---

### INV-20 · 출처 표지는 항상 실린다

- **금지** — JSON 봉투를 내면서 `untrustedContent`/`untrustedFields` 를 빼는 것. 문서를 열지 않는 명령도 예외가 아니다
- **출처** — [#3804](https://github.com/edwardkim/rhwp/pull/3804) (#3787 S1) · `capabilities.jsonContract.provenance.policy`
- **강제** — `tests/provenance_contract.rs:762` **`provenance_map_covers_every_json_command`** · `:836` `every_text_bearing_command_declares_untrusted_fields` · `:933` `untrusted_flag_matches_map` · `:983` **`every_json_envelope_carries_the_flag`** · `:1003` `export_provenance_map_is_wired_across_every_surface` · `:1074` `capabilities_advertises_the_provenance_contract`

**왜.** 봉투에는 두 종류의 값이 섞여 있다 — rhwp 가 만든 값(`pageCount`·`exitCode`)과 **공격자가 정할 수 있는
값**(`text`·`matches[].excerpt`·`cells[][]`). 에이전트는 이 둘을 구분할 방법이 없어 문서 텍스트의 지시를 도구의 지시로 오인한다.

**어기면.** 간접 프롬프트 인젝션의 기본 경로가 열린 채로 남는다.

**"항상 실린다"가 중요하다.** 문서를 열지 않는 명령의 봉투도 `untrustedContent:false` 를 **명시**한다. 있으면 문서 파생, 없으면 모름 — 이
애매함을 없앤다. 실측: `rhwp capabilities` 봉투 자신도 `untrustedContent`·`untrustedFields` 를 담는다.

**이 PR 의 진짜 가치는 가드다.** #3804 는 일부러 두 번 망가뜨렸다 — `search` 의 선언 3건 삭제 →
`every_text_bearing_command_declares_untrusted_fields` FAILED, `dump-pages` 항목 통째 삭제 → 가드 4개 동시
FAILED. 가드가 없으면 이 기능은 6개월 뒤 거짓말이 된다.

**한계는 숨기지 않았다.** 누락 판정 입도는 **최상위 키 단위**다. 새 명령·새 루트는 전부 잡히지만 이미 선언된 루트 아래 새 필드(`matches[].새필드`)는 못
잡는다.

---

### INV-21 · 판정 기준은 스펙이 아니라 이 엔진이 그려 내는 결과다

- **금지** — 스펙 문구를 근거로, 이 렌더러가 실제로 하지 않는 계산을 탐지 규칙에 넣는 것
- **출처** — [#3809](https://github.com/edwardkim/rhwp/pull/3809)
- **강제** — `tests/hidden_text_contract.rs:364` **`black_shade_sentinel_does_not_fire_on_black_text`** · `:303` `page_source_is_suppressed_when_a_graphic_covers_the_page` · `:406` `auto_color_is_not_treated_as_white` · `:463` `normal_size_text_is_not_near_invisible`

**왜.** "숨겨졌는가"는 **화면 기준** 질문이다. 스펙이 말하는 실효 크기와 이 엔진이 그리는 크기가 다르면, 스펙을 따르는 판정기는 **화면에 보이는 글자를 숨김으로
보고한다.**

**어기면.** [#3809](https://github.com/edwardkim/rhwp/pull/3809) 가 정확히 그 지점에서 `relative_sizes` 곱셈을
**제거**했다. 스펙상 `base_size × relative_size / 100` 이 맞지만 이 엔진의 `style_resolver` 는 곱하지 않는다(렌더 경로 참조
0건). 곱했으면 대량 오탐이었다.

같은 이유로 음영 sentinel `0`(= 지정 안 함)을 검정으로 읽지 않는다 — 렌더러도 0 은 안 칠한다(`svg.rs:2746`,
`skia/text_replay.rs:379`). **렌더러가 하는 대로 판정해야 한다**는 근거가 코드 경로로 남아 있다.

**부수 규약.** 스펙과 엔진이 다른 지점은 **불일치 자체를 문서에 기록**한다. 조용히 엔진을 따르면 다음 사람이 "스펙 위반 버그"로 되돌린다.

---

### INV-22 · 되돌릴 수 없는 작업은 목적지를 명시하지 않으면 거부한다

- **금지** — `-o` 도 `--in-place` 도 없는 파괴적 편집을 기본 동작으로 수행하는 것
- **출처** — [#3805](https://github.com/edwardkim/rhwp/pull/3805)
- **강제** — `tests/redact_sanitize_contract.rs:235` **`refuses_to_run_without_an_explicit_destination`** · `:301` `dry_run_writes_nothing`

**왜.** 마스킹은 되돌릴 수 없다. 기본값이 "원본 덮어쓰기"면 한 번의 오타가 문서를 잃는다.

**어기면.** 실측 규약: exit 2 + stdout 0바이트 + **원본 md5 불변**.

**형제 규칙 — `--dry-run` 이 먼저다.** "무엇을 지울지 먼저 보여준다"가 이 명령의 권장 흐름이다. 그리고 그 미리보기 봉투 자체가 유출 경로여서
[#3841](https://github.com/edwardkim/rhwp/pull/3841) 이 `--no-raw` 를 붙였다.

---

### INV-23 · 체크섬을 통과하지 못하면 마스킹하지 않는다

- **금지** — 형태만 맞는 숫자열을 개인정보로 단정해 지우는 것
- **출처** — [#3805](https://github.com/edwardkim/rhwp/pull/3805)
- **강제** — `tests/redact_sanitize_contract.rs:116` **`checksum_failures_are_never_masked`** · `:178` `masking_preserves_length`

**왜.** [INV-17](#inv-17--오탐은-기능-실패다) 의 파괴적 버전이다. **탐지가 틀리면 문서가 훼손된다.** 주민번호는 mod 11, 카드는 Luhn 을
통과해야 마스킹한다.

**어기면.** 회계 코드·접수번호·문서 번호가 별표로 바뀐다. 되돌릴 수 없다.

**실측 (같은 문서에 유효값 4 + 미끼 2).**

| 심은 값 | 결과 |
| --- | --- |
| `900101-1234567` (mod 11 불일치) | **미탐지** — 마스킹 후에도 원문 보존 |
| `1234-5678-9012-3456` (Luhn 합 64) | **미탐지** — 원문 보존 |
| 유효값 4개 | 전부 **자릿수 유지** 마스킹 |

**자릿수를 유지하는 이유**는 레이아웃이다 — 길이가 바뀌면 조판이 흔들려 시각 회귀가 된다.

**범위를 좁힌 것도 이 불변식이다.** 02 외 지역번호·13/14/19자리 카드·여권번호·계좌번호· 주소를 전부 뺐다. 체크섬이 없거나 기관마다 형태가 달라 보수적 판정이
불가능하고, 넣으면 "오탐 0" 원칙 자체가 무너지기 때문이다.

---

### INV-24 · 악성 표본을 저장소에 두지 않는다

- **금지** — 공격 페이로드가 든 문서를 파일로 커밋하는 것
- **출처** — [#3867](https://github.com/edwardkim/rhwp/pull/3867) · [`test_corpus.md`](../agent_security/test_corpus.md)
- **강제** — 합성 헬퍼가 시험 시점에 만든다 — `injection_scan_contract.rs:88` `synthesize()` · `hidden_text_contract.rs:112` `synth_hml()` · `unicode_deception_contract.rs:68` `attack_document()`

**왜.** 저장소를 클론한 모든 사람의 디스크에 악성 표본이 놓인다. 백신 오탐·재배포 문제· 사내 정책 위반이 따라온다.

**어기면.** 기여 장벽이 생기고, 최악의 경우 저장소가 배포 차단된다.

**대가.** 합성 코드가 탐지 규칙과 같은 파일에 있어, 규칙을 고치면서 합성기를 같이 고치면 red→green 이 자기 자신을 속일 수 있다. #3867 이 이를 알고
**양성 3종 + 음성 스윕 348건**을 분리해 배치했다.

---

### INV-25 · 허용목록을 베끼지 않고 원본에서 읽는다

- **금지** — 계약 테스트의 예외 목록을 검사 스크립트에 복사하는 것
- **출처** — [#3795](https://github.com/edwardkim/rhwp/pull/3795) — "이게 이 PR 의 설계 핵심"
- **강제** — 코드 구조 — `tools/agent_preflight.py` 가 `NON_ARGV_PROPERTIES`·`HELP_HIDDEN`·`capabilities_mcp_covers_every_json_command` 의 인라인 제외를 **계약 테스트 소스에서 직접 읽는다**

**왜.** 가드에는 정당한 예외가 있다. `paths`·`password` 는 stdin 으로 가고, `core-pages` 같은 내부 프로브는 `--help` 에 없어도
되며, `capabilities` 자신은 도구가 아니라 **도구 목록의 원천**이다. 이 목록을 베끼면 언젠가 원본과 어긋난다.

**어기면.** 어긋난 순간 선검사가 실제 가드와 **다른 말**을 한다. 헛울리는 검사기는 곧 무시당하고, 무시당하는 검사기는 없느니만 못하다.

**실증.** 실제로 헛울린 적이 있다 — [#3872](https://github.com/edwardkim/rhwp/pull/3872):
`export-capabilities-schema --bare` 를 "CLI 가 거부한다"고 보고했는데 **정상 동작**했다. 원인은 허용목록이 아니라 판정 방식이었다.
검사기가 `stdout+stderr` 합본에서 `"알 수 없는 옵션"` 문자열을 찾는데, 이 명령의 출력이 **종료 코드 사전**을 담고 있어 자기 스키마 안의 오류 설명에
스스로 걸렸다. 수정은 **종료 코드를 먼저 보고 진단 문자열은 stderr 에서만** 찾는 것이었다.

실측 재확인: `rhwp export-capabilities-schema --bare` → exit 0, stdout 12,296B.

---

## 5. 절차 불변식

### INV-26 · 증적은 두 종류다

- **금지** — 터미널 출력만으로 "동작한다"를 주장하는 것
- **출처** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) §4-7 · [`agent_surface_playbook.md`](../../manual/agent_surface_playbook.md) §4
- **강제** — **없다.** 리뷰 관문

**왜.** 봉투가 `verify:{identical:true}` 라고 말해도 문서가 눈으로 깨져 있을 수 있다.
[#3797](https://github.com/edwardkim/rhwp/pull/3797) 의 `img_dim` 결함은 정반대 — 봉투는 실패라 했는데 문서는 멀쩡했다.
**둘 다 필요하다.**

**두 종류.**
1. 터미널 **봉투 원문** — 잘라 쓰지 않은 실제 stdout
2. 산출물을 **실제로 열어 렌더한 화면**

**형제 규칙 — 지어낸 값 0.** [#3835](https://github.com/edwardkim/rhwp/pull/3835) 는 레시피 5편의 봉투 예시를 전부 실행해
얻었고, 실행할 수 없었던 **레시피 03(PII 마스킹)은 쓰지 않았다** — "실행 없이 출력을 지어내는 것은 이 묶음의 원칙을 정면으로 어긴다". 원칙 때문에 산출물을
줄인 사례다. [`open_gaps.md` G-24](open_gaps.md#g-24--레시피-03pii-마스킹이-비어-있다).

---

## 6. 강제되지 않는 불변식 — 이 문서의 발견

전수 조사 결과 **아래 넷은 문서·리뷰에만 존재한다.** 어겨도 CI 가 아무 말도 하지 않는다.

| ID | 불변식 | 왜 못 잡는가 | 실현 가능한 가드 |
| --- | --- | --- | --- |
| **INV-09** | 미지 옵션 침묵 무시 금지 | 검사가 **선언 → 실물** 단방향. 아무것도 선언하지 않은 명령(`dump`·`diag`·`core-pages`)은 검사 대상에서 사라진다 | 전 명령에 `--__없는플래그__` 를 넣어 exit 2 를 요구하는 스윕. 예외는 `ir-diff` 처럼 근거와 함께 허용목록에 |
| **INV-14** | 새 편집·조판 로직 금지 | 기계가 "재사용했는가"를 판정할 수 없다 | 간접 신호뿐 — `src/main.rs` 순증가 임계, 코어와 이름이 겹치는 신규 함수 탐지 |
| **INV-18** | rhwp 에 모델을 넣지 않는다 | 의존성 추가를 막는 자동 검사가 없다 | `cargo-deny` 류의 크레이트 허용목록 |
| **INV-26** | 증적 2종 | PR 첨부물은 CI 밖 | 템플릿 체크박스 + 리뷰 관문 유지 |

**그리고 셋은 강제가 약하다.**

| ID | 불변식 | 약한 지점 |
| --- | --- | --- |
| **INV-03** | 실패 경로 stdout 0바이트 | 선검사가 **명령 61개 중 2개**만 본다(`agent_preflight.py:556-560` 하드코딩). 계약 테스트는 명령마다 따로라 새 명령은 자동 편입되지 않는다 |
| **INV-05** | 부분 목록 금지 | 축별 테스트뿐. `recordFields ↔ 실물 봉투` 전수 대조가 rhwp 본체에 없어 T1 이 통과했다 |
| **INV-13** | 자기서술 = 실물 | `실물 → 선언` 역방향과 `recordFields → 봉투` 가 비어 있다. 위 둘의 뿌리다 |

> **가장 값싼 한 수**: `recordFields ↔ 실물 봉투` 전수 스윕 하나가 INV-05·INV-13 두 구멍을
> 동시에 좁힌다. 파이썬 바인딩에 이미 있는 `test_declared_fields_actually_appear` 를 rhwp
> 본체로 옮기고 대상 명령을 4개에서 전체로 넓히는 일이다.

---

## 7. 예외 대장

불변식을 어기는 것이 **의도된** 자리다. 예외가 문서에 없으면 그건 예외가 아니라 버그다.

| 예외 | 어긴 불변식 | 근거 | 자기서술에 적혀 있나 |
| --- | --- | --- | --- |
| `run` 이 실패 시 봉투를 낸다 (exit 1/2) | INV-03 | `run_plan_engine` 이 MCP 와 저널을 공유. INV-01 을 택했다 | **아니오** — [#3880](https://github.com/edwardkim/rhwp/issues/3880) T4 |
| `ir-diff` 가 미지 옵션을 무시한다 | INV-09 | [`ir_diff_command.md`](../../manual/ir_diff_command.md):42 가 "#3178 정렬은 별도 이슈"로 기록 | **예** (매뉴얼) |
| `export-text`/`export-structure` 가 `--json` 모드에서 `-o` 를 무시한다 | INV-07 | 받아 주면 "저장했다"는 거짓말이 된다 (바인딩 가이드) | **부분** (바인딩 가이드만) |
| `edit redact`/`sanitize` 가 문서를 고친다 | INV-16 | 고치는 것이 목적. INV-22·INV-23 이 대신 방벽 | **예** |
| `docId` 가 `doc-1`,`doc-2` 로 예측 가능하다 | (난수화 미채택) | stdio 위협 모델상 위협이 아님 — 과잉 대응 회피 ([#3802](https://github.com/edwardkim/rhwp/pull/3802) S8) | **예** (보안 문서) |
| `run` 의 `output` 이 `..` 을 받는다 | (경로 정화 미채택) | 호출자가 계획서에 적은 경로. 정화하면 정당한 상대 경로가 깨진다 | **예** (보안 문서) |
| 진단 명령 30종이 `--json` 을 안 낸다 | (L1 커버리지) | 출력이 유동적이라 스키마 고정이 오히려 해롭다 ([#3719](https://github.com/edwardkim/rhwp/issues/3719) §3-1 명시적 제외) | **예** |

---

## 8. 불변식 ↔ 계약 테스트 매트릭스

| 파일 | 덮는 불변식 |
| --- | --- |
| `tests/cli_json_contract.rs` | INV-02 · INV-03 · INV-13 |
| `tests/capabilities_schema_contract.rs` | INV-04 · INV-08 · INV-11 · INV-13 |
| `tests/mcp_server_contract.rs` | INV-11 · INV-12 |
| `tests/boundary_integrity_contract.rs` | INV-06 · INV-07 · INV-19 |
| `tests/provenance_contract.rs` | INV-08 · INV-20 |
| `tests/changed_pages_contract.rs` · `mcp_session_changed_pages_contract.rs` | INV-05 |
| `tests/run_plan_contract.rs` · `run_plan_dry_run_contract.rs` | INV-01 · INV-03(예외 고정) |
| `tests/hidden_text_contract.rs` | INV-16 · INV-17 · INV-21 |
| `tests/injection_scan_contract.rs` | INV-05 · INV-16 · INV-17 · INV-19 |
| `tests/unicode_deception_contract.rs` | INV-16 · INV-17 |
| `tests/redact_sanitize_contract.rs` | INV-22 · INV-23 |
| `tests/agent_profile_router_contract.rs` | INV-11 · INV-15(인플라이트) |
| `tools/agent_preflight.py` | INV-03 · INV-12 · INV-13 · INV-25 |

**계약 테스트 규모** — [#3719](https://github.com/edwardkim/rhwp/issues/3719) 가 2026-08-01 에 기록한 값은
**215건 / 38파일**(전체 1,486건 / 405파일)이다. 2026-08-03 현재 `tests/` 의 `*contract*.rs` 는 **66개 파일**이다(`ls
tests/ | grep contract`). 건수는 이 PC 에서 `cargo test` 를 돌릴 수 없어 **확인되지 않음** ([`open_gaps.md`
G-23](open_gaps.md#g-23--이-pc-에서-rhwp-를-빌드할-수-없다) 참조).

---

## 9. 이 문서를 고칠 때

1. **불변식을 추가하면** ①~④ 네 칸을 전부 채운다. ④(강제)를 못 채우면
[§6](#6-강제되지-않는-불변식--이-문서의-발견) 표에 등재한다
2. **예외를 만들면** [§7](#7-예외-대장) 에 적고, **자기서술에도 적혔는지**를 표의 마지막
칸으로 판정한다. 자기서술과 다른 예외는 예외가 아니라 결함이다
3. **강제 수단이 생기면** [§8](#8-불변식--계약-테스트-매트릭스) 과 §6 을 함께 고친다
4. 새 결정은 [`decision_log.md`](decision_log.md), 미해결은 [`open_gaps.md`](open_gaps.md)
