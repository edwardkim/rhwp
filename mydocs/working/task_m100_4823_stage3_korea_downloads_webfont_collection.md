# #4823 Stage 3: korea_downloads 웹폰트 재수집 증적

## 실행

```sh
node scripts/survey_korea_downloads_font_jsdelivr.mjs \
  --input /Users/tsjang/Downloads/korea_downloads \
  --rhwp /Users/tsjang/rhwp/target/release/rhwp \
  --report mydocs/report/survey_korea_downloads_font_jsdelivr_20260815.md \
  --details mydocs/report/assets/survey_korea_downloads_font_jsdelivr_20260815.tsv
```

## 결과

- 입력 HWP/HWPX: 10,000개
- 파싱 성공: 9,948개
- 개별 파싱 실패: 52개. `batch info`의 NDJSON 오류를 보존하고 나머지 입력을 계속 집계했다.
- 고유 선언 글꼴: 1,379개
- 글꼴 파일 다운로드 가능: 324개
- 웹폰트 사용 가능: 70개
- CDN 응답은 확인했지만 라이선스 검토 필요: 254개

## KoPub 결과

- `KoPub돋움체`·`KoPub바탕체`의 Light/Medium/Bold 6개와 영문 축약형 2개는 모두 `font-kopub@1.0.2`의 요청 굵기와 일치하는 WOFF로 `available`, `웹폰트 사용 가능`이 됐다.
- `KoPubWorld` 돋움·바탕은 `font-kopubworld@1.0.3` 파일을 찾았지만 패키지 라이선스 메타데이터가 없어 `웹폰트 사용 라이선스 검토 필요`로 분리됐다.
- `_Pro` 계열은 검증 가능한 동일 face를 찾지 못해 `not-found`로 남았다.

## 산출물

- `mydocs/report/survey_korea_downloads_font_jsdelivr_20260815.md`
- `mydocs/report/assets/survey_korea_downloads_font_jsdelivr_20260815.tsv`
- `mydocs/report/assets/survey_korea_downloads_font_jsdelivr_20260815.run.log`
