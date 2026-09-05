# 08 — 캡슐 불변

단: 캡슐. 목표: 에디터·포맷터가 파일 바이트를 바꾸면 자식의 `parent.sha256` 이 깨진다.

의도된 동작이다. 변조 검출.

```bash
# 하지 말 것
# a.capsule.json 을 열어 들여쓰기만 바꿔 저장
rhwp lineage b.capsule.json --json
# → valid:false, parentOk:false, brokenAt, exit 3
```

고치려면 **재발급**한다. 필드를 손으로 고친 캡슐은 더 이상 그 체인의 부모가 아니다.

픽스처: `fixtures/capsules/tamper_pretty_print.capsule.json`.
