# 예제 — 오라클 통과 ≠ 무손실

이슈 #5324. playbook 함정 1. gym 아님.

## 정답지

원본 ZIP 이름 집합. `--verify` 4/4 가 아니다.

## 명령

```bash
rhwp export-hwpx 서식.hwpx out.hwpx --verify --verify-pages
# 여기까지 통과해도 멈취지 않는다
# zip name set / header.xml size / tabItem count
```

## 읽는 법

#3551: tabItem 480→240, header.xml 6,737B 상수 감소.
#3557: 엔트리 수 12→12, ole1.ole 소실.
검출 94.6% 를 손실 94.6% 로 쓰지 않는다 (P16).
관련: `fixtures/envelopes/verify_pass_zip_loss.json`.
