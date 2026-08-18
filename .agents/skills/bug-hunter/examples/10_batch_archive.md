# 예제 — 대량 아카이브 대장화

이슈 #5324. playbook 카탈로그 batch 축. gym 아님.

## 정답지

폴더 N 건 = 성공+실패 행 수. 봉투에 제목이 있어야 대장이 된다.
격차 #3407 (봉투에 제목 필드 부재) — F14 로 생존 확인.

## 명령

```bash
rhwp batch info --json < list.txt > meta.ndjson
rhwp info --json 한건.hwp
```

일괄 자체는 `rhwp-bulk-pipeline`. 실패한 한 건이나 제목 필드
부재를 여기 여정으로 승격한다. gym pack 을 만들지 않는다.

## 읽는 법

요약 줄만 보지 않는다. 행별 실패를 격리하고, 한 행을 J01–J07
중 맞는 축으로 다시 돈다.
