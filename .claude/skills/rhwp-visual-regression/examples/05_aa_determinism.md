# 예제 — A==A 결정성

이슈 #5312. 실 에이전트 경로. gym 아님.

## 명령

```bash
rhwp render-diff batch_out/0001.hwp batch_out/0001.hwp
```

## 실측·카탈로그 출력

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## 읽는 법

F02 의 기준선. PASS 가 아니면 도구 문제.

관련: `references/` 같은 번호 장, `fixtures/transcripts/aa_determinism.txt`.
