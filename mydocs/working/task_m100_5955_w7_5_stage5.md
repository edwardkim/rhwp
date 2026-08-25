# Task M100 #5955 — Stage W7.5-5 future W8 mutation rehearsal 결과

- **이슈**: #5955
- **상위 tracker**: #4960
- **작업 브랜치**: `task_m100_5955`
- **기준 커밋**: `5a5f625d7`
- **단계**: Stage W7.5-5
- **상태**: 구현·검증 완료, 결과 승인 대기
- **작성일**: 2026-08-24 KST

## 1. 결론

실제 제품 mapping과 canonical registry를 바꾸지 않는 synthetic fixture에서 미래 W8 change set의 네
lifecycle 경로를 실행했다. 네 경로 모두 selection tuple 불변·신규 발급 규칙, active projection delta,
비대상 projection 무변화와 discard rollback 계약을 통과했다.

```text
evidence-only       active delta  0  target semantic delta 0  rollback restored
add-rule            active delta +1  target semantic delta 1  rollback restored
retire-rule         active delta -1  target semantic delta 1  rollback restored
retire-and-replace  active delta  0  target semantic delta 1  rollback restored
```

canonical 제품 registry의 raw SHA-256은 실행 전후
`fbab4413007a29600e5d667503e80b861ec4096827a8936943bdf74e58a5ae16`으로 같았다. rehearsal 결과는
stdout에서만 확인했으며 registry·projection artifact로 저장하거나 적용하지 않았다.

## 2. 구현

### 2.1 offline rehearsal API·CLI

`scripts/font_rule_mutation_rehearsal.mjs`를 추가했다.

- `rehearseMutationScenario()`은 caller-owned base를 clone한 reducer 결과만 검사한다.
- pre/post rule status, selection tuple hash, projection slot과 predecessor/successor를 대사한다.
- 다섯 projection의 active semantic row hash를 비교하고 선언된 한 plane 이외의 변경을 거부한다.
- 성공 결과도 canonical state에 commit하지 않고 ephemeral query model을 폐기한다.
- JSON round-trip으로 복원한 base의 registry digest와 semantic validation을 다시 확인한다.
- CLI는 인자를 받지 않고 네 공개 synthetic fixture의 canonical JSON envelope만 stdout에 출력한다.
- 제품 v2 registry의 raw bytes를 실행 전후 읽어 digest 변화가 있으면 실패한다.

`rehearseRejectedMutation()`은 잘못된 change set이 예외 없이 성공하거나 caller-owned base를 변경하면 즉시
실패한다. 오류 결과에는 synthetic error와 rollback digest만 남긴다.

### 2.2 executable contract

`scripts/tests/font_rule_mutation_rehearsal.test.mjs`를 추가해 다음을 고정했다.

1. evidence-only는 같은 ruleId와 tuple을 보존하고 evidence event만 갱신한다.
2. add는 이전에 없던 ruleId를 active tail slot에 추가한다.
3. retire는 tuple과 역사 row를 보존하고 active projection에서 제외한다.
4. retire-and-replace는 old/new tuple을 구분하고 predecessor/successor 양방향과 active slot 승계를 보존한다.
5. 네 경로 모두 비대상 네 projection과 caller base를 바꾸지 않으며 rollback digest가 pre digest와 같다.
6. 연속 실행과 CLI 재실행 결과가 결정론적으로 같다.

## 3. fail-closed negative rehearsal

Stage 5 focused test는 다음 9개 mutation을 거부하고 각 실패 뒤 입력 digest가 그대로인지 확인했다.

| mutation | 보호 경계 |
| --- | --- |
| in-place semantic update | tuple 변경에는 새 ruleId 강제 |
| stale parent digest | latest registry head에서만 적용 |
| cross-plane command | change set 하나는 한 decision plane만 소유 |
| evidence self-cycle | evidence graph는 DAG |
| checkout 밖 evidence path | private·host path 유입 차단 |
| replacement slot 변경 | predecessor의 active slot 승계 |
| 허위 active delta | 선언과 실제 projection 대사 |
| 기존 ruleId 재사용 | 전 lifecycle 전역 유일 ID |
| non-tail retirement | immutable·contiguous projection sequence 보존 |

기존 v2 계약 묶음도 함께 실행해 dangling evidence, 상한 초과, malformed nested value, retired projection 제외,
generator allowlist와 mid-commit rollback을 포함한 보호 경계를 재확인했다.

## 4. 계측 결과

synthetic base의 canonical registry digest는 모든 시나리오에서
`7faf193f59ac8e5caf8ab789266f39a0daf3e76a2ba13d72ea0ed266108cd831`이었다.

| lifecycle path | post registry SHA-256 | active delta | changed projection |
| --- | --- | ---: | --- |
| evidence-only | `4917ae7a75f7c8339f68af730aeaf19e5006aafa9b66567b505a496b346c2b15` | 0 | 없음 |
| add-rule | `2c84dba55085c29dad0c10481bf0df505ba347c4e5c9b8198a82246f8f969f36` | +1 | `canvas2d-paint` |
| retire-rule | `2413eaa18f42cbb671216eb2976f1a5b50338398348b040787901870b52cfe06` | -1 | `canvas2d-paint` |
| retire-and-replace | `243e4172ead602fc0e0466bb06d56f476f918d360097f56fbc3b96693acc6954` | 0 | `canvas2d-paint` |

evidence-only의 registry digest는 evidence graph와 lifecycle event 때문에 달라지지만 active projection의
semantic hash는 그대로다. retire-and-replace는 active 수가 같아도 selection tuple이 달라져 대상 projection
semantic hash가 달라진다. 이 구분으로 aggregate 수치가 상쇄 변경을 숨기지 못한다.

## 5. 검증

```text
node --check scripts/font_rule_mutation_rehearsal.mjs                      PASS
node --check scripts/tests/font_rule_mutation_rehearsal.test.mjs           PASS
node --test scripts/tests/font_rule_mutation_rehearsal.test.mjs             5/5 PASS
W7.5 registry+lifecycle+rehearsal+projection combined                      55/55 PASS
```

focused 5개 test 안에서 성공 lifecycle 4종, negative mutation 9종, deterministic CLI와 제품 registry
0-write를 검사했다. private corpus, Hyper-V Oracle, font bytes와 host path는 사용하지 않았다.

## 6. 보호 불변식 판정

- v1 봉인 artifact와 canonical v2 registry를 수정하지 않았다.
- 제품 mapping·generated source·renderer·trace envelope를 수정하지 않았다.
- same ruleId의 selection tuple은 evidence-only와 retirement에서 불변이다.
- semantic replacement는 새 ruleId와 양방향 lifecycle link를 가진다.
- active projection 변화는 선언된 한 plane에만 나타난다.
- 성공·실패 모두 caller-owned base와 canonical 제품 파일에 write하지 않는다.
- rehearsal 통과는 #4967 W8 mapping 변경의 승인이나 적용을 의미하지 않는다.

## 7. 다음 경계

결과 승인을 받으면 Stage W7.5-5 변경과 이 보고서를 한 경계 커밋으로 고정한다. 다음 Stage W7.5-6은
canonical 제품 0-delta와 local validation 사다리를 실행한다. remote push는 별도 승인 대상이다.
