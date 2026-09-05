# 07 — 자식 파일 기준 상대 경로

단: 캡슐. 픽스처: [../fixtures/lineage-layouts/relative-subdir](../fixtures/lineage-layouts/relative-subdir).

부모가 `root/a.capsule.json`, 자식이 `child/b.capsule.json` 이면 저장 값은
`../root/a.capsule.json` 이다.

```bash
rhwp replay --plan-json '<계획B>' --capsule child/b.capsule.json --parent root/a.capsule.json --json
```

`lineage` 는 현재 캡슐의 부모 디렉터리에 상대 경로를 붙인다. cwd 에서 해석하면
깨진 체인으로 오진한다.

체크리스트:

- [ ] `parent.capsule` 에 `..` 또는 같은 폴더 이름만 있다
- [ ] 절대 경로가 들어갔다면 다른 볼륨·다른 머신에서 깨진다
