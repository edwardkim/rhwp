# 예제 — 자기 라운드트립 --via hwp

이슈 #5312. 실 에이전트 경로. gym 아님.

## 명령

```bash
rhwp render-diff samples/form-01.hwp --via hwp
```

## 실측·카탈로그 출력

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## 읽는 법

어댑터 경로. 출력 형식은 같고 via 만 hwp.

관련: `references/` 같은 번호 장, `fixtures/transcripts/self_form01.txt`.
