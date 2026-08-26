# 계보 — `rhwp lineage` / `lineage --deep`

권위: `cmd_lineage`, `tests/lineage_contract.rs`.

`audit` 가 폴더를 **집합**으로 회계한다면, `lineage` 는 머리 캡슐에서 뿌리까지
**링크를 걸어** 연대기를 판정한다.

연대기의 정의: **부모 산출 해시 == 자식 입력 해시**.

## 1. 호출

```bash
rhwp lineage <머리캡슐.json> --json
rhwp lineage <머리캡슐.json> --deep --json
```

머리는 **최신**(자식) 쪽이다. 뿌리부터 내려가지 않는다. 거슬러 올라간다.

이 스킬 1부의 기본 플래그는 `--json` 과 `--deep` 이다.
`--keyring` / `--anchor-log` 는 opt-in 확장 축이다. 안 주면 `signerOk` /
`anchoredOk` 키가 **실리지 않는다**. 없다고 실패로 읽지 마라.

## 2. 링크 판정 3축

각 `links[]` 원소:

| 축 | 타입 | 물음 | null 인 때 |
|----|------|------|------------|
| `parentOk` | bool\|null | 지금 읽은 파일 SHA-256 == 자식이 기록한 `parent.sha256` | 머리(대조할 자식 기록 없음) |
| `lineageOk` | bool\|null | 이 캡슐 `outputSha256` == 자식 `inputSha256` | 머리 |
| `reproduced` | bool\|null | `--deep` 이면 이 링크를 임시 재실행해 3값이 맞는지 | `--deep` 없음 |

뿌리는 `parent: null` 에서 멈춘다. 깊이 1 뿌리만 보면 세 축이 전부 null 인 것이
정상이다 (`fixtures/envelopes/lineage_root.json`).

하나라도 false 면 그 캡슐을 `brokenAt` 에 적고 걷기를 멈춘다. exit 3.

## 3. 부모 경로 해석

`parent.capsule` 이 절대 경로가 아니면:

```
next = current.parent_dir() / parent.capsule
```

cwd 가 아니다. `--parent` 저장 규칙과 대칭이다
([capsule-chain.md](capsule-chain.md) §3).

순환 가드: 1000 링크를 넘으면 깨진 체인 (error: 순환 의심).

## 4. `--deep`

얕은 lineage 는 해시 등식만 본다. `--deep` 은 링크마다
`replay_execute_to_temp` 를 돌려

- 산출 해시 == `receipt.outputSha256`
- 입력 해시 == `receipt.inputSha256`
- step 수 == `receipt.steps`

를 한 번에 본다. 비용은 링크 수다.

얕은 판정이 이미 깨졌으면 deep 을 돌릴 이유가 없다.

## 5. 봉투

| 필드 | 뜻 |
|------|----|
| `head` | 호출한 머리 경로 (에코) |
| `depth` | 걸은 링크 수 |
| `valid` | false 면 exit 3 |
| `brokenAt` | 처음 깨진 캡슐 경로. 유효하면 `null` |
| `links` | 머리 → 뿌리 방향의 판정 배열 |

`valid` 는 오류가 아니라 데이터다. 깨진 연대기를 도구 고장으로 승격하지 마라.

## 6. 실패 갈래

| 관찰 | exit | 메모 |
|------|-----:|------|
| 머리 파일 없음 | **1** | IO. 링크가 비어 있을 때만 |
| 무인자 / 미지 옵션 | 2 | 사용법 |
| `kind != workCapsule` | 3 | `brokenAt` = 그 파일 |
| `parent` 필드 자체 없음 | 3 | 합법 뿌리(`null`)와 다르다 |
| `parent.sha256` 누락·비hex | 3 | fail-closed. 생략 금지 |
| 부모 파일 읽기 실패 (중간) | 3 | 머리는 1, 중간은 3 |
| `parentOk` false | 3 | 부모 바이트 변조 |
| `lineageOk` false | 3 | 입력이 부모 산출이 아님 |
| `--deep` 재현 실패 | 3 | `reproduced: false` |

## 7. 워크스루

- [15_lineage_root.md](../examples/15_lineage_root.md)
- [16_lineage_two_link.md](../examples/16_lineage_two_link.md)
- [17_lineage_deep.md](../examples/17_lineage_deep.md)
- [18_lineage_broken_at.md](../examples/18_lineage_broken_at.md)
