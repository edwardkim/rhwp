# 종료 코드 — 판정은 크래시가 아니다

세션 핸드오프는 두 도구의 코드를 읽는다. 합치지 않는다. 새 코드를 만들지 않는다.

## orchestrator.py

DATP/1.0 대역의 상위 1자리. `code // 1000`.

| exit | code | 언제 | stdout |
|---:|---:|---|---|
| 0 | 0 | 수용 (`outcome=accepted`) | 최종 봉투 |
| 1 | 1000 | timeout / spawn / unparseable | 최종 봉투 (`status=error`) |
| 2 | — | task 스키마, 빠진 `--task`/`--agent`, 정책 파싱, 저널 파일 없음 | 보통 없음 (stderr) |
| 3 | 3000 | 판정·인계 (`schemaViolation`, `incompleteResult`, `agentVerdict`, 저널 체인 깨짐) | 봉투 (`status=verdict`) |
| 4 | 4000 | 정책·boundary (`securityViolation`, `policyRejected`) | 봉투 (`status=verdict`) |

exit 3 과 exit 4 는 도구 크래시가 아니다. `nextAction` 과 `findings[]` 를 읽는다.
사용법(2)과 IO 성 조기 return 은 실패 경로 stdout 0바이트에 가깝다.

`--verify-journal` 은 체인 깨짐이면 exit 3, 파일 없으면 exit 2.
오케스트레이터 런타임 실패는 exit 1 이다.

## rhwp replay / audit / lineage (포인터)

work-receipt 정본. 여기서 재정의하지 않는다.

| exit | 뜻 |
|---:|---|
| 0 | 성공 (verify 면 `reproduced:true`) |
| 1 | IO (머리 캡슐 없음 등) |
| 2 | 사용법 (빈 audit 폴더, 잘못된 hex) |
| 3 | 판정 (`reproduced:false`, 깨진 계보, 실패한 audit) |

실패 경로 stdout 은 0바이트인 명령이 있다. work-receipt 를 따른다.

## 세션 예외 표본이 쓰는 코드

| 예외 | 표본 exit | 이유 |
|---|---:|---|
| `missing_capsule` | 1 | IO |
| `parent_hash_mismatch` | 3 | 판정 |
| `dirty_named_worktree` | 2 | 사용법 (잘못된 자리) |
| `disk_full` | 1 | IO |
| `git add -A` | 2 | 사용법 (금지 호출) |
| DocumentCore 발명 | 2 | 사용법 (발명 호출) |

픽스처 `_skillMeta.exit` 는 0/1/2/3/4 만. 다른 숫자를 발명하지 않는다.
