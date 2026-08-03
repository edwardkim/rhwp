---
kind: canonical
status: active
canonical: mydocs/tech/autonomous_maintenance/pre_registration_pattern.md
last_verified: 2026-08-04
---

# 선등재 패턴 — 병렬 PR 무충돌 접합

> 아직 없는 항목을 **미리 등재**하되, 순회 기준을 다른 곳에 두어 초과 항목이 잠들어
> 있게 한다. 상대 PR 이 머지되는 순간 자동으로 깨어난다. **어느 쪽이 먼저 머지돼도
> 후속 수정이 없다.**
>
> 로드맵상 [#3907](https://github.com/edwardkim/rhwp/issues/3907) **R92**.
> 실증은 [#3903](https://github.com/edwardkim/rhwp/pull/3903) ↔
> [#3808](https://github.com/edwardkim/rhwp/pull/3808).
> 축 지도는 [README](README.md), 착수 규약은 [병렬 세션 규약](parallel_session_protocol.md).

이 문서의 모든 주장에는 `파일:줄` 또는 PR 번호가 붙는다. 근거를 대지 못하는 항목은
**"확인되지 않음"** 으로 적었다(§8).

---

## 0. 한 줄 요약

**목록(등재)과 순회(반복)를 분리하면, 목록의 초과 항목은 무해하게 잠든다.**

```
        열린 PR A                     열린 PR B (내 것)
  SWEEP_EXEMPT 에 X 추가        완전성 가드 신설 + 호출표에 X 선등재
            │                                │
            └──────────┬─────────────────────┘
                       ▼
             어느 순서로 머지돼도
     A 먼저 → B 의 호출표에 X 가 이미 있음 (패닉 없음)
     B 먼저 → 호출표의 X 는 SWEEP_EXEMPT 에 없어 순회되지 않음 (잠듦)
```

---

## 1. 문제

### 1-1. 같은 표를 건드리면 나중 것이 충돌한다

병렬 PR 이 같은 상수 배열·매니페스트·목록을 건드리면 텍스트 충돌이 난다. 이 저장소에
실측 근거가 있다 — `../../orders/20260722.md` §"A단계 1차": *"17건 중 **14건이 같은
`mod.rs` 를 건드려** 오래된 순서로 cherry-pick 해도 8건만 적용되고 9건이 충돌했다."*

`../../manual/agent_surface_playbook.md` §2-2 가 그래서 *"기존 테스트 파일 수정보다
신설을 우선한다(병렬 PR 충돌 회피)"* 로 권한다. **파일을 나누면 충돌은 준다.**

### 1-2. 완전성 가드가 있으면 파일 분리로 안 된다

문제는 **파일을 나눠도 못 피하는 결합**이 있다는 것이다.

이 저장소의 계약 테스트는 **허용목록이 조용히 늘어나는 것**을 특히 경계한다. 반복되는
문구가 그 규율을 보여준다 — *"사유 없는 허용목록은 가드를 무력화하므로 사유를 강제한다"*
(`tests/provenance_contract.rs:302-304`), *"근거 없는 allowlist 는 가치가 없다"*
(`tests/issue_2724_passthrough_invalidation_guard.rs:941`).

그 규율의 강한 형태가 **완전성 가드**다. *"허용목록에 있는 모든 항목이 검사표에도
있어야 한다 — 없으면 패닉"*. 목적은 명확하다. **새 면제가 검사까지 조용히 면제받는
드리프트**를 막는 것이다.

그런데 이 가드는 병렬 PR 에 최악이다.

```
PR A: 허용목록에 X 추가 (검사표는 모름 — 그 PR 에 없는 파일이다)
PR B: 완전성 가드 신설 (검사표에 A·B·C 등재)

A 가 먼저 머지 → B 를 리베이스하면 X 가 검사표에 없어 **패닉**
B 가 먼저 머지 → A 를 리베이스하면 X 가 검사표에 없어 **패닉**
```

**어느 쪽이 먼저 머지돼도 나중 것이 깨진다.** 파일이 달라도(A 는 `tests/foo.rs`,
B 는 `tests/bar.rs`) 소용없다 — 결합이 텍스트가 아니라 **의미**에 있기 때문이다.
git 은 충돌을 보고하지 않고, CI 만 빨개진다.

---

## 2. 실증 — #3903 ↔ #3808

### 2-1. 두 PR 이 무엇을 했나

| | [#3808](https://github.com/edwardkim/rhwp/pull/3808) | [#3903](https://github.com/edwardkim/rhwp/pull/3903) |
|---|---|---|
| 제목 | `export-plan-schema` + 조건부 step | 출처 표지 빠진 봉투 5건 + 가드 4중 확장 |
| 개설 | 2026-08-02 12:51 UTC | 2026-08-03 15:20 UTC |
| 이 주제와 관련된 변경 | **`SWEEP_EXEMPT` 에 `export-plan-schema` 추가** | **`SWEEP_EXEMPT` 완전성 가드 신설** |
| 상태(2026-08-04) | 열림 | 열림 |

둘 다 `tests/provenance_contract.rs` 를 건드린다. 그리고 §1-2 의 정확한 형태다.

### 2-2. 무대 — `SWEEP_EXEMPT` 와 그 의미

`tests/provenance_contract.rs:302-305` (현행 `upstream/devel`):

```rust
/// 레시피를 만들 수 없어 스윕에서 빼는 명령과 **그 사유**.
///
/// 여기 넣어도 되는 것은 "문서를 입력으로 받지 않아 문서 오라클을 만들 수 없는"
/// 명령뿐이다. 사유 없는 허용목록은 가드를 무력화하므로 사유를 강제한다.
const SWEEP_EXEMPT: &[(&str, &str)] = &[
    ("build-from-ingest", "…"),
    ("export-ir-schema", "…"),
    ("export-capabilities-schema", "…"),
    ("export-agent-manifest", "…"),
];
```

devel 에서 이 상수는 **한 곳에서만 쓰인다** — 스윕 커버리지 검사의 제외 필터
(`tests/provenance_contract.rs:863`):

```rust
let uncovered: Vec<String> = json_commands(&cap)
    .into_iter()
    .filter(|n| !covered.contains(n.as_str()))
    .filter(|n| !SWEEP_EXEMPT.iter().any(|(c, _)| c == n))
    .collect();
```

#3903 이 발견한 결함이 여기 있다. **면제의 의미가 넓게 새고 있었다.**

> `SWEEP_EXEMPT` 는 "오라클을 만들 수 없다"는 뜻인데 **표지 검사까지 같이 면제**됐다
> (스키마 봉투 2종이 빠져나간 경로).

즉 `export-ir-schema`·`export-capabilities-schema` 가 출처 표지 없이 봉투를 내보내고
있었는데, 면제 목록에 있다는 이유로 **아무 가드도 그 봉투를 열어 보지 않았다.**

### 2-3. #3903 의 수정 — 면제 명령도 실제로 호출한다

#3903 이 신설한 테스트(전문은 `gh pr diff 3903 --repo edwardkim/rhwp`, 신설
`sweep_exempt_envelopes_still_carry_provenance_marks`):

```rust
/// [#3885] 스윕 면제는 "문서 오라클을 만들 수 없다"는 뜻이지 "표지를 안 실어도
/// 된다"가 아니다 — 종전에는 면제 명령의 봉투를 아무 가드도 열어 보지 않아, 스키마
/// 봉투 2종(export-ir-schema·export-capabilities-schema)이 표지 없이 나가는 것을
/// 아무도 몰랐다. 면제 명령마다 실제 호출로 표지 존재를 고정하고, 호출표 완전성을
/// SWEEP_EXEMPT 와 기계로 대조한다 — 새 면제가 표지 검사까지 조용히 면제받는
/// 드리프트를 막는다.
#[test]
fn sweep_exempt_envelopes_still_carry_provenance_marks() {
    let invocations: BTreeMap<&str, Vec<String>> = [
        ("export-ir-schema", vec![s("export-ir-schema"), s("--json")]),
        ("export-capabilities-schema", vec![…]),
        ("export-agent-manifest", vec![…]),

        // [#3808 선등재] 아직 devel 에 없는 명령 — 표는 SWEEP_EXEMPT 기준으로만
        // 순회하므로 초과 항목은 미사용으로 잠들어 있다가, #3808 이 그 명령을
        // 면제 목록에 넣는 순간 자동으로 검사에 편입된다(어느 쪽이 먼저 머지돼도
        // 후속 수정 없음 — 3-PR 누적 머지에서 이 항목 유무로 실측 확인).
        ("export-plan-schema", vec![s("export-plan-schema"), s("--json")]),

        ("build-from-ingest", vec![…]),
    ].into_iter().collect();

    for (name, _why) in SWEEP_EXEMPT {
        let args = invocations.get(name).unwrap_or_else(|| {
            panic!(
                "SWEEP_EXEMPT 의 {name} 이 표지 존재 검사 호출표에 없습니다 — \
                 이 테스트의 invocations 에 호출 방법을 더하세요"
            )
        });
        let out = run(args);
        assert_eq!(out.status.code(), Some(0), …);
        let env: Value = serde_json::from_slice(&out.stdout)…;
        assert!(
            env.get("untrustedContent").is_some() && env.get("untrustedFields").is_some(),
            "{name} 봉투에 출처 표지(untrustedContent/untrustedFields)가 없습니다"
        );
        assert_eq!(env["untrustedContent"], Value::Bool(false), …);
    }
}
```

**두 줄이 이 패턴의 전부다.**

```rust
for (name, _why) in SWEEP_EXEMPT {          // ← 순회 기준: SWEEP_EXEMPT
    let args = invocations.get(name)…       // ← 등재 목록: invocations
```

`invocations` 는 **조회표**이지 순회 대상이 아니다. 여기 `export-plan-schema` 가
들어 있어도, `SWEEP_EXEMPT` 에 그 이름이 없는 동안에는 `invocations.get()` 이 그
항목을 **아예 부르지 않는다.** 죽은 값이다.

### 2-4. #3808 쪽의 대응 변경

`gh pr diff 3808 --repo edwardkim/rhwp` 에서 `tests/provenance_contract.rs` 부분:

```diff
@@ -322,10 +322,16 @@ const SWEEP_EXEMPT: &[(&str, &str)] = &[
     (
         "export-agent-manifest",
-        "… capabilities·irSchema·provenanceMap 을 조립한 …",
+        "… capabilities·irSchema·provenanceMap·planSchema 를 조립한 …",
     ),
+    (
+        "export-plan-schema",
+        "문서를 입력으로 받지 않는 계획서 문법 스키마다. 인자가 --bare·-o·--json 뿐이고 \
+         --bare가 아닌 모드도 특정 문서가 아닌 스키마 봉투를 낸다. \
+         봉투 모양은 tests/plan_schema_contract.rs 가 따로 고정한다.",
+    ),
 ];
```

`SWEEP_EXEMPT` 에 항목이 들어오는 **그 순간** #3903 의 `for (name, _why) in SWEEP_EXEMPT`
루프가 `export-plan-schema` 를 돌기 시작하고, `invocations` 에서 호출 방법을 찾아
실제로 실행한다. **자동 편입이다.**

### 2-5. 대조군까지 확인됐다

#3903 본문이 기록한 실측:

> 세 열린 PR(#3808·#3897·이 PR)의 누적 머지 트리에서 `provenance_contract` 통과를
> 실측으로 확인했습니다(**선등재 전에는 정확히 그 완전성 패닉 1건이 났고, 선등재 후
> 0건** — 대조군까지 확인).

**대조군이 있다는 게 중요하다.** "충돌 안 났다"는 관찰만으로는 패턴이 작동했는지
애초에 충돌이 없었는지 구별할 수 없다. 선등재 항목을 뺀 트리에서 패닉 1건을
재현했으므로, **막힌 것이 무엇인지 특정된다.**

### 2-6. 무엇이 안 된 것도 기록한다

이 실증은 **누적 머지 트리(로컬)** 에서 확인한 것이지, 실제 머지로 확인한 것이 아니다.
2026-08-04 현재 #3808·#3903 **둘 다 열려 있다**. 실제 머지 순서에서의 결과는
**확인되지 않음**(§8).

---

## 3. 패턴의 형태

### 3-1. 세 요소

| 요소 | 역할 | #3903 사례 |
|---|---|---|
| **등재 목록** (registry) | 항목 → 처리 방법의 사전. **초과 허용** | `invocations: BTreeMap<&str, Vec<String>>` |
| **순회 기준** (basis) | 무엇을 검사할지 결정. 목록 **밖**에 있다 | `SWEEP_EXEMPT` |
| **완전성 가드** (guard) | 기준에 있는데 목록에 없으면 실패 | `unwrap_or_else(|| panic!(…))` |

**선등재가 성립하는 이유는 셋의 방향이 한쪽이기 때문이다.**
가드는 `기준 ⊆ 목록` 을 요구한다. `목록 ⊆ 기준` 은 **요구하지 않는다.**
그래서 목록의 초과분은 무해하다.

역방향 가드(`목록 ⊆ 기준`, 흔히 "stale 항목 회수")가 함께 있으면 이 성질이 깨진다 —
그게 §5 반례의 주된 형태다.

### 3-2. 성립 조건 6가지

여섯 개가 **모두** 만족돼야 한다. 하나라도 어긋나면 선등재는 위험하다.

| # | 조건 | 왜 |
|---|---|---|
| C1 | **순회 기준이 등재 목록 밖에 있다** | 목록을 직접 순회하면 초과 항목이 즉시 깨어난다 |
| C2 | **역방향 실재 검사가 없다** | "목록의 항목이 실제로 존재하는가"를 보면 선등재가 그 자리에서 실패한다 |
| C3 | **등재가 출력에 반향되지 않는다** | 선언이 곧 광고인 곳에서는 없는 것을 광고하게 된다 |
| C4 | **순서·인덱스에 의존하지 않는다** | 위치가 의미를 가지면 삽입이 기존 항목의 뜻을 바꾼다 |
| C5 | **초과 항목이 다른 가드를 약화시키지 않는다** | 이름 매칭이 넓으면 잠든 항목이 다른 검사를 조용히 끈다 |
| C6 | **상대 PR 의 항목 이름을 확정할 수 있다** | 이름이 바뀌면 선등재는 죽은 코드로 남는다 |

**C6 은 사회적 조건이다.** #3903 은 열린 #3808 의 diff 를 읽어 `export-plan-schema` 라는
이름을 확정했다. 상대 PR 이 없거나 이름이 미정이면 선등재할 대상이 없다. 그리고
상대가 이름을 바꾸면 선등재 항목은 영영 안 깨어난다 — **잘못된 성공**이므로 §7-2 의
표시 규율이 필요하다.

### 3-3. 성립하지 않는 형태

- **선언이 곧 동작인 곳** — `capabilities` 의 플래그 선언처럼, 선언 자체가 계약이고
  소비자가 그것을 읽어 호출한다. C3 위반.
- **래칫(ratchet) 규율이 걸린 허용목록** — "면제는 줄어들기만 해야 한다"는 설계에는
  거의 항상 실재 검사가 붙는다. C2 위반.
- **인덱스 결합 배열** — `enumerate()` 로 위치를 쓰는 목록. C4 위반.
- **잎 이름 매칭 예외 목록** — 이름이 겹치면 무관한 곳까지 예외가 된다. C5 위반.

---

## 4. 저장소 전수 조사 — 같은 구조가 또 있는가

`grep -rn "const [A-Z_]*: &\[" --include=*.rs` 와 사용처 추적으로 찾은 목록이다.
판정은 §3-2 의 C1~C6 대입 결과다.

| 구조 | 위치 | 순회 기준 | 판정 |
|---|---|---|---|
| `invocations` (면제 호출표) | #3903 신설 `tests/provenance_contract.rs` | `SWEEP_EXEMPT` | **적용됨 — 실증** |
| `CONDITIONAL_RECORD_FIELDS` | #3903 신설, 현재 빈 배열 | `capabilities.commands[].recordFields` | **적용 가능** |
| `meta_only_by_design` | `tests/agent_profile_router_contract.rs:270대` | `capabilities --mcp` 의 `tools[].name` | **적용 가능** |
| `PROFILES[].tools` | `src/agent_profiles.rs:64`~ | `mcp_tool_definitions()` (교집합) | **적용 가능** |
| `NON_ARGV_PROPERTIES` | `tests/mcp_server_contract.rs:193` | `inputSchema.properties` 키 | **적용 가능하나 위험(C5)** |
| `HELP_HIDDEN` | `tests/cli_json_contract.rs:837` | `capabilities.commands` + `--help` 토큰 | **조건부 — 분류에 베팅** |
| `CALLER_ECHO` | `tests/provenance_contract.rs:233` | 봉투 경로의 **잎 이름** | **위험 — 침묵 약화(C5)** |
| `EXEMPT` (#2724) | `tests/issue_2724_passthrough_invalidation_guard.rs:88` | 소스 스캔 + **실재 검사** | **반례 — 즉시 실패(C2)** |
| `PROFILES[].session_tools` | `src/agent_profiles.rs` | 봉투에 **그대로 반향** | **반례 — 광고 드리프트(C3)** |
| `commands[].flags` (capabilities) | `src/main.rs` 선언 | 선검사가 **실제 호출** | **반례 — 검사가 잡음(C2·C3)** |
| `KNOWN_TEXT_FEATURES` | `src/paint/json.rs:33` | `enumerate()` 인덱스 | **반례 — 순서 결합(C4)** |
| `MCP_STDIN_TOOLS` | `src/main.rs` (const, 3항목) | 선언 = 배선 공유 | **반례(추정) — C3** |
| `FAILURE_STDOUT_ALLOWED` | **없음** | — | **확인되지 않음** |

### 4-1. `CONDITIONAL_RECORD_FIELDS` — 적용 가능

#3903 이 함께 신설한 `declared_record_fields_actually_appear_in_envelopes` 의 예외 목록이다.

```rust
/// 조건부 필드는 **사유와 함께** CONDITIONAL_RECORD_FIELDS 에 적는다 — 사유 없는
/// 허용목록은 가드를 무력화한다. 가능하면 허용 대신 레시피가 그 필드를 실제로
/// 나오게 하는 쪽을 택한다(…).
const CONDITIONAL_RECORD_FIELDS: &[(&str, &str, &str)] = &[
    // (명령, 필드, 스윕 레시피가 그 필드를 못 내는 사유)
];
```

순회 기준은 `cap["commands"]` 의 `recordFields` 선언이고, 이 목록은 **제외 필터**다.
아직 없는 명령·필드 쌍을 미리 넣어 두면 조회되지 않아 잠든다 — **C1·C2 만족.**

> [!NOTE]
> 다만 이 목록은 **비어 있는 것이 정상 상태**로 설계됐다(주석이 "가능하면 허용 대신
> 레시피를 택한다"고 명시). 선등재로 채우면 **비어 있어야 할 목록에 항목이 생긴다** —
> 기술적으로 안전해도 설계 의도와 어긋난다. §7-3 참조.

### 4-2. `meta_only_by_design` — 적용 가능

`tests/agent_profile_router_contract.rs` 의 `every_stateless_tool_belongs_to_some_specific_profile`:

```rust
let missing: Vec<&String> = all_names
    .iter()
    .filter(|n| !meta_only_by_design.contains(n.as_str()) && !covered.contains(n.as_str()))
    .collect();
```

`all_names` 는 `capabilities --mcp` 의 실제 도구 목록이다. `meta_only_by_design` 는
그 위의 제외 필터이므로 **없는 도구 이름을 미리 넣어도 조회되지 않는다.**
C1~C6 전부 만족. **새 자기서술 도구를 추가하는 PR 과 병렬 작업할 때 쓸 수 있다.**

### 4-3. `PROFILES[].tools` — 적용 가능 (같은 구조체의 다른 필드는 반례)

프로필 필터는 **교집합**이다(`src/main.rs:382`):

```rust
let mut tools = mcp_tool_definitions();
if let Some(p) = profile {
    tools.retain(|t| {
        t["name"].as_str().map(|n| agent_profiles::allows_tool(p, n)).unwrap_or(false)
    });
}
```

순회 기준은 `mcp_tool_definitions()` 이고 `PROFILES[].tools` 는 **판정표**다. 없는 도구
이름을 미리 넣어도 `retain` 이 아무것도 남기지 않는다 — 잠든다.

그리고 이 저장소에는 **정확히 그 상황을 예상한 이력**이 있다:
[#3838](https://github.com/edwardkim/rhwp/pull/3838) *"프로필 도구 등재 누락 14건 +
재발 방지 가드"* — 새 무상태 도구를 추가하면서 어느 프로필에도 등재하지 않는 실수가
실제로 났다(`hwp_insert_image`). **새 도구를 추가하는 PR 과 프로필을 손보는 PR 이
병렬로 열리면 선등재가 정확한 해법이다.**

> [!WARNING]
> **같은 구조체의 `session_tools` 는 반례다.** §5-2 를 보라. 필드마다 판정이 다르다 —
> "이 목록은 안전하다"가 아니라 **"이 목록의 이 필드는 안전하다"** 로 말해야 한다.

### 4-4. `NON_ARGV_PROPERTIES` — 적용 가능하나 위험

```rust
// tests/mcp_server_contract.rs:193
const NON_ARGV_PROPERTIES: &[(&str, &str)] = &[
    ("paths", "자식 CLI stdin 으로 한 줄에 하나씩 흘려 넣는다(batch 계열)."),
    ("password", "민감값이라 argv 금지 — cli.passwordStdin 계약으로 stdin 전달."),
];
```

순회 기준은 도구의 `inputSchema.properties` 키(`:254` 의 `for key in props.keys()`)이고
이 목록은 제외 필터다. **역방향 실재 검사는 찾지 못했다** — 이 상수는 파일 안에서
`:193`(선언) `:254`(필터) `:269`(오류 메시지) 세 곳에만 등장한다. C1·C2 만족.

**그러나 매칭이 속성 **이름**이다.** 잠들어 있어야 할 항목이, 다른 도구가 우연히 같은
이름의 속성을 선언하는 순간 **그 도구의 배선 누락을 조용히 면제한다.** 이것이 C5 위반의
전형이고, 이 가드가 막으려는 사고가 정확히 그 형태다:

> `dryRun: true` 를 보냈는데 파일이 써지고 응답에는 `"dryRun": false` 가 오는 형태였다
> (#3712 이전 devel). 컴파일 에러도 런타임 오류도 없이 **계약만 거짓말한다.**
> — `tests/mcp_server_contract.rs:206-212`

**판정: 이름이 충분히 특이할 때만.** `paths`·`password` 같은 흔한 이름은 선등재하지 않는다.

### 4-5. `HELP_HIDDEN` — 조건부

```rust
// tests/cli_json_contract.rs:837
const HELP_HIDDEN: &[(&str, &str)] = &[
    ("core-pages", "…내부 프로브…"),
    ("dump-extents", "…렌더러 디버깅 전용…"),
    ("measure-width", "…문서 처리 명령이 아니다"),
];
```

정방향(`:872`)은 제외 필터라 C1 만족. **그런데 역방향 검사가 있다**(`:882-889`):

```rust
// 허용목록이 낡는 것도 같은 부류의 드리프트다 — help 에 실린 명령이 목록에 남아
// 있으면 "감췄다"는 설명 자체가 거짓이 되므로 지우게 만든다.
let stale: Vec<&str> = HELP_HIDDEN.iter().map(|(hidden, _)| *hidden)
    .filter(|hidden| help.iter().any(|h| h.as_str() == *hidden))
    .collect();
assert!(stale.is_empty(), "이미 --help 에 실린 명령이 HELP_HIDDEN 에 남아 있습니다: {stale:?}");
```

이 검사는 **"존재하는가"가 아니라 "help 에 실렸는가"** 를 본다. 그래서:

- 상대 PR 이 **`--help` 에 싣지 않는** 내부 프로브를 추가한다 → 선등재 **안전**
- 상대 PR 이 그 명령을 **`--help` 에 싣는다** → 선등재 항목이 **stale 로 잡혀 실패**

**즉 선등재가 "존재"가 아니라 "분류"에 베팅한다.** C2 를 부분적으로만 만족한다.
상대 PR 의 diff 에서 `print_help` 수정 여부를 확인했을 때만 쓴다.

### 4-6. `CALLER_ECHO` — 위험

```rust
// tests/provenance_contract.rs:233
const CALLER_ECHO: &[(&str, &str)] = &[
    ("source", "호출자가 준 입력 경로의 반향"),
    ("query", "search 검색어 — 호출자가 준 값"),
    …  // 12개
];

fn is_caller_echo(path: &str) -> bool {
    let leaf = path.rsplit('.').next().unwrap_or(path);
    let leaf = leaf.trim_end_matches("[]");
    CALLER_ECHO.iter().any(|(k, _)| *k == leaf)
}
```

**잎 이름 매칭이다.** `foo.bar[].name` 도 `baz.name` 도 똑같이 `name` 으로 본다.
초과 항목 하나가 봉투 전체에서 그 잎 이름의 출처 판정을 끈다 — C5 정면 위반.

#3903 이 이 위험을 실제로 만나 **경로 전체 매칭으로 우회**했다:

> `CALLER_ECHO` 에 **전체 경로 매칭**으로 등재했습니다 — 잎(`name`)으로 넓히면
> 문서 파생인 `fields[].name` 탐지까지 꺼져 가드가 약해지기 때문입니다.

**판정: 잎 매칭인 동안은 선등재 금지.** 전체 경로 매칭이 머지된 뒤에는 재판정한다
(경로가 특이하면 C5 를 만족할 수 있다).

### 4-7. `FAILURE_STDOUT_ALLOWED` — 그런 상수는 없다

이 이름의 상수를 `*.rs` 전체에서 찾지 못했다. 실패 경로 stdout 검사는 두 곳에 있는데
둘 다 허용목록이 없다.

- `tools/agent_preflight.py:562` `check_failure_stdout_silent` — **하드코딩된 `cases` 3개**
  (`info --json <없는파일>` 등)를 직접 실행한다.
- `capabilities` 의 `jsonContract.failure` 선언 — 이건 허용목록이 아니라 **계약 서술**이다
  (#3897 이 `run` 의 의도된 stdout 예외를 여기에 명시했다).

열린 PR [#3887](https://github.com/edwardkim/rhwp/pull/3887)(*"검사 범위 3경로 → 60경로"*)이
그 하드코딩을 표로 바꾸는데, **그 표에 허용목록이 생기는지는 확인되지 않음**(§8).
생긴다면 그때 이 항목을 재판정한다.

---

## 5. 반례 — 선등재가 위험한 곳

### 5-1. 실재 검사가 있는 목록 — `EXEMPT` (#2724)

`tests/issue_2724_passthrough_invalidation_guard.rs` 의 `stale_exemptions_are_reclaimed`:

```rust
for &(file, name, _, reason) in EXEMPT {
    let key = exempt_key(file, name);
    …
    match by_key.get(&key) {
        None => missing.push(format!(
            "  - {key} — 범위 내 `pub fn (&mut self)` 로 실재하지 않음(rename/제거/가시성 변경?)"
        )),
        …
    }
}
```

**선등재한 항목은 "실재하지 않음"으로 그 자리에서 잡힌다.** C2 정면 위반이고,
우회할 방법이 없다.

이 파일의 설계 의도가 그것을 못박는다:

> 면제는 **줄어들기만** 해야 한다(래칫). […] 방치하면 목록이 실재하지 않는 이름으로
> 채워져 가드가 무통보로 헐거워진다.

**"래칫"이라는 단어가 보이면 선등재를 시도하지 않는다.** 같은 파일은 `Exempt::Pending`
항목 수에 상한(`MAX_PENDING = 1`)까지 걸고, `Exempt::DelegatesTo(target)` 의 위임 대상이
실제로 무효화에 도달하는지도 검사한다(`delegation_targets_actually_invalidate`) —
**세 겹으로 실재를 강제한다.**

### 5-2. 선언이 곧 광고인 곳 — `PROFILES[].session_tools`

같은 `AgentProfile` 구조체인데 `tools` 와 판정이 정반대다. 이유는 **출력 반향**이다.
`src/main.rs` 의 매니페스트 조립부:

```rust
"profile": profile.map(|p| serde_json::json!({
    "name": p.name,
    "summary": p.summary,
    "session": crate::agent_profiles::opens_session(p),
    "sessionTools": p.session_tools.map(|t| if t.is_empty() { ALL_SESSION_TOOLS.to_vec() } else { t.to_vec() }),
    "recipe": p.recipe,
})),
```

`tools` 는 `retain` 으로 **걸러진 뒤** 나가고, `session_tools` 는 **그대로 나간다.**
없는 도구 이름을 선등재하면 **`capabilities --mcp --profile <역할>` 이 존재하지 않는
세션 도구를 광고한다.** C3 위반.

그 광고를 소비자가 어떻게 쓰는지가 문제의 크기를 정한다 —
`tests/agent_profile_router_contract.rs:224-225` 가 적는다:

> 자기서술도 실제 제공 집합과 같아야 한다 — 선언 7종, 실제 19종이면 **매니페스트로
> 도구 정의를 자동 생성하는 소비자가 실물과 다른 표면을 얻는다.**

**어느 테스트가 이 드리프트를 패닉으로 잡는지는 확인되지 않음**(§8) — 그러나
"기계가 안 잡으면 해도 된다"가 아니다. 계약 위반은 테스트 유무와 무관하다.

### 5-3. 선검사가 실제로 실행해 보는 곳 — `capabilities` 의 `flags`

`tools/agent_preflight.py:523` `check_declared_flags_real`:

```python
"""선언한 플래그가 실제로 수용되는지. 존재하지 않는 플래그를 문서에 적으면
에이전트가 그대로 호출했다가 usage error 를 맞는다."""
…
r = run([str(binary), name, str(flag)])
if r.returncode == 0:
    continue
blob = r.stderr
if "알 수 없는 옵션" in blob or "unknown option" in blob.lower():
    rep.fail(check, f"{name} {flag} — 선언됐지만 CLI 가 거부한다",
             "capabilities 선언을 지우거나 CLI 에 플래그를 구현하라.")
```

**선언한 플래그를 전부 실제로 호출해 본다.** 아직 없는 플래그를 미리 선언하면
그 자리에서 잡힌다. C2·C3 동시 위반.

`../../manual/agent_surface_playbook.md` §1 규칙 1 이 그 배경이다 — *"선언·실행·문서는
한 곳에서 갈라진다."* **선언이 단일 출처인 표면에서는 선등재가 곧 거짓 선언이다.**

### 5-4. 순서가 의미인 배열 — `KNOWN_TEXT_FEATURES`

`src/paint/json.rs:33` 선언, `:261` 사용:

```rust
for (idx, feature) in KNOWN_TEXT_FEATURES.iter().enumerate() {
```

**인덱스가 값의 일부다.** 중간에 항목을 끼우면 그 뒤 항목들의 인덱스가 전부 밀린다.
끝에 붙이면 인덱스는 안 밀리지만, 이 목록은 **순회 기준이 자기 자신**이므로(C1 위반)
초과 항목이 즉시 깨어난다.

**`enumerate()`·비트 플래그·배열 인덱스가 보이면 선등재하지 않는다.**

### 5-5. 선언과 배선이 같은 상수인 곳 — `MCP_STDIN_TOOLS`

```rust
/// stdin 으로 경로 목록을 받는 MCP 도구 — `capabilities --mcp` 의 `invocation.stdinTools`
/// 선언과 `mcp-serve` 의 자식 stdin 배선(`run_cli_tool`)이 이 목록 하나를 공유한다.
const MCP_STDIN_TOOLS: [&str; 3] = ["hwp_batch", "hwp_batch_search", "hwp_batch_extract_data"];
```

**하나의 상수가 선언과 배선 양쪽을 구동한다.** 없는 도구를 넣으면 선언에 광고가 실리고
(C3 위반), 배선 쪽에서 무슨 일이 나는지는 `run_cli_tool` 을 읽어야 안다 —
**확인되지 않음**(§8). 주석이 이 목록의 안전 목적을 명시하므로(자식이 서버의 프로토콜
stdin 을 상속해 JSON-RPC 프레임을 파일 경로로 소비하는 것을 막는다) **보안 축에 닿는
목록에는 선등재를 시도하지 않는다.**

---

## 6. 쓰는 법

### 6-1. 판정 절차

```
1. 상대 PR 의 diff 에서 추가될 항목 이름을 확정한다      → C6
   gh pr diff <n> --repo edwardkim/rhwp

2. 내 가드가 순회하는 것이 무엇인지 코드로 확인한다        → C1
   `for … in X` 의 X 가 등재 목록 자신이면 → 선등재 불가

3. 등재 목록에 역방향 검사가 있는지 grep 한다             → C2
   상수 이름으로 파일 전체 grep. "stale"·"실재"·"래칫" 이 보이면 중단

4. 등재 항목이 출력(봉투·매니페스트)에 반향되는지 본다      → C3
   src/ 의 조립부에서 그 목록이 직렬화되면 중단

5. enumerate()·인덱스 결합을 확인한다                     → C4

6. 매칭 입도를 확인한다 (전체 경로 / 이름)                 → C5
   이름 매칭이고 이름이 흔하면 중단
```

### 6-2. 등재할 때 반드시 적는 것

```rust
// [#<상대 PR 번호> 선등재] 아직 devel 에 없는 <대상> — 표는 <순회 기준> 기준으로만
// 순회하므로 초과 항목은 미사용으로 잠들어 있다가, #<번호> 가 <조건>을 만족하는
// 순간 자동으로 검사에 편입된다(어느 쪽이 먼저 머지돼도 후속 수정 없음).
```

**네 가지가 다 들어가야 한다.** 상대 PR 번호(C6 추적), 순회 기준(C1 근거),
잠든 상태라는 사실, 편입 조건. 이 주석이 없으면 다음 사람에게는 **정체불명의
죽은 항목**으로 보인다.

### 6-3. 검증 — 대조군을 만든다

선등재는 "아무 일도 안 일어나는 것"이 성공이라, 검증하지 않으면 성공했는지 알 수 없다.
#3903 이 한 방식을 그대로 쓴다.

```bash
# 1. 열린 PR 브랜치를 오래된 순서로 누적 머지한 임시 트리를 만든다
#    (playbook §2-5 의 누적 머지 충돌검사와 같은 트리)
# 2. 대상 계약 테스트를 돌린다 → 통과해야 한다
cargo test --profile release-test --test provenance_contract
# 3. 대조군: 선등재 항목만 지우고 다시 돌린다 → 예상한 실패가 나야 한다
```

**3번이 핵심이다.** 실패가 안 나면 애초에 충돌이 없었던 것이고, 선등재는 불필요한
죽은 코드였다는 뜻이다.

### 6-4. PR 본문에 적는다

#3903 은 별도 절(`## #3808 과의 접합 — 선등재로 무수정 통과`)을 뒀다. 리뷰어가
**정체불명의 초과 항목을 지적하지 않도록** 하는 것이 목적이다. 최소한 이 셋:
상대 PR 번호 / 잠드는 이유 / 대조군 결과.

---

## 7. 위험과 한계

### 7-1. 상대가 안 오면 죽은 코드가 남는다

상대 PR 이 close 되거나 항목 이름이 바뀌면 선등재 항목은 **영영 안 깨어난다.**
컴파일 에러도 테스트 실패도 없이 남는다 — **가장 조용한 실패 형태**다.

완화: §6-2 의 주석이 상대 PR 번호를 담으므로, 그 PR 이 닫히면 회수 대상임을 알 수 있다.
**자동 회수 수단은 없다** — 확인되지 않음.

### 7-2. "충돌을 없앤다"가 아니라 "가드 패닉을 없앤다"

선등재는 **의미 결합**을 푸는 기법이다. 두 PR 이 `SWEEP_EXEMPT` 배열 자체를 같은
줄 근처에서 고치면 **여전히 텍스트 충돌**이 난다. #3903↔#3808 이 무충돌이었던 것은
서로 다른 것을 고쳤기 때문이다 — #3808 은 `SWEEP_EXEMPT`(상수), #3903 은 새 테스트 함수.

**같은 배열에 둘 다 항목을 추가하는 상황은 선등재로 못 푼다.** 그건 적층이나
순서 대기의 몫이다([병렬 세션 규약](parallel_session_protocol.md) §5-3).

### 7-3. 허용목록 규율과의 긴장

이 저장소는 허용목록이 커지는 것을 계속 경계한다 — *"allowlist 가 커지면 가드가
무의미해진다"*(`tests/mcp_server_contract.rs:191-192`). 선등재는 **아직 필요하지 않은
항목을 목록에 넣는 행위**라 그 규율과 정면으로 긴장한다.

세 가지로 완화한다.

1. **제외 목록이 아니라 조회표에만 선등재한다.** #3903 이 넣은 곳은 "면제 목록"이
   아니라 "면제 명령을 어떻게 호출하는가"의 사전이다. **면제를 늘린 게 아니다.**
   §4-1 의 `CONDITIONAL_RECORD_FIELDS` 를 유보한 이유도 이것이다 — 그건 제외 목록이다.
2. **주석에 상대 PR 번호를 박는다.** 정당화가 코드에 남는다.
3. **상대 PR 이 닫히면 회수한다.**

### 7-4. 리뷰어에게는 정체불명이다

선등재 항목은 **그 PR 만 봐서는 왜 있는지 모른다.** 리뷰어가 "쓰이지 않는 항목"으로
지적하는 것이 정상 반응이다. §6-2·§6-4 의 표시 규율은 예의가 아니라 **작동 조건**이다.

---

## 8. 확인되지 않음

| # | 항목 | 상태 |
|---|---|---|
| 1 | #3903·#3808 의 **실제 머지 순서에서의 결과** | 2026-08-04 현재 둘 다 열려 있다. 실증은 로컬 누적 머지 트리 |
| 2 | #3903 이 실측했다는 "3-PR 누적 머지" 트리의 **재현** | PR 본문 기술을 근거로 인용. 직접 재현하지 않음 |
| 3 | `PROFILES[].session_tools` 에 없는 도구를 넣었을 때 **패닉하는 테스트가 있는지** | 계약 위반은 확실하나 어느 가드가 잡는지 특정 못 함 |
| 4 | `MCP_STDIN_TOOLS` 에 없는 도구를 넣었을 때 `run_cli_tool` 의 동작 | 코드 경로 추적 안 함 |
| 5 | `NON_ARGV_PROPERTIES` 의 역방향 실재 검사 **부재** | 상수 이름 grep 3곳으로 판단. 다른 파일에서의 사용은 확인 못 함 |
| 6 | [#3887](https://github.com/edwardkim/rhwp/pull/3887) 이 실패 경로 검사에 **허용목록을 신설하는지** | 변경 파일이 `tools/agent_preflight.py` 하나인 것만 확인 |
| 7 | 선등재 항목의 **자동 회수 수단** | 없는 것으로 보이나 확인 못 함 |
| 8 | `tools/agent_preflight.py:387` `load_allowlist` 가 **선등재 항목을 어떻게 다루는지** | 정규식으로 Rust 소스에서 목록을 읽으므로 초과 항목도 함께 읽힌다. 그것이 선검사 판정을 바꾸는지는 확인 못 함 |

---

## 9. 관련

- [자율 유지보수 문서 지도](README.md) — 이 축의 진입점
- [병렬 세션 규약](parallel_session_protocol.md) §5-3 — 접합 선택지(선등재 / 적층 / 순서 대기)
- [드리프트 자동 감지](drift_detection.md) — §4 의 완전성 가드들이 감지 대상이다
- [야간 자율 정비 사이클](nightly_cycle.md) — 무인 사이클이 병렬 PR 을 열 때 §6-1 판정 절차가 선행 조건
- [#3907](https://github.com/edwardkim/rhwp/issues/3907) R92 — 로드맵상 위치
- [#3903](https://github.com/edwardkim/rhwp/pull/3903) · [#3808](https://github.com/edwardkim/rhwp/pull/3808) — 실증 두 PR
- [#3885](https://github.com/edwardkim/rhwp/issues/3885) — #3903 의 대상 이슈
- [#3838](https://github.com/edwardkim/rhwp/pull/3838) — 프로필 등재 누락 14건(§4-3 배경)
- `../../manual/agent_surface_playbook.md` §1 규칙 1 · §2-2 · §2-5 — 단일 출처·테스트 파일 신설·누적 머지 충돌검사
- `../agent_security/threat_model.md` — 근거 규율의 문체 원본
- `../../orders/20260722.md` — 같은 파일 다중 수정의 충돌 실측(17건 중 14건)
