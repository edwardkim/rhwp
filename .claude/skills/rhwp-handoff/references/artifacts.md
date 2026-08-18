# 인계 묶음 산출물

outgoing 이 닫는 디스크 레이아웃이다. incoming 은 이 폴더 밖을 추측하지 않는다.

## 권장 루트

```
output/handoff/<taskId>/
```

`taskId` 는 HandoffTask 의 그 필드와 같다. 세션 번호나 날짜를 섞고 싶으면
`taskId` 자체에 넣는다 (`t-ord-20260818-s03`). 폴더 이름을 따로 발명하지 않는다.

## 필수 파일

| 경로 | 누가 쓰나 | incoming 이 읽는 것 |
|---|---|---|
| `task.json` | outgoing | 위임 명세. 재실행·대조 |
| `result.json` | outgoing (orchestrator stdout 리다이렉트) | `status`·`outcome`·`nextAction`·`collectedOutputs` |
| `handoff.journal.ndjson` | orchestrator `--journal` 기본값 | `--verify-journal` 로 체인만 확인 |
| `session.capsule.json` | `rhwp replay --capsule` | `receipt` 3해시, `parent` |
| `mydocs/working/<작업>.md` | outgoing (저장소 working) | 남은 목표, 세 파일 경로, 금지 |

있으면 좋은 파일:

| 경로 | 용도 |
|---|---|
| `parent.capsule.json` | 같은 폴더의 `--parent` 대상. 상대 경로가 가장 단순 |
| `collected/<taskId>/…` | 오케스트레이터가 수용한 산출만 복사 |
| `sandbox_<label>_aN/` | 시도별 sandbox. incoming 은 읽지 않는다. 수거물은 `collected/` |

## result.json 위치 규칙

오케스트레이터는 `result.json` 을 만들지 않는다. outgoing 이 `--json` stdout 을
저장한다. 경로는 다음 중 하나고, working doc 이 **어느 쪽인지** 적는다.

- `output/handoff/<taskId>/result.json` (권장)
- 호스트가 준 절대 경로 (시트 리필 시 후임 working doc 에 그대로)

두 곳에 다른 내용이 있으면 **더 최근 저널 seq 와 맞는 쪽**만 인정한다.
둘 다 저널과 안 맞으면 예외(추측 금지).

## 캡슐 위치 규칙

`--parent` 의 상대 경로는 **캡슐 파일 기준**이다 (호출 cwd 아님). 세션
핸드오프는 부모와 자식을 같은 폴더에 둔다.

```
output/handoff/t-session-03/parent.capsule.json
output/handoff/t-session-03/session.capsule.json   # --parent parent.capsule.json
```

다른 폴더에 두면 `../t-session-02/session.capsule.json` 처럼 캡슐 파일
기준으로 적는다. 절대경로는 금지에 가깝다 — 시트 리필 때 깨진다.

상세: [`capsule-parent-chain.md`](capsule-parent-chain.md).

## incoming 이 열지 않는 것

- `sandbox_*` 안의 입력 사본 (원본이 아니다)
- 에이전트 stderr
- 호스트 대화 로그
- 이름 붙은 워킹트리의 dirty 파일
- `gym/` 산출

## 최소 묶음이 불완전할 때

| 빠진 것 | 갈래 |
|---|---|
| `session.capsule.json` 없음 | [`exception-missing-capsule.md`](exception-missing-capsule.md) |
| 캡슐은 있는데 `parent.sha256` 이 실파일과 다름 | [`exception-parent-hash.md`](exception-parent-hash.md) |
| 산출을 쓰다 ENOSPC / disk full | [`exception-disk-full.md`](exception-disk-full.md) |
| 이름 붙은 트리만 있고 isolation 이 아님 | [`exception-dirty-worktree.md`](exception-dirty-worktree.md) |

부분 묶음을 완전한 것처럼 이어 받지 않는다.

## 픽스처

- `fixtures/layouts/complete-bundle/` — 세 파일 + 저널 + 캡슐
- `fixtures/layouts/missing-capsule/` — result + working, 캡슐 없음
- `fixtures/layouts/parent-mismatch/` — 자식 parent.sha256 위조
- `fixtures/incoming/read-order.json` — incoming 이 읽는 순서
