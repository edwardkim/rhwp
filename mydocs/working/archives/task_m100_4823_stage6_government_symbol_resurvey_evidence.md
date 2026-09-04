---
title: "#4823 Stage 6 - 정부상징서체 재수집 증적"
date: 2026-08-15
issue: 4823
stage: 6
status: completed
---

# 정부상징서체 재수집 증적

## 실행

다음 명령으로 `korea_downloads` 입력 10,000개를 다시 조사했다.

```sh
node scripts/survey_korea_downloads_font_jsdelivr.mjs \
  --input /Users/tsjang/Downloads/korea_downloads \
  --rhwp /Users/tsjang/rhwp/target/release/rhwp \
  --report mydocs/report/survey_korea_downloads_font_jsdelivr_20260815.md \
  --details mydocs/report/assets/survey_korea_downloads_font_jsdelivr_20260815.tsv
```

## 결과

- 선언 글꼴: 1,379개
- 다운로드 가능: 328개
- 웹폰트 사용 가능: 80개
- 라이선스 검토 필요: 248개
- 개별 파싱 실패: 52개, NDJSON으로 보존 후 조사 계속

## 정부상징서체 판정

입력 문서에는 `Government_16040911` 및 `정부상징 부처명_16040911` 선언이 없어 TSV 행이
생성되지 않았다. 따라서 전수 결과에 없는 행을 성공으로 오인하지 않았다.

Stage 5의 실제 매핑 함수는 두 패밀리와 `Government_16040911.ttf` 입력에서 아래 고정 URL을
동일하게 반환함을 별도 확인했다.

```text
https://cdn.jsdelivr.net/gh/jangster77/korea-government-symbol-font@v1.0.0/fonts/Government_16040911.ttf
```

이 URL은 HTTP 200 및 `font/ttf`로 확인했으며, 공공누리 제4유형 조건을 함께 기록한다.
