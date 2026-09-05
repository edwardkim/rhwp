# 22 — 대량 batch

갈래: **대량**. 장: `80_대량과_상주.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`폴더 전체를 info.`

## 명령

```bash
printf '%s\n' docs/*.hwp | rhwp batch info --json
```

계약만 장. 실행 규약은 rhwp-bulk-pipeline. 실패 행 격리, N=성공+실패.
