# 예제 — export-png 재렌더

이슈 #5312. 실 에이전트 경로. gym 아님.

## 명령

```bash
rhwp export-png filled.hwp -p 0 -o filled_p0.png
```

## 실측·카탈로그 출력

```
페이지 수: A=1 B=1
최대 변위: 0.00 px (page -)
임계 초과 페이지: 0 / 구조 불일치 페이지: 0 (임계 1.00px)
status: PASS
```

## 읽는 법

눈 검증 기준. native-skia. thumbnail 을 대체하지 않는다.

관련: `references/` 같은 번호 장, `fixtures/transcripts/self_form01.txt`.
