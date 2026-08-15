# korea_downloads HWP/HWPX 글꼴과 jsDelivr 전수 조사

- **생성 시각**: 2026-08-15T07:06:07.921Z
- **기준 커밋**: `9eda49613bee` (local `devel`)
- **입력**: `/Users/tsjang/Downloads/korea_downloads`의 HWP/HWPX 10,000건
- **파서**: `/Users/tsjang/rhwp/target/release/rhwp`의 `batch info --json --threads 8`
- **글꼴 범위**: HWP/HWPX DOCINFO의 한글·영어·한자·일어·기타·기호·사용자 7개 글꼴군 전체. 문서 내부 중복은 문서별 1회만 센다.
- **jsDelivr 판정**: Fontsource 카탈로그 2,096건, `font-loader.ts`에 등록된 jsDelivr GitHub 글꼴, npm 전문 검색은 레지스트리 요청 제한으로 생략했고, Fontsource 카탈로그와 기존 등록 GitHub 배포본만 실제 CDN 파일까지 확인했다.

## 결과

| 지표 | 건수 |
| --- | ---: |
| 입력 문서 | 10,000 |
| 파싱 성공 | 9,948 |
| 파싱 실패 | 52 |
| 고유 선언 글꼴 | 1,414 |
| jsDelivr에서 다운로드 확인 | 19 |
| 검증 가능한 배포본 미발견 | 1,395 |
| 조회 오류 | 0 |

`미발견`은 인터넷의 임의 GitHub 저장소까지 부정하는 판정이 아니다. 공개 Fontsource 카탈로그와 기존 등록 GitHub 배포본을 이 스크립트의 동일 알고리즘으로 확인했을 때, **글꼴 바이트 파일을 실제로 내려받을 수 있는 jsDelivr URL을 검증하지 못했다**는 뜻이다. 패키지가 존재해도 원 글꼴과 동일한 서체인지, 라이선스가 해당 사용 목적을 허용하는지는 각 배포본의 라이선스를 별도로 확인해야 한다.

## 파싱 실패

| 분류 | 문서 수 |
| --- | ---: |
| 빈 파일 | 24 |
| 미지원 형식 | 15 |
| DRM 보호 | 8 |
| 암호 문서 | 5 |

## jsDelivr 다운로드 확인 글꼴

| 글꼴 | 사용 문서 | 배포 경로 | 패키지 | 파일 |
| --- | ---: | --- | --- | --- |
| 한컴바탕 | 5248 | jsDelivr GitHub | `projectnoonnu/noonfonts_2104` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff) |
| 함초롬바탕 | 4370 | jsDelivr GitHub | `projectnoonnu/noonfonts_2104` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff) |
| 함초롬돋움 | 3336 | jsDelivr GitHub | `projectnoonnu/noonfonts_four` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff) |
| 한컴돋움 | 495 | jsDelivr GitHub | `projectnoonnu/noonfonts_four` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff) |
| 나눔고딕 Bold | 251 |  | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| 나눔고딕 | 205 |  | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| 나눔명조 | 116 |  | `@fontsource/nanum-myeongjo` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-myeongjo@5.3.0/files/nanum-myeongjo-0-400-normal.woff) |
| 나눔고딕 ExtraBold | 91 |  | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| 나눔고딕 Light | 65 |  | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| 나눔명조 ExtraBold | 19 |  | `@fontsource/nanum-myeongjo` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-myeongjo@5.3.0/files/nanum-myeongjo-0-400-normal.woff) |
| 나눔고딕_코딩 | 18 |  | `@fontsource/nanum-gothic-coding` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic-coding@5.3.0/files/nanum-gothic-coding-0-400-normal.woff) |
| 한컴산뜻돋움 | 16 | jsDelivr GitHub | `projectnoonnu/noonfonts_four` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff) |
| NanumGothic | 2 |  | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| 새바탕 | 1 | jsDelivr GitHub | `projectnoonnu/noonfonts_2104` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff) |
| DejaVu Serif | 1 |  | `@fontsource/dejavu-serif` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/dejavu-serif@5.3.0/files/dejavu-serif-latin-400-italic.woff) |
| Noto Sans KR Medium | 1 |  | `@fontsource/noto-sans-kr` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/noto-sans-kr@5.3.0/files/noto-sans-kr-0-100-normal.woff) |
| Pretendard | 1 |  | `@fontsource/pretendard` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/pretendard@5.3.0/files/pretendard-latin-100-normal.woff) |
| Pretendard Light | 1 |  | `@fontsource/pretendard` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/pretendard@5.3.0/files/pretendard-latin-100-normal.woff) |
| Roboto | 1 |  | `@fontsource/roboto` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/roboto@5.3.0/files/roboto-cyrillic-100-italic.woff) |

## 사용 빈도 상위 30개

| 글꼴 | 사용 문서 | jsDelivr 판정 |
| --- | ---: | --- |
| 한양신명조 | 7549 | not-found |
| 명조 | 7533 | not-found |
| 휴먼명조 | 6368 | not-found |
| 바탕 | 5921 | not-found |
| HCI Poppy | 5732 | not-found |
| 굴림 | 5609 | not-found |
| 한컴바탕 | 5248 | available |
| 한양중고딕 | 5169 | not-found |
| 함초롬바탕 | 4370 | available |
| 맑은 고딕 | 3708 | not-found |
| 돋움 | 3374 | not-found |
| 함초롬돋움 | 3336 | available |
| HY헤드라인M | 3328 | not-found |
| 산세리프 | 3291 | not-found |
| HY중고딕 | 2803 | not-found |
| 바탕체 | 2590 | not-found |
| 돋움체 | 2589 | not-found |
| 굴림체 | 2506 | not-found |
| HY견고딕 | 2218 | not-found |
| HY신명조 | 2055 | not-found |
| HY견명조 | 1852 | not-found |
| 한양견고딕 | 1823 | not-found |
| 신명 견명조 | 1711 | not-found |
| 신명 태명조 | 1515 | not-found |
| #세명조 | 1459 | not-found |
| 한양견명조 | 1437 | not-found |
| -윤고딕130 | 1414 | not-found |
| HCI Tulip | 1410 | not-found |
| #신명조 | 1346 | not-found |
| 휴먼고딕 | 1307 | not-found |

## 전수 목록과 재현

전체 1,414개 글꼴의 사용 문서 수, 패키지·버전·라이선스 표기, 검증 URL, 판정 사유는 [TSV 상세 목록](assets/survey_korea_downloads_font_jsdelivr_20260815.tsv)에 기록했다.

`node scripts/survey_korea_downloads_font_jsdelivr.mjs --input <HWP|HWPX|디렉터리>`를 `devel`에서 실행하면 원시 임시 파일 없이 위 Markdown·TSV를 직접 다시 만든다. 실행 전에는 최신 바이너리를 만들기 위해 `cargo build --release`가 필요하다.
