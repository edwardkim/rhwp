# 08. goal=export-hwpx

```
rhwp export-hwpx <doc> out/<stem>.hwpx --verify
```

자기검증(`--verify`)이 게이트다. exit 0 그리고 파일 실존.

## 왜 --verify 가 필수인가

HWPX 변환은 "파일이 생겼다"만으로는 부족하다. `--verify` 는 저장 직후
재파싱 대조다. 실패하면 성공처럼 보이는 미완성 산출물을 회신하지 않는다
(에이전트 툴킷 계약, C12).

## 산출

- `out/<stem>.hwpx`
- `summary`: `HWPX 변환 + verify 통과`

## 입력

HWP5·HWP3 가 전형적인 손님. 이미 HWPX 인 문서를 다시 내보내도 루프는
거절하지 않는다 — 그것은 CLI 의 일. 루프는 표의 게이트만 본다.
