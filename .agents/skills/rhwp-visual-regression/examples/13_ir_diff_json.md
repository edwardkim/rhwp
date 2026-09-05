# 예제 — ir-diff --json 차이

이슈 #5312. 실 에이전트 경로. gym 아님.

## 명령

```bash
rhwp ir-diff samples/hwp3-sample.hwp samples/SO-SUEOP.hwp --json
```

## 실측·카탈로그 출력

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## 읽는 법

F08. exit 3, identical false. 텍스트 모드면 같은 차이가 0.

관련: `references/` 같은 번호 장, `fixtures/transcripts/self_form01.txt`.
