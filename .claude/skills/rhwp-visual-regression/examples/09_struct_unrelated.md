# 예제 — 무관한 STRUCT

이슈 #5312. 실 에이전트 경로. gym 아님.

## 명령

```bash
rhwp render-diff 전.hwp 후.hwp
```

## 실측·카탈로그 출력

```
페이지 수: A=1 B=1
최대 변위: 495.93 px (page 0)
임계 초과 페이지: 1 / 구조 불일치 페이지: 1 (임계 1.00px)
  page   0: max= 495.93 mean= 13.40 nodes=39/37  [STRUCT]
       495.93px  Page/Body2/Column0/TextLine10/TextRun0
         0.00px  Page
         0.00px  Page/PageBg0
      Δ TextRun: 15→13 (-2)
status: STRUCT_MISMATCH
```

## 읽는 법

경로가 Header/Image0 이면 F04. export-png 로 그 쪽을 본다.

관련: `references/` 같은 번호 장, `fixtures/transcripts/pair_fill.txt`.
