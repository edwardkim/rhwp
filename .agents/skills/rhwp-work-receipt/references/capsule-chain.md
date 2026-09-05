# 작업 캡슐과 해시 체인 — `--capsule` / `--parent`

권위: `cmd_replay` 캡슐 기록부, `tests/lineage_contract.rs`,
`tests/audit_contract.rs`.

캡슐은 계획+영수증의 **자기완결 교환 파일**이다. 제3자는 산출 HWP 없이
이 파일만으로 재실행 검증을 시작할 수 있다(입력 문서 바이트는 여전히 필요하다).

## 1. 발급

```bash
rhwp replay --plan-json '<계획A>' --capsule a.capsule.json --json
```

stdout 봉투는 일반 영수증과 같다. 추가로 파일이 생긴다.

골격 (`kind: workCapsule`):

```json
{
  "schemaVersion": "1.0",
  "kind": "workCapsule",
  "parent": null,
  "plan": { "planVersion": "1.0", "input": "…", "output": "…", "steps": [] },
  "planText": "<원문>",
  "receipt": { "inputSha256": "…", "planSha256": "…", "outputSha256": "…", "mode": "attest" }
}
```

감사·계보가 강제하는 불변식 (`validated_capsule_plan`):

- `planText` 의 SHA-256 == `receipt.planSha256`
- `plan` 객체 == `planText` 를 파싱한 값
- `receipt.steps` == `plan.steps` 길이
- `receipt.inputSha256` / `outputSha256` 는 64 hex

하나를 에디터로 고치면 감사가 실패한다. 그것이 계약이다.

## 2. 불변

**캡슐은 발급 후 불변이다.**

- 들여쓰기만 바꿔 저장해도 파일 바이트가 바뀐다.
- 자식이 기록한 `parent.sha256` 은 **부모 파일 바이트**의 SHA-256 이다.
- 부모가 바뀌면 `lineage` 의 `parentOk` 가 false 가 되고 exit 3.

수정하려면 같은 계획으로 **재발급**한다. 필드 패치는 체인을 끊는다.

표본: `fixtures/capsules/tamper_pretty_print.capsule.json`,
예제 [08_immutability.md](../examples/08_immutability.md).

## 3. `--parent` — 상대 경로는 캡슐 파일 기준

```bash
rhwp replay --plan-json '<계획B>' --capsule b.capsule.json --parent a.capsule.json --json
```

저장되는 값:

```json
"parent": { "capsule": "a.capsule.json", "sha256": "<부모 파일 64hex>" }
```

해석 규칙 (`cmd_replay` / `cmd_lineage`):

1. 부모 인자를 `canonicalize` 해 파일 바이트를 읽는다 (없으면 exit 1).
2. 저장 경로는 **캡슐 파일이 있는 폴더**를 prefix 로 벗겨 상대 경로를 만든다.
   다른 드라이브면 절대 경로가 남을 수 있다 — 같은 폴더가 가장 단순하다.
3. `lineage` 는 현재 캡슐의 parent dir + `parent.capsule` 으로 다음 파일을 연다.
   **호출 cwd 가 아니다.**

하위 폴더 표본: `fixtures/lineage-layouts/relative-subdir/`
(`child/b.capsule.json` → `../root/a.capsule.json`).

## 4. 같은 파일 거부

`--capsule` 과 `--parent` 가 같은 **기존 파일**을 가리키면 exit 2.
부모를 덮어쓰지 않는다.

```bash
# 거부
rhwp replay --plan-json '<계획>' --capsule a.capsule.json --parent a.capsule.json
```

## 5. 실산출이 필요할 때

다음 계획의 `input` 이 파일이어야 하면 `replay` 만으로는 부족하다.

```bash
rhwp run planA.json --json
rhwp replay --plan-json '<계획A>' --capsule a.capsule.json --json
# 계획B.input = planA.output
rhwp replay --plan-json '<계획B>' --capsule b.capsule.json --parent a.capsule.json --json
```

`lineageOk` 의 정의: 부모 `receipt.outputSha256` == 자식 `receipt.inputSha256`.
run 이 쓴 바이트와 replay 임시 재실행 바이트가 같아야 이 등식이 산다.

## 6. 서명은 이 장의 기본 경로가 아니다

`--sign-key` 는 `--capsule` 과 함께만 동작하고, 서명은 **사이드카**
(`*.sig.json`) 다. 캡슐 안에 넣지 않는다.

이 스킬 1부는 사이드카를 요구하지 않고, 서명됐다는 이유로 작성자를 주장하지
않는다 ([pitfalls.md](pitfalls.md)).

## 7. 워크스루

- [05_capsule_issue.md](../examples/05_capsule_issue.md)
- [06_parent_same_folder.md](../examples/06_parent_same_folder.md)
- [07_parent_relative_subdir.md](../examples/07_parent_relative_subdir.md)
- [08_immutability.md](../examples/08_immutability.md)
- [09_same_file_rejected.md](../examples/09_same_file_rejected.md)
- [10_run_then_chain.md](../examples/10_run_then_chain.md)
