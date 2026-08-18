# 예제 — 없는 파일

이슈 #5312. 실 에이전트 경로. gym 아님.

## 명령

```bash
rhwp render-diff samples/no-such.hwp
```

## 실측·카탈로그 출력

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## 읽는 법

F07. 단건 exit 1. 배치 폴더 오류는 exit 2.

관련: `references/` 같은 번호 장, `fixtures/transcripts/self_form01.txt`.
