# 요청 → 단 고르기

SKILL.md 의 사다리 표를 실행 순서로 펼친다.

```
사용자가 무엇을 원하는가?
│
├─ 작업 하나 / 영수증 / 3해시 / "증명해"
│   └─ replay attest  (± --capsule)
│       예제 01, 05
│
├─ 받은 산출이 맞는지 / 해시 대조
│   └─ replay --expect-output-sha256
│       예제 02, 03
│
├─ 작업을 이어서 기록 / 체인 / 부모
│   ├─ 다음 입력이 파일이어야 함 → 먼저 run
│   └─ replay --capsule --parent
│       예제 06, 07, 10
│
├─ 폴더 전수 / 재현율 / 회계
│   └─ audit <폴더>
│       예제 11–14
│
├─ 역사 / 계보 / 부모 산출=자식 입력
│   ├─ 해시 등식만 → lineage
│   └─ 링크마다 재실행 → lineage --deep
│       예제 15–18
│
├─ 누가 했는지 / 서명 / 귀속
│   └─ 거절. 3해시는 신원이 아니다 (pitfalls)
│
└─ gym 채점 / 새 명령
    └─ 거절. 기존 replay·audit·lineage 만.
```

## 한 줄 매핑

| 사용자 말 | 명령 |
|-----------|------|
| 이 작업 증명해 / 영수증 남겨 | `rhwp replay --plan-json … --json` |
| 타인 산출물 재현 검증 | `rhwp replay … --expect-output-sha256 <hex> --json` |
| 작업 캡슐 만들어 | `rhwp replay … --capsule a.capsule.json --json` |
| 체인 만들어 | `rhwp replay … --capsule b --parent a --json` |
| 캡슐 폴더 감사 / 재현율 | `rhwp audit <dir> --json` |
| 계보 검증 | `rhwp lineage <head> --json` |
| 링크마다 재실행 | `rhwp lineage <head> --deep --json` |

시나리오 전수는 `fixtures/scenario_catalog.json`.
