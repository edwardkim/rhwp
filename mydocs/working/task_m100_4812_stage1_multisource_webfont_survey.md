---
kind: investigation
status: active
issue: 4812
canonical: mydocs/working/task_m100_4812_stage1_multisource_webfont_survey.md
last_verified: 2026-08-15
---

# Stage 1: 다중 공급자 웹폰트 전수 조사

## 목적

실제 HWP/HWPX 문서가 선언한 글꼴을 대상으로, 글꼴 파일을 내려받을 수 있는지와 웹폰트로 사용할 수 있는지를 별도로 조사한다.

## 분석 도구 경계

- `src/main.rs`의 `info --json`과 `batch info --json`은 DOCINFO의 한글, 영어, 한자, 일어, 기타, 기호, 사용자 글꼴군 전체를 문서 순서로 평탄화해 분석 입력을 제공한다.
- `scripts/survey_korea_downloads_font_jsdelivr.mjs`는 입력 HWP/HWPX 파일 또는 디렉터리를 받아 선언 글꼴을 집계하고 Fontsource, jsDelivr, Google Fonts, Noonnu, OnlineWebFonts를 순서대로 확인한다.
- 조사 스크립트는 선언명의 저장 형식 접두 기호, CSS 이스케이프, CP949 모지바케를 보수적으로 정규화한다. 원본 선언명과 외부 공급자 조회용 `search_name`은 TSV에서 분리한다.

## 판정 계약

- `download_available`은 실제 CDN 글꼴 파일 응답 여부를 기록한다.
- `webfont_usable`은 공급자의 라이선스 단서까지 확인된 경우만 `가능`으로 기록한다.
- OnlineWebFonts는 기술적 다운로드 응답만 확인하므로 원 권리자의 웹 사용 허가가 확인되기 전에는 `라이선스 검토 필요`로 기록한다.
- Google Fonts는 공식 CSS API의 family 일치와 `fonts.gstatic.com` 글꼴 응답을 모두 확인한다.

## 증적

- 요약 보고서: `mydocs/report/survey_korea_downloads_font_jsdelivr_20260815.md`
- 전수 TSV: `mydocs/report/assets/survey_korea_downloads_font_jsdelivr_20260815.tsv`
- 상세 실행 로그: `mydocs/report/assets/survey_korea_downloads_font_jsdelivr_20260815.run.log`

## 재현

```bash
node scripts/survey_korea_downloads_font_jsdelivr.mjs \
  --input <HWP|HWPX|디렉터리>
```

실행 전에는 현재 소스에 맞는 `target/release/rhwp`가 필요하다.
