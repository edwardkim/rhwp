# 04 — 계획 파일과 `--plan-json`

단: 영수증. 목표: 두 입력 경로가 같은 명령을 쓰는지 고정한다.

```bash
rhwp replay plan.json --json
rhwp replay --plan-json "$(cat plan.json)" --json
```

`planSha256` 은 **원문 바이트**다. `cat` 이 개행을 바꾸거나 에디터가
pretty-print 하면 해시가 갈린다. 제3자 검증은 **같은 바이트**를 써야 한다.

함정: 위치 인자는 계획이지 캡슐이 아니다.

```bash
# 하지 말 것 — 캡슐을 계획으로 넣음
rhwp replay a.capsule.json --json
```

캡슐 검증은 `lineage` / `audit` 이다.
