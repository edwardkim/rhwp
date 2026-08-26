# 예제 — 시행문·공고문·회의록 법정 서식

이슈 #5324. playbook 카탈로그. gym 아님.

## 정답지

해당 편람/규정 별지. 별지를 확보하기 전에 생성 여정을 시작하지
않는다 (F02). 기안문(J03)과 같은 축의 다른 서식이다.

## 명령

```bash
rhwp export-svg 별지렌더.hwp -o truth/
rhwp fields --json 서식.hwp
rhwp edit fill-fields 서식.hwp --data @row.json -o 산출.hwp --json
rhwp export-svg 산출.hwp -o out/
```

가상 데이터만. 실제 시행/공고 시스템에 올리지 않는다 (F13).

## 읽는 법

결재란 표·항목체계·두문/결문/출석 표가 표현되는지. ingest 스키마
부재면 #3372 계열로 묶되, 서식별 실측 표를 붙인다 (F15).
