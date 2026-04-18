# 웹 폰트 목록

웹 배포에 사용하는 woff2 폰트 파일 목록이다.
저작권 보호가 필요한 폰트는 Git에 포함하지 않으며, 별도로 준비해야 한다.

## 저작권 폰트 (Git 미포함)

로컬에 직접 배치해야 한다. 웹 배포 시 대응되는 폰트 폴백으로 대체 가능하다.
현재 웹 렌더러는 네트워크 의존성을 피하기 위해 함초롬 aliases를 번들된 Noto 계열 대체 폰트로 매핑한다.

### 한컴 폰트

| 파일명 | 폰트명 | 폴백 대체 |
|--------|--------|----------|
| hamchob-r.woff2 | 함초롬바탕 | Pretendard 또는 시스템 세리프 |
| hamchod-r.woff2 | 함초롬돋움 | Pretendard 또는 시스템 산세리프 |
| h2hdrm.woff2 | HY헤드라인M | Pretendard-Bold |
| hygprm.woff2 | HY고딕 | Pretendard |
| hygtre.woff2 | HY그래픽 | Pretendard |
| hymjre.woff2 | HY명조 | Pretendard 또는 시스템 세리프 |

### Microsoft 폰트

| 파일명 | 폰트명 | 폴백 대체 |
|--------|--------|----------|
| ArialW05-Regular.woff2 | Arial | 시스템 산세리프 |
| Calibri.woff2 | Calibri | Pretendard |
| CourierNewW05-Regular.woff2 | Courier New | 시스템 모노스페이스 |
| TahomaW05-Regular.woff2 | Tahoma | Pretendard |
| TimesNewRomanW05-Regular.woff2 | Times New Roman | 시스템 세리프 |
| VerdanaW05-Regular.woff2 | Verdana | Pretendard |
| MalgunGothicW35-Regular.woff2 | 맑은 고딕 | Pretendard |
| WebdingsW95-Regular.woff2 | Webdings | — |
| WingdingsW95-3.woff2 | Wingdings 3 | — |

## 오픈 라이선스 폰트 (Git 포함)

### Serif (명조체 계열)

| 파일명 | 폰트명 | 라이선스 | 출처 | 대체 대상 |
|--------|--------|---------|------|----------|
| NotoSerifKR-Regular.woff2 | Noto Serif KR Regular | SIL OFL 1.1 | Google Fonts | 바탕, 한컴바탕, 함초롬바탕 |
| NotoSerifKR-Bold.woff2 | Noto Serif KR Bold | SIL OFL 1.1 | Google Fonts | 바탕 Bold |
| NanumMyeongjo-Regular.woff2 | 나눔명조 Regular | SIL OFL 1.1 | Google Fonts | HY명조, 휴먼명조 |
| NanumMyeongjo-Bold.woff2 | 나눔명조 Bold | SIL OFL 1.1 | Google Fonts | HY명조 Bold |
| NanumMyeongjo-ExtraBold.woff2 | 나눔명조 ExtraBold | SIL OFL 1.1 | Google Fonts | HY명조 ExtraBold |
| GowunBatang-Regular.woff2 | 고운바탕 Regular | SIL OFL 1.1 | Google Fonts | 궁서 대체 |
| GowunBatang-Bold.woff2 | 고운바탕 Bold | SIL OFL 1.1 | Google Fonts | 궁서 Bold |

### Sans-serif (고딕체 계열)

| 파일명 | 폰트명 | 라이선스 | 출처 | 대체 대상 |
|--------|--------|---------|------|----------|
| Pretendard-*.woff2 (9종) | Pretendard | SIL OFL 1.1 | GitHub | 맑은 고딕, 함초롬돋움 |
| NotoSansKR-Regular.woff2 | Noto Sans KR Regular | SIL OFL 1.1 | Google Fonts | 돋움, 굴림, 한컴돋움 |
| NotoSansKR-Bold.woff2 | Noto Sans KR Bold | SIL OFL 1.1 | Google Fonts | 돋움 Bold |
| NanumGothic-Regular.woff2 | 나눔고딕 Regular | SIL OFL 1.1 | Google Fonts | 나눔고딕 (동일) |
| NanumGothic-Bold.woff2 | 나눔고딕 Bold | SIL OFL 1.1 | Google Fonts | 나눔고딕 Bold |
| NanumGothic-ExtraBold.woff2 | 나눔고딕 ExtraBold | SIL OFL 1.1 | Google Fonts | 나눔고딕 ExtraBold |
| GowunDodum-Regular.woff2 | 고운돋움 Regular | SIL OFL 1.1 | Google Fonts | HY고딕 대체 |
| SpoqaHanSans-Regular.woff2 | 스포카 한 산스 | SIL OFL 1.1 | GitHub | 보조 Sans |

### Monospace (고정폭)

| 파일명 | 폰트명 | 라이선스 | 출처 | 대체 대상 |
|--------|--------|---------|------|----------|
| D2Coding-Regular.woff2 | D2 Coding Regular | SIL OFL 1.1 | GitHub (naver) | 굴림체, 바탕체 |
| D2Coding-Bold.woff2 | D2 Coding Bold | SIL OFL 1.1 | GitHub (naver) | 굴림체 Bold |
| NanumGothicCoding-Regular.woff2 | 나눔고딕코딩 Regular | SIL OFL 1.1 | Google Fonts | 보조 Monospace |
| NanumGothicCoding-Bold.woff2 | 나눔고딕코딩 Bold | SIL OFL 1.1 | Google Fonts | 보조 Monospace Bold |

### 특수/장식체

| 파일명 | 폰트명 | 라이선스 | 출처 |
|--------|--------|---------|------|
| Cafe24Ssurround-v2.0.woff2 | 카페24 써라운드 | 무료 배포 | Cafe24 |
| Cafe24Supermagic-Regular-v1.0.woff2 | 카페24 슈퍼매직 | 무료 배포 | Cafe24 |
| Happiness-Sans-*.woff2 (4종) | 행복고딕 | 무료 배포 | 행복나눔 |
