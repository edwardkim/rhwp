---
kind: canonical
status: active
canonical: mydocs/tech/fuzzing/agent_surface_robustness.md
last_verified: 2026-08-03
---

# 에이전트 표면의 견고성 — 안 죽는 것만으로는 부족하다

> 퍼징은 "파서가 죽는가"를 묻는다. 그런데 rhwp 는 이제 **에이전트 도구**다.
> 에이전트 표면에서는 죽지 않는 것만으로 부족하다 — **잘못된 봉투를 내면
> 에이전트가 잘못된 판단을 한다.** 이 문서는 그 간극을 정의하고, 퍼징 자산을
> 그 간극을 메우는 데 어떻게 쓸지 정한다.

이 묶음의 canonical 문서다. 실행은 [operations.md](operations.md), 크래시 처리는
[crash_triage.md](crash_triage.md), 지도는 [README.md](README.md).

기준선: **2026-08-03**, `rhwp v0.8.2`, HEAD `9095cd52d`. 모든 주장에 코드 경로·테스트
이름·커밋 해시를 붙였고, 대지 못한 것은 **"확인되지 않음"** 으로 적었다.

---

## 0. 이 문서가 정하는 것

| 정한다 | 정하지 않는다 |
| --- | --- |
| 파싱 실패가 **exit code 와 봉투에 어떻게 나타나야 하는가** | 파서를 어떻게 고치는가 → [crash_triage.md](crash_triage.md) |
| 손상 문서에서 **부분 결과를 참인 것처럼 내지 않는다**는 계약 | 문서 내용이 에이전트를 조종하는 경로 → [agent_security/](../agent_security/README.md) |
| 퍼징 코퍼스를 **표면 계약 검증**에 쓰는 방법 | 인젝션 탐지 정책 → [detection_policy](../agent_security/detection_policy.md) |

---

## 1. 왜 "패닉 없음"이 충분하지 않은가

### 1-1. 두 성공 판정이 다르다

```
   퍼징의 성공 판정            에이전트 표면의 성공 판정
   ─────────────────           ────────────────────────────
   프로세스가 살아 있다        봉투에 적힌 것이 참이다
```

하네스 6개는 전부 이렇게 생겼다.

```rust
fuzz_target!(|data: &[u8]| {
    let _ = rhwp::parser::parse_hwp(data);
});
```
— `fuzz/fuzz_targets/parse_hwp.rs`. `fuzz/README.md` 가 그 의도를 명시한다:
"각 하네스는 `let _ = parse_xxx(data);` 형태로 **반환값을 무시**합니다."

**반환값을 버리는 하네스는 반환값이 틀린 것을 볼 수 없다.** 이건 결함이 아니라
설계다 — 퍼징의 일이 아니기 때문이다. 문제는 그 일을 **누구도 안 하고 있다**는 것이다.

### 1-2. 실패 모드 세 가지

| # | 모드 | 퍼저가 보나 | 에이전트에게 무슨 일이 | 예 |
| --- | --- | --- | --- | --- |
| **F1** | 죽는다 (패닉·abort·OOM·무한루프) | **본다** | 도구 호출이 exit 101 이나 무응답 → 판정 불능 | #3311 |
| **F2** | 거짓으로 성공한다 | **못 본다** | `Ok` + 그럴듯한 봉투 → **에이전트가 사실로 받아들임** | #2743 "조용한 구간" |
| **F3** | 성공했는데 봉투가 불완전하다 | **못 본다** | 부분 결과를 전부인 것으로 오인 → 거짓 통과 | 부분 목록 문제 |

**F2 가 이 문서의 존재 이유다.** 그리고 F2 는 가설이 아니라 **이 저장소에서 실제로
일어났던 일**이다. `tests/issue_2743_hml_resource_id_limit.rs` 주석:

> "**조용한 구간**: `Id="1000000"` → 힙 최대 120,009,531 바이트를 쓰고도
> `parse_hml` 이 **`Ok` 를 반환한다. 오류도 경고도 없어 호출자가 알 수 없다.**"

382바이트짜리 파일이 120MB 를 먹고 **성공으로 끝났다.** `-rss_limit_mb=2048` 은
2GB 를 안 넘었으니 크래시를 기록하지 않는다. 퍼저에게 이 입력은 **평범한 정상 입력**이다.

