# 단건 영수증 — `rhwp replay`

권위: `src/main.rs` `cmd_replay`, `tests/audit_contract.rs`,
`mydocs/manual/agent_knowledge_map.md` §작업 영수증,
`mydocs/manual/agent_codex/50_검증_사다리.md` `replay` 절.

이 장은 **작업 하나**를 3해시로 고정하고, 같은 계획으로 다시 돌려 주장을
검증하는 경로만 다룬다. 폴더는 [audit-accounting.md](audit-accounting.md),
체인은 [capsule-chain.md](capsule-chain.md).

## 1. 두 모드

| 모드 | 조건 | `mode` | `reproduced` | 성공 exit |
|------|------|--------|--------------|----------:|
| 발급 (attest) | `--expect-output-sha256` 없음 | `attest` | `null` | 0 |
| 검증 (verify) | `--expect-output-sha256 <64hex>` | `verify` | true/false | 0 / **3** |

같은 명령이다. 플래그 하나가 모드를 가른다. `receipt` 명령을 만들지 마라.

## 2. 3해시

| 필드 | 대상 바이트 | 자주 하는 실수 |
|------|-------------|----------------|
| `inputSha256` | 계획서 `input` 파일 전체 | 텍스트 추출 해시로 대체 |
| `planSha256` | **계획 원문** (`--plan-json` 문자열 또는 파일 바이트) | pretty-print 한 JSON 을 같은 계획으로 착각 |
| `outputSha256` | 임시 재실행이 쓴 산출 파일 전체 | 사용자 `-o` 경로의 다른 파일 |

세 값 모두 SHA-256, 소문자 64 hex. 검증 플래그도 64 hex 가 아니면 exit 2
([exit-codes.md](exit-codes.md)). CLI 는 입력을 ascii lowercase 로 정규화한다.

`steps` 는 **숫자**다. `run` 저널의 `steps` 배열과 이름이 같다. 타입을 섞지 마라.

## 3. 발급 (attest)

```bash
rhwp replay --plan-json '{"planVersion":"1.0","input":"원본.hwp","output":"산출.hwp","steps":[{"action":"replace_text","find":"2025년","replace":"2026년"}]}' --json
```

위치 인자도 된다.

```bash
rhwp replay plan.json --json
```

봉투 골격은 `fixtures/envelopes/replay_attest.json`.

읽는 순서:

1. `mode == "attest"`
2. 세 해시가 각 64 hex
3. `toolVersion` 을 같이 적는다 ([pitfalls.md](pitfalls.md))
4. `reproduced` 는 `null` — 검증하지 않았다는 뜻이다. 성공이 아니다.

### 사용자 경로는 생기지 않는다

`replay` 는 계획의 `output` 을 **임시 경로로 덮어 실행**한 뒤 지운다
(`replay_execute_to_temp`). 영수증의 `outputSha256` 은 그 임시 파일의 해시다.

실산출이 필요하면 같은 계획으로 `rhwp run plan.json --json` 을 **따로** 돌린다.
`run` 의 `outputSha256` 과 `replay` 의 `outputSha256` 은 교차 결정론 위에서
같아야 한다 (`tests/lineage_contract.rs`).

## 4. 검증 (verify)

상대에게 받을 것:

- 계획 **원문** (바이트가 같아야 `planSha256` 이 같다)
- 주장 `outputSha256` (64 hex)
- `toolVersion` (선대조)

```bash
rhwp replay --plan-json '<같은 계획>' --expect-output-sha256 <64hex> --json
```

| 결과 | `reproduced` | exit | 다음 |
|------|--------------|-----:|------|
| 일치 | true | 0 | 주장 채택 |
| 불일치 | false | **3** | 주장 기각. 봉투를 근거로 보여 준다 |
| 값 길이·문자 위반 | (봉투 없음) | 2 | 호출을 고친다 |
| 입력/계획 IO | (실패 경로) | 1 | 경로를 고친다 |

불일치 표본: `fixtures/envelopes/replay_verify_mismatch.json`.

exit 3 을 크래시로 재시도하지 마라. 결정론이면 같은 입력이 같은 판정을 낸다.

## 5. 계획 원문 계약

`planSha256` 은 파싱된 객체가 아니라 **문자열 바이트**다.

- 키 순서, 공백, 개행, UTF-8 BOM 이 바뀌면 다른 계획이다.
- 제3자 검증은 상대가 준 원문을 그대로 `--plan-json` 에 넣는다.
- 에디터로 pretty-print 한 뒤 검증하면 실패가 아니라 **다른 작업**이다.

`cmd_replay` 는 `--plan-json` 이 있으면 위치 인자 파일보다 인라인을 이긴다.

필수 키: `input` (문자열). 없으면 exit 2. JSON 파싱 실패도 exit 2.

## 6. 허용 플래그 (발명 금지)

이미 있는 것:

- `--json`
- `--plan-json <json>`
- `--expect-output-sha256 <hex>`
- `--capsule <경로>` — [capsule-chain.md](capsule-chain.md)
- `--parent <경로>` — 캡슐과 함께
- `--sign-key <키>` — 캡슐과 함께만. 이 스킬 1부는 기본 경로에서 쓰지 않는다

없는 것 (만들지 마라):

- `--expect-input-sha256` (입력 대조는 영수증 필드와 audit 가 한다)
- `--recursive`
- `--receipt-only`
- `rhwp receipt` / `rhwp prove`

## 7. 워크스루

- [01_attest_three_hashes.md](../examples/01_attest_three_hashes.md)
- [02_verify_expect_output.md](../examples/02_verify_expect_output.md)
- [03_verify_mismatch_exit3.md](../examples/03_verify_mismatch_exit3.md)
- [04_plan_file_vs_inline.md](../examples/04_plan_file_vs_inline.md)
