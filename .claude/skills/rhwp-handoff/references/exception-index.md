# 예외 네 갈래

세션 핸드오프의 예외는 네 개다. 더 만들지 않는다. 오케스트레이터 자체의
category (`timeout`, `schemaViolation`, …) 는 도구 문서의 것이고, incoming 은
그걸 `result.json` 으로 읽는다.

| id | 신호 | 첫 동작 | 정본 |
|---|---|---|---|
| `missing_capsule` | 머리가 가리키는 `*.capsule.json` 없음 | 추측 재개 금지 | [exception-missing-capsule.md](exception-missing-capsule.md) |
| `parent_hash_mismatch` | `parent.sha256` ≠ 실파일 | 후속 `--parent` 금지 | [exception-parent-hash.md](exception-parent-hash.md) |
| `dirty_named_worktree` | 금지 목록 트리가 dirty | 그 트리 checkout/reset 금지 | [exception-dirty-worktree.md](exception-dirty-worktree.md) |
| `disk_full` | ENOSPC / 쓰기 실패 | 추가 산출 금지, 미완 표시 | [exception-disk-full.md](exception-disk-full.md) |

공통:

- 판정은 데이터. 예외를 삼켜 성공으로 바꾸지 않는다
- 빈 캡슐·위조 해시를 만들어 예외를 지우지 않는다
- `gym/` 으로 우회하지 않는다
- 새 CLI 로 예외를 조회하지 않는다

픽스처 루트: `fixtures/exceptions/`.
각 JSON 은 `id`, `halt`, `next`, `_skillMeta.exit` 를 가진다.