같은 파일의 회귀 테스트가 그래서 `is_ok()` 를 단언하지 않고 **결과 테이블 길이와
경고 개수**를 단언한다 — "그렇게 해야 수정 전에 red 가 된다"(테스트 주석).
**이 발상이 그대로 에이전트 표면 계약의 원리다.**

### 1-3. 그렇다고 퍼징이 덜 중요한 게 아니다 — 전제조건이다

F1 이 남아 있으면 F2·F3 를 논할 수 없다. 패닉은 **exit 101** 로 끝나는데,
그 코드는 [#2707 exit 사전](#2-1-exit-사전)에 **없다**. 사전에 없는 코드를 받은
에이전트는 성공인지 실패인지, 재시도해도 되는지 판정할 수 없다.

즉 순서는 이렇다: **퍼징이 F1 을 지우고 → 표면 계약이 F2·F3 를 지운다.**

---

## 2. 불변식 A — 파싱 실패는 exit code 와 봉투에 정확히 반영된다

### 2-1. exit 사전

[#3719](https://github.com/edwardkim/rhwp/issues/3719) §4 가 갱신한 사전(#2707 승계):

| 코드 | 의미 |
| --- | --- |
| 0 | 성공 |
| **1** | **런타임 실패 (읽기·파싱·렌더·쓰기)** ← 손상 문서가 오는 자리 |
| 2 | 사용법 오류 — 인자 없음/미지 옵션·명령/범위 초과, 계획 선검증 위반 |
| 3 | 검증 단언 실패 (`--verify` 계열) |
| 4 | `--verify-pages` 페이지 수 불일치 |

같은 §4 의 횡단 불변식 두 개가 이 축의 근거다.

> 2. **판정은 데이터** — 봉투 필드가 1차, exit 코드는 그 파생. **둘이 모순되면 버그다.**
> 6. **stdout 순수성** — 데이터만. 진단·진행·요약은 stderr.

### 2-2. 실패 불변식 — "stdout 0바이트"

파싱이 실패하면 stdout 은 **한 바이트도 나오지 않는다.** 부분 JSON 을 흘리면 소비자가
잘린 객체를 파싱하려다 또 다른 오판을 한다.

계약 테스트가 이미 이 불변식을 여러 표면에서 고정하고 있다(실측).

| 테스트 | 무엇을 고정하나 |
| --- | --- |
| `tests/cli_json_contract.rs::info_json_missing_file_exit_runtime_and_silent_stdout` | exit **1** + `stdout.is_empty()` — "실패 시 stdout 에 부분 JSON 을 흘리지 않는다 — 소비자는 stdout 만 파싱한다" |
| `tests/cli_json_contract.rs::info_json_multiple_files_exit_usage_silent_stdout` | exit **2** + stdout 0바이트 |
| `tests/digest_macro_contract.rs::digest_missing_file_exit_runtime_silent_stdout` | digest 축 동형 |
| `tests/digest_v2_contract.rs` (파일 주석) | "실패 시 stdout 0바이트, 종료 코드는 [#2707] 계약(0/1/2)" |
| `tests/boundary_integrity_contract.rs:539` | "실패 경로 stdout 은 0바이트여야 합니다" |
| `tests/capabilities_schema_contract.rs:351` | "알 수 없는 옵션은 사용법 오류(2), stdout 0바이트" |
| `tests/batch_fill_contract.rs:1373` | "인자 오류에서 stdout 은 0바이트여야 합니다" |

**그러나 이 테스트들은 전부 "없는 파일" 또는 "인자 오류"로 실패를 만든다.**
`grep -rn "0바이트\|stdout.is_empty" tests/` 실측 결과, **손상된 문서 바이트로
실패를 만드는 계약 테스트는 없다.** 없는 파일과 손상 파일은 다른 코드 경로다 —
전자는 `fs::read` 에서, 후자는 파서 안에서 실패한다.

이 공백이 §5 가 메우려는 것이다.

### 2-3. 패닉이 이 불변식을 깨는 방식

패닉이 나면:
- exit 은 **101** — 사전에 없다.
- stdout 은 그 시점까지 쓰인 것이 남는다 — `println!` 이 이미 실행됐다면 **부분 JSON 이
  나간다.** stdout 0바이트 불변식이 깨진다.
- stderr 에는 백트레이스가 나간다 — 에이전트가 이걸 "도구가 준 정보"로 읽는다.

`tests/issue_cli_test_caption_no_panic.rs` 가 정확히 이 관점의 테스트다.

```rust
assert_ne!(code, Some(101),
    "Rust panic(exit 101) 발생 — 범위 밖 인덱싱 회귀. stderr: {}", ...);
```

**패닉은 파서 문제이자 동시에 계약 위반이다.** 이게 퍼징 축과 에이전트 표면 축이
만나는 정확한 지점이다.

---

## 3. 불변식 B — 부분 결과를 참인 것처럼 내지 않는다

### 3-1. 원칙 — 부분 목록 금지

#3719 §4 횡단 불변식 4: **"부분 목록 금지 — 확정 불가는 `null`. 부분 목록은
침묵보다 나쁘다."**

[detection_policy.md](../agent_security/detection_policy.md) §④ 가 이유를 표로 정리한다.

| 응답 | 소비자가 읽는 것 | 실제 | 결과 |
| --- | --- | --- | --- |
| `null` | "모른다 — 전체를 보라" | 모름 | 안전 |
| **부분 목록** | **"이게 전부다"** | 일부만 | **거짓 통과** |

원문: "부분 목록은 **정밀한 척하는 것**이고, 정밀한 척은 소비자에게 '나머지는 안전'
이라는 잘못된 [확신을 준다]."

코드에도 같은 문장이 박혀 있다.

- `src/document_core/queries/changed_pages.rs:17` — "없으면 None — **부분 목록 금지** (#3630 P3)"
- `src/mcp_serve.rs:1011` — "대상 문단이 하나라도 조판 커버리지 밖이면 부분 목록 대신 `null`"
- `src/main.rs:14105` — "확정 불가면 null(부분 목록 금지)"

### 3-2. 지금 잘 되고 있는 것 — 절단은 봉투에 드러난다

[agent_boundary_contract.md](../agent_boundary_contract.md) S7 이 "산출량에 상한을 걸 수
있고, **절단은 반드시 봉투에 드러난다**"를 계약으로 세웠고, 봉투가 실제로 그렇다.

`digest --sections` 봉투(`src/main.rs` ~7260):

```json
{
  "schemaVersion": "1.0", "source": "...", "sectionCount": 12,
  "sections": [ ... ],
  "truncated": true,
  "nextStep": "절 원문은 export-text --json -p <쪽>, 찾으려면 search --json"
}
```

`truncated` 계산이 두 축을 모두 본다:

```rust
let truncated = any_truncated || section_count > sections.len();
```

— 개별 발췌가 잘렸거나(`any_truncated`), **절 자체가 빠졌으면**(`section_count >
sections.len()`) `true`. 이게 §3-1 원칙의 구현이다.

### 3-3. **실측 공백** — 손상 문서의 열화가 봉투에 안 나타난다

여기가 이 문서에서 가장 중요한 발견이다.

`info --json` 봉투는 `info_json_value()`(`src/main.rs:6993`)가 만들고, 필드는
**정확히 열 개**다.

```
schemaVersion · source · format · sizeBytes · version ·
sections · pageCount · paraCount · fonts · title
```

**`warnings` 가 없다.** 그런데 파서는 경고를 갖고 있다:

- `parse_hml` 은 `HmlParseResult.warnings` 를 돌려주고, `HmlWarningCode::InvalidReference`
  가 "상한 초과 리소스를 건너뛰었음"을 뜻한다
  (`tests/issue_2743_hml_resource_id_limit.rs` 가 개수를 단언한다).
- `show_info()`(`src/main.rs:7370`)는 **사람 모드에서만** 그것을 찍는다:

```rust
if json_mode {
    let info = info_json_value(file_path, file_size, detected_format, &doc);
    println!("{info}");
    return EXIT_OK;              // ← 여기서 끝난다
}
...
if let Some(metadata) = doc.hml_metadata() {
    println!("warnings: {}", metadata.warnings.len());
    for warning in &metadata.warnings { eprintln!("warning [{:?}] ...", ...); }
}
```

**JSON 모드는 경고 출력에 도달하지 못한다.** 데이터가 없어서가 아니라 배선이 없어서다 —
WASM 표면은 같은 정보를 JSON 으로 낸다(`src/wasm_api.rs:5770` 의 `"warnings": warnings`).

#### 그래서 무슨 일이 벌어지나

`Id="1000000"` 이 든 HML 을 에이전트가 `info --json` 으로 연다.

| | 일어나는 일 |
| --- | --- |
| 파서 | 상한 초과 리소스 6종을 **건너뛰고** `InvalidReference` 경고 6개를 남긴다 |
| exit | **0** |
| stdout | 완전한 형태의 봉투 — `sections`·`pageCount`·`paraCount`·`fonts` 전부 채워짐 |
| 봉투 안의 열화 신호 | **없음** |
| 에이전트의 결론 | "정상 문서다. 글꼴 목록은 이렇다." |

`fonts` 배열은 `document.doc_info.font_faces.first()` 에서 온다(`src/main.rs:7023`).
글꼴 리소스가 상한으로 잘려 나갔다면 **그 배열은 부분 목록**이고, 봉투는 그것이
부분이라고 말하지 않는다. §3-1 이 금지한 바로 그 형태다.

> **판정**: 이건 파서 결함이 아니다. 상한 기구는 의도대로 동작했다.
> **봉투 계약의 공백**이다. 퍼저는 이걸 절대 못 잡는다(F2).
> **확인되지 않음**: 이 공백이 이슈로 등록돼 있는지 저장소에서 확인하지 못했다.

#### 고치는 방향 (제안 — 결정 아님)

- `info --json` 에 `warnings: []` 추가 — #3719 불변식 5("필드 추가는 자유")에 부합하므로
  `schemaVersion` 범프 없이 가능하다.
- 또는 최소한 `degraded: true` 같은 **단일 불리언**. 부분 목록보다 낫다.
- HML 이외 포맷에도 같은 개념이 필요한가는 별도 조사가 필요하다 —
  HWP5/HWPX/HWP3 파서가 "건너뜀"을 구조적으로 보고하는지는 **확인되지 않음**.

---

## 4. 불변식 C — 손상 입력이 봉투 스키마를 바꾸지 않는다

[envelope_provenance.md](../envelope_provenance.md) 가 세운 표지 계약:

```json
{ "schemaVersion": "1.0", "source": "...", "matches": [ … ],
  "untrustedContent": true, "untrustedFields": ["matches[].text", "matches[].context"] }
```

봉투 안에는 **엔진이 만든 값**(`pageCount`·`bytes`·`exitClass`)과 **문서에서 온 값**
(`pages[].text`·`title`·`tables[].cells[].text`)이 섞여 나가고, 표지가 그 구분을 밝힌다.

퍼징 관점에서 이 계약에 붙는 질문은 하나다.

> **손상된 문서가 봉투의 구조 자체를 바꿀 수 있는가?**

[agent_security/disclosure.md](../agent_security/disclosure.md) §1 이 이걸 **취약점 부류 ②
"봉투 오염"**으로 분류한다 — "문서 내용이 봉투의 **구조나 메타 필드**를 바꾼다 …
소비자가 판정 자체를 신뢰할 수 없게 된다." 그리고 "①과 ②는 **언제나 취약점**이다."

무작위·손상 입력이 이 부류를 만들 수 있는 경로 후보:

| 경로 | 어떻게 | 지금 상태 |
| --- | --- | --- |
| 제목 문자열이 JSON 을 깨뜨림 | `title` 은 문서에서 온다(`document_title(doc)`) — 이스케이프 실패 시 봉투가 깨진 JSON 이 됨 | `serde_json` 이 직렬화하므로 이론상 안전. **손상 입력으로 시험된 적 없음** |
| 제어문자·불법 UTF-8 | 파서가 손상 바이트를 문자열로 만들 때 | 확인되지 않음 |
| 극단적 길이 | 문서에서 온 문자열이 수백 MB → 봉투가 컨텍스트를 범람 | S7 상한이 일부 축에만 있음 |
| `untrustedFields` 누락 | 새 필드가 표지 없이 추가됨 | `src/provenance.rs:74` 의 `pub const MAP` 이 단일 출처. 드리프트 가드 유무는 확인되지 않음 |

**이 표의 대부분이 "확인되지 않음"이라는 사실 자체가 결론이다** — 봉투를 손상 입력으로
때려 본 적이 없다.

---

## 5. 퍼징 코퍼스를 표면 계약 검증에 쓰기

### 5-1. 왜 되는가

퍼징 하네스는 파서 함수 하나만 부른다. **CLI 는 그 위에 층이 하나 더 있다.**

```
   fuzz 하네스              CLI 에이전트 표면
   ───────────              ─────────────────────────────
   parse_hwp(data)          fs::read → parse → 봉투 조립 → stdout/exit
        │                        │        │        │
   반환값 버림              (같음)   (같음)   ← 여기가 검증 안 됨
```

같은 입력을 **CLI 프로세스**에 먹이면 파서 층은 그대로 지나가고, 그 위 두 층
(봉투 조립·종료 코드)이 추가로 검증된다. **코퍼스는 이미 있다** —
`fuzz/corpus/` 12개 + 앞으로 쌓일 회귀 입력.

### 5-2. 무엇을 단언하나 — 계약 3개

무작위/손상 입력 하나를 `rhwp <명령> --json <파일>` 에 넣었을 때, 문서가 정상이든
쓰레기든 **항상** 참이어야 하는 것.

| # | 단언 | 근거 |
| --- | --- | --- |
| **A1** | exit code ∈ {0, 1, 2} — **101 이 나오면 실패** | #2707/#3719 exit 사전. 101 은 사전에 없다 |
| **A2** | exit ≠ 0 이면 **stdout 0바이트** | §2-2 의 기존 계약을 손상 입력으로 확장 |
| **A3** | exit == 0 이면 stdout 은 **유효 JSON** 이고 `schemaVersion` 을 포함 | `tests/cli_json_contract.rs` 의 `parse_stdout_json` 이 이미 쓰는 판정 |
| A4 | exit == 0 이면 그 명령의 **필수 필드가 전부 존재** | 명령별 봉투 필드 목록이 이미 선언돼 있다 — MCP 도구 정의의 마지막 인자(`src/main.rs:653`·`668`·`683`·`698` 등) |
| A5 | stdout 에 진단·경고 문자열이 섞이지 않는다 | #3719 불변식 6 stdout 순수성 |
| A6 | 프로세스가 유한 시간 안에 끝난다 | `-timeout` 의 표면판 |

A1~A3 이 핵심이고, 나머지는 있으면 좋다.

> **A1 이 특히 중요한 이유**: 퍼저는 패닉을 **크래시**로 보고 잡지만, 그건
> `fuzz/` 하네스를 돌릴 때만이다. CLI 경로에서만 나는 패닉
> (`issue_cli_test_caption_no_panic.rs` 가 잡은 것이 정확히 그런 것)은
> **어떤 하네스도 안 본다.** A1 이 그 사각을 덮는다.

### 5-3. 구현안 3가지

**안 ① — 코퍼스 재생 계약 스위트 (권장, 먼저)**

`cargo-fuzz` 없이 평범한 `#[test]` 로 `fuzz/corpus/**` 와 `fuzz/regressions/**` 전체를
읽어 CLI 프로세스에 먹이고 A1~A3 를 단언한다.

- 비용: 파일 수 × 프로세스 기동. 코퍼스가 12개면 수 초.
- 툴체인 추가 없음 — **stable 에서 돌고, 지금 CI 에 그대로 들어간다.**
- 모델로 삼을 파일: `tests/issue_3311_malformed_cfb_no_panic.rs`(케이스 목록 → 루프 →
  단언)와 `tests/cli_json_contract.rs`(프로세스 기동 + `parse_stdout_json`).
  **두 패턴을 합치면 그대로 나온다.**
- 이름 후보: `tests/fuzz_corpus_envelope_contract.rs`.

**안 ② — 손상 변이 스윕**

정상 샘플의 바이트를 규칙적으로 뒤집어(헤더 필드 치환·절단·구간 무작위화) 수백 케이스를
만들어 같은 단언을 돌린다. `tests/issue_3311_malformed_cfb_no_panic.rs` 가 이미
`real_field{off}_val{val}` · `real_truncated_1_over_{n}` 로 이 방식을 쓴다 — 177 케이스.
**시드 고정 의사난수**를 쓰면 재현 가능하고 CI 에서 안정적이다.

**안 ③ — 봉투 조립 함수 직접 퍼징**

`info_json_value`·`digest` 조립 함수를 부르는 fuzz 타깃을 추가한다.

- 장점: 프로세스 기동 없이 초당 수천 회.
- 걸림돌: 이 함수들이 `src/main.rs` 안에 있어 **`fuzz` 크레이트에서 접근할 수 없다**
  (바이너리 크레이트). 라이브러리로 끌어올리는 리팩터가 선행돼야 한다.
  `#3719` 불변식 3("새 로직 금지 — 상위 층은 검증된 코어 함수를 재사용한다")과
  방향이 같으므로 언젠가 할 일이지만, **지금 이걸 먼저 하는 건 순서가 틀렸다.**

### 5-4. 순서

```
① 코퍼스 재생 계약 스위트  ── stable, 지금 CI 에 들어감, 툴체인 0
        │                     ↳ operations.md §7-3 단계 A 와 같은 잡에 얹는다
        ▼
② 손상 변이 스윕           ── #3311 패턴 재사용
        ▼
③ 봉투 조립 함수 퍼징      ── main.rs → lib 리팩터 선행
```

①은 **[operations.md §7-3](operations.md) 의 "단계 A 회귀 코퍼스 재생"과 같은 작업**이다.
단계 A 가 파서 생존만 볼 것을 A1~A3 까지 보게 넓히면 된다 — **비용은 같고 얻는 게 많다.**

### 5-5. 하지 말아야 할 것

- **CLI 를 libFuzzer 하네스로 감싸지 마라.** 프로세스 기동 비용 때문에 exec/s 가
  두세 자릿수로 떨어져 변이 퍼징의 의미가 없어진다. 코퍼스 재생은 재생으로 족하다.
- **A3 를 "JSON 이면 아무거나"로 느슨하게 두지 마라.** `schemaVersion` 부재는
  소비자에게 스키마 협상 불능이다.
- **정상 문서에서만 계약을 시험하지 마라.** 지금 계약 테스트가 그 상태다 —
  `tests/cli_json_contract.rs` 의 `SAMPLE` 은 `samples/hwp3-sample.hwp`,
  주석 그대로 "파싱까지 성공하는 실제 샘플"이다.

---

## 6. 지금 있는 것 / 없는 것 (실측 요약)

| 계약 | 상태 | 근거 |
| --- | --- | --- |
| 실패 → exit 1 | **있음** | `cli_json_contract.rs`·`digest_macro_contract.rs` 등 |
| 실패 → stdout 0바이트 | **있음** (없는 파일·인자 오류에 한정) | 위 + `boundary_integrity_contract.rs:539` |
| **손상 문서**로 위 두 가지 검증 | **없음** | `tests/` 전수 grep 결과 손상 바이트 → 봉투 계약 테스트 0건 |
| 패닉(exit 101) 금지 | **부분** | `issue_cli_test_caption_no_panic.rs` 1개 명령만 |
| 손상 CFB 무패닉(라이브러리 층) | **있음** | `issue_3311_malformed_cfb_no_panic.rs` 177 케이스 |
| 절단이 봉투에 드러남 | **있음** | `digest` `truncated`, S7 계약 |
| 부분 목록 금지 | **있음**(changedPages 등) / **공백**(info 의 열화) | §3-1 vs §3-3 |
| 봉투 출처 표지 | **있음** | `untrustedContent`/`untrustedFields`, `src/provenance.rs` |
| 손상 입력에 대한 봉투 스키마 불변성 | **확인되지 않음** | §4 |
| 코퍼스 → 표면 계약 재생 | **없음** | §5 가 제안 |

---

## 7. 위협 모델과의 접합

[agent_security/threat_model.md](../agent_security/threat_model.md) §1.1 의 도식에서
① 은 퍼징, ② 는 보안 축이 맡는다. **그런데 F2·F3 는 정확히 그 사이에 있다.**

```
 ┌──────────────┐   ①    ┌────────────────┐   ②    ┌──────────────────┐
 │  손상 .hwp   │ ─────▶ │  rhwp 프로세스  │ ─────▶ │ 에이전트 컨텍스트 │
 └──────────────┘        └────────────────┘        └──────────────────┘
     퍼징이 봄            ▲            ▲             보안 축이 봄
                          │            └── 봉투 조립: 아무도 안 봄 ← 이 문서
                          └── 파서: 퍼징이 봄
```

두 축의 실패 정의를 나란히 두면 왜 이 문서가 별도로 필요한지가 분명해진다.

| 축 | "안전하다"의 뜻 | 실패했을 때 |
| --- | --- | --- |
| 퍼징(①) | 프로세스가 산다 | 크래시 · DoS |
| **이 문서** | **봉투에 적힌 게 참이다** | **에이전트가 거짓을 사실로 씀** |
| 보안(②) | 문서가 에이전트를 조종 못 한다 | 간접 프롬프트 인젝션 |

세 축은 같은 문장으로 이어진다: **죽지 않고(①), 참을 말하고(이 문서), 명령하지
않는다(②).** 셋 중 하나만 빠져도 에이전트 도구로서 신뢰할 수 없다.

---

## 8. 확인되지 않음

| 항목 | 왜 | 확인하려면 |
| --- | --- | --- |
| §3-3 의 `info --json` 경고 공백이 이슈로 등록됐는지 | 저장소 검색으로 특정 못 함 | 이슈 검색 후 없으면 등록 |
| HWP5/HWPX/HWP3 파서가 "건너뜀"을 구조적으로 보고하는지 | HML 만 `HmlWarningCode` 를 가짐 | 각 파서의 경고 기구 조사 |
| 손상 입력이 봉투 JSON 을 깨뜨릴 수 있는지 | 시험된 적 없음 | §5 안 ①·② 실행 |
| `untrustedFields` 에 드리프트 가드가 있는지 | `src/provenance.rs` 미조사 | 해당 파일과 계약 테스트 확인 |
| A1(exit 101 금지)을 전 명령에 대해 시험한 적 있는지 | `test-caption` 1건만 확인 | `capabilities` 의 명령 목록 × 손상 입력 |
| MCP 표면(`mcp-serve`)의 동일 불변식 | CLI 만 조사함. MCP 는 `isError` 어휘가 별도 | `src/mcp_serve.rs` 의 오류 경로 조사 |

---

## 9. 다음 조각 (제안 — 착수 시 개별 이슈로)

| # | 조각 | 크기 | 왜 지금 |
| --- | --- | --- | --- |
| 1 | `fuzz/regressions/` 생성 + 코퍼스 재생 계약 스위트(A1~A3) | 중 | operations.md §7-3 단계 A 와 한 몸. **stable 에서 돌아 CI 즉시 편입** |
| 2 | #3608 M21 현황판 갱신(하네스 6종 머지 반영) | 소 | 진행률이 실제보다 낮게 보인다 |
| 3 | `info --json` 열화 신호(§3-3) | 소 | 부분 목록 금지 원칙의 명백한 공백 |
| 4 | WMF 잔여 후보 2곳 이슈화([operations.md §5-3](operations.md)) | 소 | 코드 근거가 이미 확정됨 |
| 5 | 2순위 하네스(`parse_body_text_section`·`parse_doc_info`·`parse_control`·EMF) | 대 | RFC #3141 §4 잔여 |

---

## 관련

- [README.md](README.md) — 묶음 지도 · [operations.md](operations.md) — 실행 · [crash_triage.md](crash_triage.md) — 트리아지
- [agent_security/threat_model.md](../agent_security/threat_model.md) §1.1 — 신뢰 경계 ①/②
- [agent_security/detection_policy.md](../agent_security/detection_policy.md) §④ — 부분 목록 금지
- [agent_security/disclosure.md](../agent_security/disclosure.md) §1 — 봉투 오염 = 취약점 부류 ②
- [agent_boundary_contract.md](../agent_boundary_contract.md) — S5·S6·S7·S8
- [envelope_provenance.md](../envelope_provenance.md) — `untrustedContent`/`untrustedFields`
- [weak_agent_proofing.md](../weak_agent_proofing.md) — 에이전트의 무능(환각·검증 누락) 축
- 이슈: [#3608](https://github.com/edwardkim/rhwp/issues/3608) M21 · [#3141](https://github.com/edwardkim/rhwp/issues/3141) RFC · [#3719](https://github.com/edwardkim/rhwp/issues/3719) 6층 지도 · [#3787](https://github.com/edwardkim/rhwp/issues/3787) 경계 무결성 · [#3311](https://github.com/edwardkim/rhwp/issues/3311) 실제 발견 사례
