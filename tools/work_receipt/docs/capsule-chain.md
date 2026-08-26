# 작업 캡슐과 --parent 체인

캡슐은 발급 후 **불변**이다. 에디터·포맷터가 공백 하나를 넣으면
자식이 기록한 `parent.sha256` 과 실물이 갈라지고 `parentOk=false` 가 된다.

## 필드

| 키 | 의미 |
| --- | --- |
| `kind` | 항상 `workCapsule` |
| `parent` | 뿌리이면 JSON null. 키 자체가 없으면 lineage fail-closed |
| `parent.capsule` | 부모 경로. 캡슐 파일 기준 상대 |
| `parent.sha256` | 발급 당시 부모 **파일** SHA-256 |
| `plan` | 파싱된 계획 (원본 output 경로 보존) |
| `planText` | 해시된 원문 |
| `receipt` | replay 봉투 |

`validated_capsule_plan` 은 다음을 이 순서로 본다.

1. `planText` 존재
2. `receipt.planSha256` 64hex
3. `sha256(planText) == receipt.planSha256`
4. `planText` JSON 객체
5. `plan ==` 파싱 결과
6. `receipt.steps` 음이 아닌 정수
7. `plan.steps` 배열 길이 == `receipt.steps`

산출이 같아도 이 가드 중 하나면 audit/lineage 는 실패한다.
`audit-layouts/plan-vs-text` 와 `plan-text-sha` 가 두 변조를 가른다.
