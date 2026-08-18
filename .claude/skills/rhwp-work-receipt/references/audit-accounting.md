# 폴더 감사 — `rhwp audit` 재현율 회계

권위: `cmd_audit`, `collect_audit_capsules`, `tests/audit_contract.rs`.

개별 `replay` 가 작업 하나의 증명이라면, `audit` 는 폴더에 쌓인 캡슐의
**전수 재현율** 이다. 조직 단위 회계.

## 1. 호출

```bash
rhwp audit <캡슐 폴더> --json
```

플래그는 `--json` 뿐이다. `--recursive` 를 발명하지 마라.

## 2. 대상 규약 (비재귀)

`collect_audit_capsules`:

- `read_dir` 의 **직속** 항목만 본다.
- 파일명(lossy)이 `*.capsule.json` 으로 끝나면 대상.
- 이름 정렬 후 순서대로 재실행.
- `.json` / `.bak` / `.txt` / 하위 폴더 안 캡슐은 **세지 않는다**.

표본:

| 레이아웃 | 직속 캡슐 | 무시 | `total` |
|----------|----------:|------|--------:|
| `fixtures/audit-layouts/all-ok` | 3 | `notes.txt` | 3 |
| `fixtures/audit-layouts/nested-ignored` | 1 | `nested/hidden.capsule.json` | 1 |
| `fixtures/audit-layouts/mixed-ext` | 1 | `notes.json`, `*.bak`, `*.txt` | 1 |
| `fixtures/audit-layouts/empty` | 0 | README | (봉투 없음, exit 2) |

하위 폴더를 감사하려면 그 경로로 다시 `audit` 한다.

## 3. 회계 봉투

| 필드 | 타입 | 뜻 |
|------|------|----|
| `root` | string | 호출한 폴더 (에코) |
| `total` | number | 직속 `*.capsule.json` 수 |
| `reproduced` | number | 재현 성공 건수 — `replay` 의 bool 과 **동명 다른 타입** |
| `failed` | array | 실패 분개. 이름 + 사유 또는 기대/실측 해시 |
| `reproducedRate` | number | `reproduced / total` (0.0–1.0) |

공식은 나눗셈 한 줄이다. 가중치·재귀가 없다.

```
reproducedRate = reproduced / total
```

`total == 0` 이면 이 나눗셈에 도달하지 않는다. 사용법(exit 2)으로 거절한다.

## 4. 한 건을 재현으로 치는 조건

`validated_capsule_plan` 통과 후 임시 재실행:

1. 실제 입력 SHA-256 == `receipt.inputSha256` (아니면 `kind: inputSha256`)
2. 실제 step 수 == `receipt.steps` (아니면 `kind: steps`)
3. 실제 산출 SHA-256 == `receipt.outputSha256` (아니면 expected/actual)

셋이 같아야 `reproduced` 에 1을 더한다. 입력 변조는 산출 크레딧 **전에** 잡힌다
(`tests/audit_contract.rs` `tampered_input_receipt_is_caught_before_output_credit`).

`plan` 만 바꾸고 `planText` 를 그대로 두면
`plan 과 planText 불일치` 로 실패한다. 둘 다 바꿔도
`planText 와 receipt.planSha256` 불일치로 실패한다.

## 5. 종료 코드

| 상황 | exit | stdout |
|------|-----:|--------|
| 전부 재현 | 0 | 봉투, `failed: []`, rate 1.0 |
| 실패 ≥ 1 | **3** | 봉투. rate < 1.0. **회계를 읽어라** |
| 폴더에 대상 0개 | 2 | 0바이트 |
| 폴더 자체 읽기 실패 | 1 | 0바이트 |
| 인자 없음 / 미지 옵션 | 2 | 0바이트 |

exit 3 이어도 `reproduced` 는 성공 건수를 유지한다. 한 건의 변조가
나머지 노동을 지우지 않는다 — 실패 캡슐만 개별 `replay --expect-output-sha256`
으로 추적한다.

## 6. 워크스루

- [11_audit_all_ok.md](../examples/11_audit_all_ok.md)
- [12_audit_mixed_rate.md](../examples/12_audit_mixed_rate.md)
- [13_audit_non_recursive.md](../examples/13_audit_non_recursive.md)
- [14_audit_empty_exit2.md](../examples/14_audit_empty_exit2.md)
