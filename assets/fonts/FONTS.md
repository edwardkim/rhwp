# 웹 폰트 목록

웹 배포에 사용하는 WOFF2 폰트의 canonical source 목록이다. Git에는 36개 WOFF2,
총 22,651,296 bytes가 포함된다. Studio와 legacy `/web`은 각각의 `fonts/` runtime URL을 유지하면서
이 디렉토리를 참조하고, extension build는 여기에서 배포 산출물로 복사한다.

저작권 보호가 필요한 폰트는 Git에 포함하지 않으며, 별도로 준비해야 한다.

## 저작권 폰트 (Git 미포함)

로컬에 직접 배치해야 한다. 웹 배포 시 대응되는 폰트 폴백으로 대체 가능하다.

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
| NotoSansKR-Regular.woff2 | Noto Sans KR Regular | SIL OFL 1.1 | Google Fonts | 한컴돋움(=함초롬돋움 별칭), CanvasKit 기본 typeface |
| NotoSansKR-Bold.woff2 | Noto Sans KR Bold | SIL OFL 1.1 | Google Fonts | 돋움 Bold |
| NotoSansKR-ExtraLight.woff2 | Noto Sans KR ExtraLight | SIL OFL 1.1 | Google Fonts (wght 200 인스턴스) | Haansoft Dotum·돋움·굴림 (Task #1224 — 한컴 돋움 획 두께 정합) |
| NanumGothic-Regular.woff2 | 나눔고딕 Regular | SIL OFL 1.1 | Google Fonts | 나눔고딕 (동일) |
| NanumGothic-Bold.woff2 | 나눔고딕 Bold | SIL OFL 1.1 | Google Fonts | 나눔고딕 Bold |
| NanumGothic-ExtraBold.woff2 | 나눔고딕 ExtraBold | SIL OFL 1.1 | Google Fonts | 나눔고딕 ExtraBold |
| GowunDodum-Regular.woff2 | 고운돋움 Regular | SIL OFL 1.1 | Google Fonts | HY고딕 대체 |
| SpoqaHanSans-Regular.woff2 | 스포카 한 산스 | SIL OFL 1.1 | GitHub | 보조 Sans |

### Noto Sans KR Regular 서브셋 재생성

CanvasKit은 브라우저 시스템 폰트 폴백을 사용하지 않으므로 `NotoSansKR-Regular.woff2`에 필요한
글머리/도형 glyph가 실제로 포함돼야 한다. Regular asset은 기존 한글/라틴 cmap에 다음 범위를 추가한다.

- `U+2500-257F`: 표 테두리용 Box Drawing
- `U+25A0-25FF`: KS X 1001 글머리와 Geometric Shapes

입력은 Google Fonts의 `ofl/notosanskr/NotoSansKR[wght].ttf`이고, `wght=400` 정적 instance를
생성한다. 현재 재현 기준 source TTF는 `ttfs/opensource/NotoSansKR-Regular.ttf`에 커밋되어 있으므로
일반 검증이나 CI가 인터넷 다운로드에 의존하지 않는다. 아래 `curl`은 maintainer가 새 source에서 asset을
다시 생성할 때의 갱신 절차다. 재생성은 `fonttools[woff]` 환경에서 수행한다.

```bash
curl -L -o '/tmp/NotoSansKR[wght].ttf' \
  https://github.com/google/fonts/raw/main/ofl/notosanskr/NotoSansKR%5Bwght%5D.ttf
python tools/subset_noto_sans_kr_regular.py \
  --source '/tmp/NotoSansKR[wght].ttf'
```

출력은 `ttfs/opensource/NotoSansKR-Regular.ttf`와 `assets/fonts/NotoSansKR-Regular.woff2`다.
`npm run e2e:canvaskit-font-coverage`는 CanvasKit 실번들에서 `■`, `▪`, `□`, `○`, `─`의 glyph ID가
`0`이 아닌지 확인한다.

2026-07-11 생성 입력의 SHA-256은
`194018e6b2b293a7964f037b25c0249ce1418bc9ab3c971060a03aa57861e252`이다.

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
| Happiness-Sans-*.woff2 (3종) | 행복고딕 Regular/Bold/Title | 무료 배포 | 행복나눔 |
| HappinessSansVF.woff2 | 행복고딕 Variable | 무료 배포 | 행복나눔 |

### 문자/수식 보조

| 파일명 | 폰트명 | 라이선스 | 출처 | 용도 |
|--------|--------|---------|------|------|
| SourceHanSerifK-OldHangul-subset.woff2 | Source Han Serif K Old Hangul subset | SIL OFL 1.1 (`SourceHanSerifK-OFL.txt`) | Adobe / Google | 옛한글 자모·합자 fallback |
| LatinModernMath-Regular.woff2 | Latin Modern Math | GUST Font License | GUST / TeX ecosystem | 수식 fallback |
