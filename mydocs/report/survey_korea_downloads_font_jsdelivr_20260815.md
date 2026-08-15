# korea_downloads HWP/HWPX 글꼴과 웹폰트 전수 조사

- **생성 시각**: 2026-08-15T13:03:54.582Z
- **기준 커밋**: `7540dc8a5840` (local `devel`)
- **입력**: `/Users/tsjang/Downloads/korea_downloads`의 HWP/HWPX 10,000건
- **파서**: `/Users/tsjang/rhwp/target/release/rhwp`의 `batch info --json --threads 8`
- **글꼴 범위**: HWP/HWPX DOCINFO의 한글·영어·한자·일어·기타·기호·사용자 7개 글꼴군 전체. 문서 내부 중복은 문서별 1회만 센다.
- **웹폰트 판정**: Fontsource 카탈로그 2,096건, `font-loader.ts`에 등록된 jsDelivr GitHub 글꼴, jsDelivr 웹 검색 후보를 조사하고, 후보는 jsDelivr Data API의 파일 목록과 실제 CDN 글꼴 파일 응답까지 확인했다. Google Fonts는 공식 CSS API의 family 일치와 fonts.gstatic.com 응답을 확인했다. 동일 이름 Noonnu 후보는 상세 페이지의 웹사이트 사용 허가 요약과 CDN 응답을 함께 확인했다. OnlineWebFonts 후보는 CDN 응답만 확인하고 라이선스 검토 상태로 분리했다.

## 결과

| 지표 | 건수 |
| --- | ---: |
| 입력 문서 | 10,000 |
| 파싱 성공 | 9,948 |
| 파싱 실패 | 52 |
| 고유 선언 글꼴 | 1,379 |
| 글꼴 파일 다운로드 가능 | 328 |
| 웹폰트 사용 가능 | 80 |
| 웹폰트 사용 라이선스 검토 필요 | 248 |
| 웹폰트 공급 경로·CDN 응답 확인 | 95 |
| CDN 응답 확인·원 권리자 라이선스 검토 필요 | 233 |
| 검증 가능한 배포본 미발견 | 1,050 |
| 조회 오류 | 1 |

`미발견`은 인터넷의 임의 GitHub 저장소까지 부정하는 판정이 아니다. 공개 Fontsource 카탈로그와 jsDelivr 웹 검색, 기존 등록 GitHub 배포본을 이 스크립트의 동일 알고리즘으로 확인했을 때, **글꼴 바이트 파일을 실제로 내려받을 수 있는 웹폰트 URL을 검증하지 못했다**는 뜻이다. Noonnu의 `웹사이트 사용 가능` 표기는 Noonnu가 제공하는 요약 정보이므로 실제 배포 전 해당 글꼴의 최신 원 라이선스를 확인한다. OnlineWebFonts 응답 확인은 원 권리자의 웹 사용 허가를 뜻하지 않으며, `원 권리자 라이선스 검토 필요` 행은 서비스 배포에 사용하면 안 된다.

## 파싱 실패

| 분류 | 문서 수 |
| --- | ---: |
| 빈 파일 | 24 |
| 미지원 형식 | 15 |
| DRM 보호 | 8 |
| 암호 문서 | 5 |

## 웹폰트 공급 경로·CDN 응답 확인 글꼴

| 글꼴 | 사용 문서 | 다운로드 | 웹폰트 사용 | 배포 경로 | 패키지 | 파일 |
| --- | ---: | --- | --- | --- | --- | --- |
| 한컴바탕 | 5248 | 가능 | 가능 | jsDelivr GitHub | `projectnoonnu/noonfonts_2104` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff) |
| 함초롬바탕 | 4370 | 가능 | 가능 | jsDelivr GitHub | `projectnoonnu/noonfonts_2104` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff) |
| 함초롬돋움 | 3336 | 가능 | 가능 | jsDelivr GitHub | `projectnoonnu/noonfonts_four` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff) |
| 바탕체 | 2590 | 가능 | 가능 | jsDelivr 웹 검색 | `@noonnu/bareun-batang` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/bareun-batang@0.1.0/fonts/bareunbatang-400.woff) |
| 돋움체 | 2589 | 가능 | 가능 | jsDelivr 웹 검색 | `@noonnu/yi-sun-shin-dotum-m` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/yi-sun-shin-dotum-m@0.1.0/fonts/yisunshindotumm-normal.woff) |
| Times New Roman | 889 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `@canvas-fonts/times-new-roman` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/times-new-roman@1.0.4/Times New Roman.ttf) |
| KoPub바탕체 Light | 582 | 가능 | 가능 | jsDelivr npm | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubBatang-Light.woff) |
| KoPub돋움체 Light | 503 | 가능 | 가능 | jsDelivr npm | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubDotum-Light.woff) |
| 한컴돋움 | 495 | 가능 | 가능 | jsDelivr GitHub | `projectnoonnu/noonfonts_four` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff) |
| Arial | 464 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `@canvas-fonts/arial` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/arial@1.0.4/Arial.ttf) |
| 나눔고딕 Bold | 251 | 가능 | 가능 | Fontsource npm | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| KoPubWorld돋움체 Bold | 214 | 가능 | 가능 | jsDelivr npm | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Dotum-Bold.otf) |
| 나눔고딕 | 205 | 가능 | 가능 | Fontsource npm | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| KoPub돋움체 Bold | 201 | 가능 | 가능 | jsDelivr npm | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubDotum-Bold.woff) |
| KoPub돋움체 Medium | 143 | 가능 | 가능 | jsDelivr npm | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubDotum-Medium.woff) |
| Arial Narrow | 141 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `@canvas-fonts/arial-narrow-bold` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/arial-narrow-bold@1.0.4/Arial Narrow Bold.ttf) |
| 나눔명조 | 116 | 가능 | 가능 | Fontsource npm | `@fontsource/nanum-myeongjo` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-myeongjo@5.3.0/files/nanum-myeongjo-0-400-normal.woff) |
| 나눔고딕 ExtraBold | 91 | 가능 | 가능 | Fontsource npm | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| 나눔고딕 Light | 65 | 가능 | 가능 | Fontsource npm | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| Garamond | 38 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/cormorant-garamond` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/cormorant-garamond@5.3.0/files/cormorant-garamond-cyrillic-300-italic.woff) |
| Courier New | 37 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `@canvas-fonts/courier-new` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/courier-new@1.0.4/Courier New.ttf) |
| KoPub바탕체 Bold | 34 | 가능 | 가능 | jsDelivr npm | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubBatang-Bold.woff) |
| KoPub바탕체 Medium | 29 | 가능 | 가능 | jsDelivr npm | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubBatang-Medium.woff) |
| MS Mincho | 27 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/shippori-mincho` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/shippori-mincho@5.3.0/files/shippori-mincho-0-400-normal.woff) |
| Arial Black | 23 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `@canvas-fonts/arial-black` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/arial-black@1.0.4/Arial Black.ttf) |
| 경기천년바탕 Regular | 20 | 가능 | 가능 | Noonnu CDN | `noonnu:font_page/13` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/2410-3@1.0/Batang_Regular.woff) |
| 나눔명조 ExtraBold | 19 | 가능 | 가능 | Fontsource npm | `@fontsource/nanum-myeongjo` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-myeongjo@5.3.0/files/nanum-myeongjo-0-400-normal.woff) |
| 나눔고딕_코딩 | 18 | 가능 | 가능 | Fontsource npm | `@fontsource/nanum-gothic-coding` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic-coding@5.3.0/files/nanum-gothic-coding-0-400-normal.woff) |
| SimSun | 17 | 가능 | 가능 | jsDelivr 웹 검색 | `react-native-font-sim` | [파일](https://cdn.jsdelivr.net/npm/react-native-font-sim@2.0.1/fonts/SimSun.ttf) |
| 한컴산뜻돋움 | 16 | 가능 | 가능 | jsDelivr GitHub | `projectnoonnu/noonfonts_four` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff) |
| 경기천년제목 Medium | 13 | 가능 | 가능 | Noonnu CDN | `noonnu:font_page/14` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/2410-3@1.0/Title_Medium.woff) |
| Baskerville BT | 13 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/libre-baskerville` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/libre-baskerville@5.3.0/files/libre-baskerville-latin-400-italic.woff) |
| Comic Sans MS | 12 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `@canvas-fonts/comic-sans-ms` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/comic-sans-ms@1.0.4/Comic Sans MS.ttf) |
| KoPubWorld돋움체 Light | 12 | 가능 | 가능 | jsDelivr npm | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Dotum-Light.otf) |
| KoPubWorld바탕체 Light | 12 | 가능 | 가능 | jsDelivr npm | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Batang-Light.otf) |
| Bodoni Bd BT | 11 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/bodoni-moda` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/bodoni-moda@5.3.0/files/bodoni-moda-latin-400-italic.woff) |
| Bodoni Bk BT | 11 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/bodoni-moda` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/bodoni-moda@5.3.0/files/bodoni-moda-latin-400-italic.woff) |
| BrushScript BT | 10 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/nanum-brush-script` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-brush-script@5.3.0/files/nanum-brush-script-0-400-normal.woff) |
| KoPubWorld돋움체 Medium | 10 | 가능 | 가능 | jsDelivr npm | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Dotum-Medium.otf) |
| MS Gothic | 10 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/zen-maru-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/zen-maru-gothic@5.3.0/files/zen-maru-gothic-10-300-normal.woff) |
| 나눔명조OTF ExtraBold | 9 | 가능 | 가능 | jsDelivr 웹 검색 | `@kfonts/nanum-myeongjo-otf` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-myeongjo-otf@0.2.0/src/NanumMyeongjoExtraBold.otf) |
| Times New Roman Bold | 8 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `@canvas-fonts/times-new-roman-bold` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/times-new-roman-bold@1.0.4/Times New Roman Bold.ttf) |
| 다음_SemiBold | 7 | 가능 | 가능 | jsDelivr 웹 검색 | `alibabapuhuiti-3-75-semibold` | [파일](https://cdn.jsdelivr.net/npm/alibabapuhuiti-3-75-semibold@1.0.0/AlibabaPuHuiTi-3-75-SemiBold.otf) |
| MS UI Gothic | 6 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/zen-maru-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/zen-maru-gothic@5.3.0/files/zen-maru-gothic-10-300-normal.woff) |
| Calisto MT | 5 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/calistoga` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/calistoga@5.3.0/files/calistoga-latin-400-normal.woff) |
| 경기천년제목 Light | 4 | 가능 | 가능 | Noonnu CDN | `noonnu:font_page/14` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/2410-3@1.0/Title_Medium.woff) |
| Helvetica Neue | 4 | 가능 | 가능 | jsDelivr 웹 검색 | `@marcius-studio/font` | [파일](https://cdn.jsdelivr.net/npm/@marcius-studio/font@0.0.1/HelveticaNeueCyr/HelveticaNeueCyr-Black.ttf) |
| KoPubWorld바탕체 Medium | 4 | 가능 | 가능 | jsDelivr npm | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Batang-Medium.otf) |
| Myeongjo | 4 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/nanum-myeongjo` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-myeongjo@5.3.0/files/nanum-myeongjo-0-400-normal.woff) |
| 경기천년제목 Bold | 3 | 가능 | 가능 | Noonnu CDN | `noonnu:font_page/14` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/2410-3@1.0/Title_Medium.woff) |
| 나눔바른펜 | 3 | 가능 | 가능 | Noonnu CDN | `noonnu:font_page/42` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_two@1.0/NanumBarunpen.woff) |
| 에스코어 드림 3 Light | 3 | 가능 | 가능 | jsDelivr 웹 검색 | `@noonnu/s-core-dream-3-light` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/s-core-dream-3-light@0.1.0/fonts/s-coredream-3light-normal.woff) |
| Bodoni MT | 3 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/bodoni-moda` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/bodoni-moda@5.3.0/files/bodoni-moda-latin-400-italic.woff) |
| Century Schoolbook | 3 | 가능 | 가능 | jsDelivr 웹 검색 | `centschbook-mono` | [파일](https://cdn.jsdelivr.net/npm/centschbook-mono@3.2.1/Century-Schoolbook-Monospace-BT.ttf) |
| Cooper Black | 3 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `fonts-archive-cooper-black` | [파일](https://cdn.jsdelivr.net/npm/fonts-archive-cooper-black@0.0.0/CooperBlack Italic-Regular.otf) |
| Helvetica | 3 | 가능 | 가능 | jsDelivr 웹 검색 | `helvetica-original` | [파일](https://cdn.jsdelivr.net/npm/helvetica-original@1.0.0/Black/Helvetica-Black.ttf) |
| KoPubBatangLight | 3 | 가능 | 가능 | jsDelivr npm | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubBatang-Light.woff) |
| KoPubWorld바탕체 Bold | 3 | 가능 | 가능 | jsDelivr npm | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Batang-Bold.otf) |
| MT Extra | 3 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/fira-sans-extra-condensed` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/fira-sans-extra-condensed@5.3.0/files/fira-sans-extra-condensed-cyrillic-100-italic.woff) |
| Segoe UI | 3 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontpkg/segoe-ui` | [파일](https://cdn.jsdelivr.net/npm/@fontpkg/segoe-ui@5.67.0/segoeui.ttf) |
| 경기천년바탕 Bold | 2 | 가능 | 가능 | Noonnu CDN | `noonnu:font_page/13` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/2410-3@1.0/Batang_Regular.woff) |
| 나눔바른고딕 Light | 2 | 가능 | 가능 | jsDelivr 웹 검색 | `@kfonts/nanum-barun-gothic` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-barun-gothic@0.3.0/NanumBarunGothicLight.woff) |
| arial | 2 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `@canvas-fonts/arial` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/arial@1.0.4/Arial.ttf) |
| MS Song | 2 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/song-myung` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/song-myung@5.3.0/files/song-myung-10-400-normal.woff) |
| NanumGothic | 2 | 가능 | 가능 | Fontsource npm | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| Noto | 2 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/noto-sans-jp` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/noto-sans-jp@5.3.0/files/noto-sans-jp-0-100-normal.woff) |
| Vladimir Script | 2 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `fonts-archive-vladimir-script` | [파일](https://cdn.jsdelivr.net/npm/fonts-archive-vladimir-script@0.0.1/VladimirScript.ttf) |
| 62570체 | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@noonnu/62570che` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/62570che@0.1.0/fonts/62570-normal.woff) |
| 나눔고딕OTF | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@kfonts/nanum-gothic-otf` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-gothic-otf@0.2.0/src/NanumGothic.otf) |
| 나눔고딕OTF Bold | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@kfonts/nanum-gothic-otf` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-gothic-otf@0.2.0/src/NanumGothicBold.otf) |
| 나눔바른고딕OTF | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@kfonts/nanum-barun-gothic-yet-hangul-otf` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-barun-gothic-yet-hangul-otf@0.2.0/src/NanumBarunGothic-YetHangul.otf) |
| 나눔스퀘어라운드 Bold | 1 | 가능 | 가능 | Noonnu CDN | `noonnu:font_page/38` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_two@1.0/NanumSquareRound.woff) |
| 나눔스퀘어라운드 ExtraBold | 1 | 가능 | 가능 | Noonnu CDN | `noonnu:font_page/38` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_two@1.0/NanumSquareRound.woff) |
| 나눔스퀘어라운드 Regular | 1 | 가능 | 가능 | Noonnu CDN | `noonnu:font_page/38` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_two@1.0/NanumSquareRound.woff) |
| 나눔스퀘어OTF | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@kfonts/nanum-square-otf` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-square-otf@0.2.0/src/NanumSquareB.otf) |
| 새바탕 | 1 | 가능 | 가능 | jsDelivr GitHub | `projectnoonnu/noonfonts_2104` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff) |
| Apple SD 산돌고딕 Neo 일반체 | 1 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `font-applesdgothicneo` | [파일](https://cdn.jsdelivr.net/npm/font-applesdgothicneo@1.0.3/fonts/100_AppleSDGothicNeo-Thin.otf) |
| Arial (W1) | 1 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `@canvas-fonts/arial` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/arial@1.0.4/Arial.ttf) |
| DejaVu Serif | 1 | 가능 | 가능 | Fontsource npm | `@fontsource/dejavu-serif` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/dejavu-serif@5.3.0/files/dejavu-serif-latin-400-italic.woff) |
| FangSong | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontpkg/zhuque-fangsong-technical-preview` | [파일](https://cdn.jsdelivr.net/npm/@fontpkg/zhuque-fangsong-technical-preview@0.212.0/ZhuqueFangsong-Regular.ttf) |
| Futura Hv BT | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `futura-font` | [파일](https://cdn.jsdelivr.net/npm/futura-font@1.0.0/FuturaBT-Medium.ttf) |
| Futura Std ExtraBold | 1 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `fonts-archive-futura-std` | [파일](https://cdn.jsdelivr.net/npm/fonts-archive-futura-std@0.0.0/FuturaStd-ExtraBold.otf) |
| Futura Std Light | 1 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `fonts-archive-futura-std` | [파일](https://cdn.jsdelivr.net/npm/fonts-archive-futura-std@0.0.0/FuturaStd-Light.otf) |
| Futura Std Medium | 1 | 가능 | 라이선스 검토 필요 | jsDelivr 웹 검색 | `fonts-archive-futura-std` | [파일](https://cdn.jsdelivr.net/npm/fonts-archive-futura-std@0.0.0/FuturaStd-Medium.otf) |
| HCRDotum | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@noonnu/hcr-dotum` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/hcr-dotum@0.1.0/fonts/hcrdotum-normal.woff) |
| Helvetica 65 Medium | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@duppla-font/helvetica-now` | [파일](https://cdn.jsdelivr.net/npm/@duppla-font/helvetica-now@1.0.0/files/HelveticaNowTextMedium.otf) |
| KBIZ한마음명조 R | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@noonnu/kbiz-hanmaum-myungjo` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/kbiz-hanmaum-myungjo@0.1.0/fonts/kbizhanmaummyungjo-normal.woff) |
| KoPubDotumMedium | 1 | 가능 | 가능 | jsDelivr npm | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubDotum-Medium.woff) |
| Nanum Barun Gothic | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@kfonts/nanum-barun-gothic` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-barun-gothic@0.3.0/NanumBarunGothic.woff) |
| Noto Sans CJK JP Regular | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `noto-sans-cjk-jp` | [파일](https://cdn.jsdelivr.net/npm/noto-sans-cjk-jp@1.0.1/fonts/NotoSansCJKjp-Regular.woff) |
| Noto Sans KR Medium | 1 | 가능 | 가능 | Fontsource npm | `@fontsource/noto-sans-kr` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/noto-sans-kr@5.3.0/files/noto-sans-kr-0-100-normal.woff) |
| Pretendard | 1 | 가능 | 가능 | Fontsource npm | `@fontsource/pretendard` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/pretendard@5.3.0/files/pretendard-latin-100-normal.woff) |
| Pretendard Light | 1 | 가능 | 가능 | Fontsource npm | `@fontsource/pretendard` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/pretendard@5.3.0/files/pretendard-latin-100-normal.woff) |
| Roboto | 1 | 가능 | 가능 | Fontsource npm | `@fontsource/roboto` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/roboto@5.3.0/files/roboto-cyrillic-100-italic.woff) |
| Yu Mincho | 1 | 가능 | 가능 | jsDelivr 웹 검색 | `@fontsource/shippori-mincho` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/shippori-mincho@5.3.0/files/shippori-mincho-0-400-normal.woff) |

## CDN 응답 확인·원 권리자 라이선스 검토 필요

| 글꼴 | 사용 문서 | 다운로드 | 웹폰트 사용 | 공급 경로 | 파일 | 비고 |
| --- | ---: | --- | --- | --- | --- | --- |
| Calibri | 195 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/a78cfad3beb089a6ce86d4e280fa270b.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/a78cfad3beb089a6ce86d4e280fa270b); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| AmeriGarmnd BT | 143 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/5dbbb35318f7b9fd3db52618337e56a6.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/5dbbb35318f7b9fd3db52618337e56a6); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Tahoma | 60 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/cd0381aa3322dff4babd137f03829c8c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/cd0381aa3322dff4babd137f03829c8c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Arial Unicode MS | 27 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/25392c1fcb8a06c1f490a0e959a32b03.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/25392c1fcb8a06c1f490a0e959a32b03); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Book Antiqua | 25 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/127984ac535ca158ad9724f752ade6a6.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/127984ac535ca158ad9724f752ade6a6); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Cambria | 23 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/2db80501ab27169c9b8395ce6f749be1.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/2db80501ab27169c9b8395ce6f749be1); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Verdana | 19 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/562fa31bba08b3f71cb71257ddb880d5.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/562fa31bba08b3f71cb71257ddb880d5); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Hobo BT | 18 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/f5221403a96ef433e963d80a24259396.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/f5221403a96ef433e963d80a24259396); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Blippo Blk BT | 17 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/651028c4747c9241cc0c4e1e04e0690a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/651028c4747c9241cc0c4e1e04e0690a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Wingdings | 17 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e991cc888d4fb544fe0a88d065ab6efc.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e991cc888d4fb544fe0a88d065ab6efc); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Symbol | 16 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/39a0230c9a2f421123a02f97cd0d451e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/39a0230c9a2f421123a02f97cd0d451e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Century | 15 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/165b3175b1345c0eb8b4097f4d024455.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/165b3175b1345c0eb8b4097f4d024455); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Century Gothic | 14 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/0d78b12d6be09203d1fbeb76871a369a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/0d78b12d6be09203d1fbeb76871a369a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Trebuchet MS | 14 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/03e852a9d1635cf25800b41001ee80c7.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/03e852a9d1635cf25800b41001ee80c7); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Bookman Old Style | 13 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/08a4f684fb0599188430dd0b97af52ac.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/08a4f684fb0599188430dd0b97af52ac); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Georgia | 13 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/7dca09e227fdfe16908cebb4244589e4.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/7dca09e227fdfe16908cebb4244589e4); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| GoudyOlSt BT | 13 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/3ad52e45811ab0c75cc0ce85f2a81bac.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/3ad52e45811ab0c75cc0ce85f2a81bac); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| HyhwpEQ | 11 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/21ef608aa01347328562d13cc5a55169.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/21ef608aa01347328562d13cc5a55169); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Microsoft Sans Serif | 11 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/2442df04682466647c9b737e374dd1ef.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/2442df04682466647c9b737e374dd1ef); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| BernhardFashion BT | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/50e41820e2ada7948a61a51b6bf74b1a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/50e41820e2ada7948a61a51b6bf74b1a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| CentSchbook BT | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/23aa1bdf235d88702921ae7d08d52f95.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/23aa1bdf235d88702921ae7d08d52f95); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| CommercialScript BT | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/449b1c567da51480d590112112addcd6.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/449b1c567da51480d590112112addcd6); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Cooper Blk BT | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/5ef79458159fb07a0c8f9b9a9a99c666.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/5ef79458159fb07a0c8f9b9a9a99c666); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Courier10 BT | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/8c87ce7c5ad1cf181e1020e401990e39.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/8c87ce7c5ad1cf181e1020e401990e39); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| DomCasual BT | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/fd4e2b1939db4f17e901e779c80f1d4c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/fd4e2b1939db4f17e901e779c80f1d4c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Haettenschweiler | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/97d6f29a4bda3a872dad26cc5b2d0d7b.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/97d6f29a4bda3a872dad26cc5b2d0d7b); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Impact | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/771d2d395300ea1b80c34ba5282bf694.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/771d2d395300ea1b80c34ba5282bf694); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Lucida Console | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/97a55c954a18d2daae22c5f9114794d5.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/97a55c954a18d2daae22c5f9114794d5); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Lucida Sans Unicode | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/26c724b3b181aad246aa1321eaa9ea21.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/26c724b3b181aad246aa1321eaa9ea21); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Marlett | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/c8da7be807a6253e53142d51f6d3c37d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/c8da7be807a6253e53142d51f6d3c37d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| MurrayHill Bd BT | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/a1c46075a3198167739167533920832e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/a1c46075a3198167739167533920832e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Orator10 BT | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/1a373e4d652fde7a42eb3d18468a3820.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/1a373e4d652fde7a42eb3d18468a3820); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| PMingLiU | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/b8c332e7b686ba28daebf6524c66aeec.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/b8c332e7b686ba28daebf6524c66aeec); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Swis721 Lt BT | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/ff21985e04861bb8d9745440ee1e2e7d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/ff21985e04861bb8d9745440ee1e2e7d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Wingdings 2 | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/f0f870099e5e748e93a126ea16dcaeba.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/f0f870099e5e748e93a126ea16dcaeba); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Wingdings 3 | 10 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/6ffe077a0058332e71fa05151a519699.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/6ffe077a0058332e71fa05151a519699); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| 3 of 9 Barcode | 8 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/7a3abce9f43e0b74320df47238a0460c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/7a3abce9f43e0b74320df47238a0460c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Franklin Gothic Medium | 8 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9c9dbb999dd7068f51335d93cc7328bd.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9c9dbb999dd7068f51335d93cc7328bd); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| FuturaBlack BT | 8 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/58fd3ae6d13f5bd68f8b04c7e5d5d824.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/58fd3ae6d13f5bd68f8b04c7e5d5d824); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Gautami | 8 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e62d85a3dc9d8815cacfc13ff5dea781.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e62d85a3dc9d8815cacfc13ff5dea781); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Liberty BT | 8 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/85256468379fd013a62272dc0e9e3b1e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/85256468379fd013a62272dc0e9e3b1e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| MingLiU | 8 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/23cfc97e0a97d980f87f5780303baf01.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/23cfc97e0a97d980f87f5780303baf01); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| ParkAvenue BT | 8 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/af0086fe524ded40bc732eb91ad03ccf.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/af0086fe524ded40bc732eb91ad03ccf); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Stencil BT | 8 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/353f60b63a9452cbd838c62d8f358184.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/353f60b63a9452cbd838c62d8f358184); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Arial Rounded MT Bold | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/ef6bdf5ef216552c7e9869841e891ca0.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/ef6bdf5ef216552c7e9869841e891ca0); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| HYHeadLine M | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/b6bd6a4ea9787e452355467f5c8bde76.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/b6bd6a4ea9787e452355467f5c8bde76); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Latha | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/87d92990ec1481924e2bfee102e5c0eb.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/87d92990ec1481924e2bfee102e5c0eb); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Mangal | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/76620e2b3e9d54512d94915e11659eb5.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/76620e2b3e9d54512d94915e11659eb5); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| MS PGothic | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/b73897a75e8d28c20f9ab68f075c458f.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/b73897a75e8d28c20f9ab68f075c458f); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| MV Boli | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/19a8db654e36276ffccbb72c68b0305a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/19a8db654e36276ffccbb72c68b0305a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Raavi | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/8e06b2bd31ace3dbb6f97378872ca4e9.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/8e06b2bd31ace3dbb6f97378872ca4e9); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Shruti | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/740e77af6f9757e75c0be9b2664472b9.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/740e77af6f9757e75c0be9b2664472b9); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| SimHei | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/090ec3fbb2be3c7b36967f0bda8e0964.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/090ec3fbb2be3c7b36967f0bda8e0964); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Sylfaen | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/cf85131ef1119a8d56e92cd8ff533995.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/cf85131ef1119a8d56e92cd8ff533995); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Tunga | 6 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/acf560a35ee1cc896e2892c5d6653a5a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/acf560a35ee1cc896e2892c5d6653a5a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Copperplate Gothic Bold | 5 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/483f8a5e2868222491b8baed78121c3a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/483f8a5e2868222491b8baed78121c3a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Lucida Sans | 5 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/db90a54b1d16a8a33b8fc256d41e228a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/db90a54b1d16a8a33b8fc256d41e228a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Aptos | 4 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/7dd5f4bf5d38875ca1822a830b6e6fe4.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/7dd5f4bf5d38875ca1822a830b6e6fe4); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Berlin Sans FB | 4 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/fe2027c27b6a24505f548c6fd2e1076d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/fe2027c27b6a24505f548c6fd2e1076d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Bodoni MT Black | 4 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/037a670cefc97958beb036dab0f6e254.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/037a670cefc97958beb036dab0f6e254); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Bookshelf Symbol 7 | 4 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/549dddcecfd3e61f35f4fde66019618f.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/549dddcecfd3e61f35f4fde66019618f); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Candara | 4 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e662339992c4abf5b43f537391bd3169.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e662339992c4abf5b43f537391bd3169); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Copperplate Gothic Light | 4 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/483f8a5e2868222491b8baed78121c3a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/483f8a5e2868222491b8baed78121c3a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Gulim | 4 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/d9521b14999b76104e98b3d2f96079a1.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/d9521b14999b76104e98b3d2f96079a1); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Agency FB | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/7a9ddc1b445c1713f7ad1cf3de47edd7.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/7a9ddc1b445c1713f7ad1cf3de47edd7); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Algerian | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/c144af7d488f9069913d40dee3cd1f70.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/c144af7d488f9069913d40dee3cd1f70); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Baskerville Old Face | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/f3077c21790d7835da194a845217f8ce.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/f3077c21790d7835da194a845217f8ce); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Bell MT | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/01661f50354ecd0a3560ac450ecf43d3.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/01661f50354ecd0a3560ac450ecf43d3); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Berlin Sans FB Demi | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/fe2027c27b6a24505f548c6fd2e1076d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/fe2027c27b6a24505f548c6fd2e1076d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Blackadder ITC | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/4b0f1c076dfb624a4c79376edb6adf1b.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/4b0f1c076dfb624a4c79376edb6adf1b); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Bodoni MT Condensed | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e3d5ec4c7e5f3041c277d5cf3d518c71.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e3d5ec4c7e5f3041c277d5cf3d518c71); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Bradley Hand ITC | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/faeedc1fd74dce8e508221970594cb53.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/faeedc1fd74dce8e508221970594cb53); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Britannic Bold | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/8be4a2f403c2dc27187d892cca388e24.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/8be4a2f403c2dc27187d892cca388e24); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Broadway | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/19007e0c85468fd509414342e0ca9c68.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/19007e0c85468fd509414342e0ca9c68); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Brush Script MT | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/70927e11d1779ee1fa9a6b97278c01c1.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/70927e11d1779ee1fa9a6b97278c01c1); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Californian FB | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/12c13307742d4e286b692cce7ec65307.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/12c13307742d4e286b692cce7ec65307); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Castellar | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/0407ed4aef00d4db57f6001e710e0a85.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/0407ed4aef00d4db57f6001e710e0a85); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Centaur | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/1c5f0e6f12173bc7387400a837c28477.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/1c5f0e6f12173bc7387400a837c28477); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Chiller | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/cfde7197e2e3b805f27da82b6faa93e6.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/cfde7197e2e3b805f27da82b6faa93e6); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Colonna MT | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/91dd06fc110a0b45117e9338dcb9dcf9.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/91dd06fc110a0b45117e9338dcb9dcf9); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Consolas | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/1db29588408eadbd4406aae9238555eb.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/1db29588408eadbd4406aae9238555eb); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Constantia | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/0b9856633d4311a19df074ea509d8390.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/0b9856633d4311a19df074ea509d8390); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Corbel | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/29dc27977e417a98e56556776f41607c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/29dc27977e417a98e56556776f41607c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Curlz MT | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/3567d4a22b8d7e0d857a16df0afbaa1d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/3567d4a22b8d7e0d857a16df0afbaa1d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Edwardian Script ITC | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/84ae358e627d67d90bd613fcedc20c10.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/84ae358e627d67d90bd613fcedc20c10); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Elephant | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/bd7064a014d98f04fad0891082a6d521.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/bd7064a014d98f04fad0891082a6d521); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Engravers MT | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/3135c98efb051f346203c2f2ed708638.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/3135c98efb051f346203c2f2ed708638); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Eras Bold ITC | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/401b3afcdd14cf76b92956ecc7f7d8e6.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/401b3afcdd14cf76b92956ecc7f7d8e6); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Eras Light ITC | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/1de5ea5f55f61a4aa5a2a7fb39306cfe.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/1de5ea5f55f61a4aa5a2a7fb39306cfe); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Eras Medium ITC | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/7d1c0df7a7b61f4189d0ca451f707db0.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/7d1c0df7a7b61f4189d0ca451f707db0); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Felix Titling | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/87de6bcedaa28b9a81b24587815faf41.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/87de6bcedaa28b9a81b24587815faf41); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Footlight MT Light | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/82ac464059468e8dacb3d4d3f5c81253.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/82ac464059468e8dacb3d4d3f5c81253); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Forte | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/b01ef1168fc758bfbf3ef88fbee42ab4.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/b01ef1168fc758bfbf3ef88fbee42ab4); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Franklin Gothic Book | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9c9dbb999dd7068f51335d93cc7328bd.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9c9dbb999dd7068f51335d93cc7328bd); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Franklin Gothic Demi | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9c9dbb999dd7068f51335d93cc7328bd.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9c9dbb999dd7068f51335d93cc7328bd); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Franklin Gothic Demi Cond | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/69481e5634dda591e1dcbe06fc517650.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/69481e5634dda591e1dcbe06fc517650); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Franklin Gothic Heavy | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9d5211f76bf48fb129c4c940c3dd7cb8.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9d5211f76bf48fb129c4c940c3dd7cb8); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Franklin Gothic Medium Cond | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/c379b03bb3feeb76b9e05ed70791b22f.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/c379b03bb3feeb76b9e05ed70791b22f); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Freestyle Script | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/50eafd25cbb5f88fe7bb5cc77421bb49.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/50eafd25cbb5f88fe7bb5cc77421bb49); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| French Script MT | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/7af32ec3d600b58e6091d4d42ef19545.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/7af32ec3d600b58e6091d4d42ef19545); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Gill Sans MT Ext Condensed Bold | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/de869314fcbbc141d042ad3f0200e17c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/de869314fcbbc141d042ad3f0200e17c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Gill Sans Ultra Bold Condensed | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/fea8a931fd3f37ee4bc5e4c503ad6c9d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/fea8a931fd3f37ee4bc5e4c503ad6c9d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Gloucester MT Extra Condensed | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/141449d91ea53b0c3f08600f47ecbc0c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/141449d91ea53b0c3f08600f47ecbc0c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Goudy Old Style | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/78d7bdc55148aaa3307a1e8ad735c40f.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/78d7bdc55148aaa3307a1e8ad735c40f); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Goudy Stout | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/cb1a9d8c8e33d221e26a0d8247856c5e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/cb1a9d8c8e33d221e26a0d8247856c5e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Harlow Solid Italic | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/208c60a0a6b9e5ebbc434f9f24c2aeae.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/208c60a0a6b9e5ebbc434f9f24c2aeae); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Harrington | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/20c2d461d736b1073c57cffba76d35d1.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/20c2d461d736b1073c57cffba76d35d1); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Kartika | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/813afca1a50409ac32828551874eebb4.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/813afca1a50409ac32828551874eebb4); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Lucida Handwriting | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/31cfd9d14874e1be831c18bf5371ad7c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/31cfd9d14874e1be831c18bf5371ad7c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| MS Reference Sans Serif | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/c8f34a4d8d6a866f095261f987a237a8.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/c8f34a4d8d6a866f095261f987a237a8); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| MS Reference Specialty | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/da7d0632677782c7c4dd8b201ce85a8f.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/da7d0632677782c7c4dd8b201ce85a8f); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| OCR A Extended | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/fd6fa80f1e3345834599de891cca3f4c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/fd6fa80f1e3345834599de891cca3f4c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Tempus Sans ITC | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/261506be3344b3806eefa054f0f6fbf1.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/261506be3344b3806eefa054f0f6fbf1); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| TeXplus RM | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9ffd7ce2bf8b8c18e29930bd989d5f3d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9ffd7ce2bf8b8c18e29930bd989d5f3d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Tw Cen MT Condensed Extra Bold | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/a6e52658c34e3c3b5aab798f098593dc.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/a6e52658c34e3c3b5aab798f098593dc); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Vrinda | 3 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e1f8d16379efd89740cc26099c533a74.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e1f8d16379efd89740cc26099c533a74); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Angsana New | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e6a66ad34a680b8090172c85e4fece1d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e6a66ad34a680b8090172c85e4fece1d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| CG Times | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9685e3efc5a2270c6a47d201281ff08a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9685e3efc5a2270c6a47d201281ff08a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Cordia New | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/1059ad38e2a3bd334504686a2901eedb.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/1059ad38e2a3bd334504686a2901eedb); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| gulim | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/d9521b14999b76104e98b3d2f96079a1.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/d9521b14999b76104e98b3d2f96079a1); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| High Tower Text | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9e23421978544d8e00a00eb47740d280.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9e23421978544d8e00a00eb47740d280); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Imprint MT Shadow | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/b27d4ad5fd7f7c5044c7cbbf2dad758d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/b27d4ad5fd7f7c5044c7cbbf2dad758d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Informal Roman | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9596b74e7ea7b56c73c185ee751952eb.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9596b74e7ea7b56c73c185ee751952eb); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Jokerman | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/ea349025350b62eee57920eff9fe07b2.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/ea349025350b62eee57920eff9fe07b2); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Juice ITC | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/2986f53fe3f109fbac6f4e51015ef62c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/2986f53fe3f109fbac6f4e51015ef62c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Kristen ITC | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/ac63b6058f703ae46123ce6c383c6287.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/ac63b6058f703ae46123ce6c383c6287); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Kunstler Script | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/3e13bee6781c476e142a2bca4d9ab99c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/3e13bee6781c476e142a2bca4d9ab99c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Lucida Bright | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/dbca3735efb1a1b244da42542c8cdbeb.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/dbca3735efb1a1b244da42542c8cdbeb); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Lucida Calligraphy | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/4053837519f2a20cd733848fc1f9aa03.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/4053837519f2a20cd733848fc1f9aa03); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Lucida Fax | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/626419e51aa2bd64427a0e0921edab61.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/626419e51aa2bd64427a0e0921edab61); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Lucida Sans Typewriter | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/5e412ec1303cd5e91634c9e9f9a0f291.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/5e412ec1303cd5e91634c9e9f9a0f291); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Magneto | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e564ab2a94b273e5648ff05697eccad2.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e564ab2a94b273e5648ff05697eccad2); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Maiandra GD | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/0dc85fad34f00fbec56d8081d0f9267a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/0dc85fad34f00fbec56d8081d0f9267a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Malgun Gothic | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/3a025ae92e6446cec24efcb6d29e5bf3.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/3a025ae92e6446cec24efcb6d29e5bf3); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Matura MT Script Capitals | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e657499431090407423c1539e4af6364.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e657499431090407423c1539e4af6364); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Mistral | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/bda64a7a2fff9dda305ff3c1cb6ca679.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/bda64a7a2fff9dda305ff3c1cb6ca679); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Niagara Engraved | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e5b38fc8a405b9de2da31804f25b66af.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e5b38fc8a405b9de2da31804f25b66af); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Niagara Solid | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/deea7a74ea0e562b89edca5d89c75436.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/deea7a74ea0e562b89edca5d89c75436); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Old English Text MT | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/f3258385782c4c96aa24fe8b5d5f9782.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/f3258385782c4c96aa24fe8b5d5f9782); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Palace Script MT | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/0ea9f658ad61e247eca8603d6cceafff.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/0ea9f658ad61e247eca8603d6cceafff); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Papyrus | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/107eaa009712278feeb98175016a8a81.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/107eaa009712278feeb98175016a8a81); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Parchment | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/a844729f70281579017e703a84941e16.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/a844729f70281579017e703a84941e16); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Perpetua | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/afb95001b7a95a9cd3d5a8486fe0e1e1.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/afb95001b7a95a9cd3d5a8486fe0e1e1); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Perpetua Titling MT | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/60a72abfabeb852579f6de5afc2be918.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/60a72abfabeb852579f6de5afc2be918); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Playbill | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/13df2824b3f7c3ee824795faff222e15.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/13df2824b3f7c3ee824795faff222e15); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Poor Richard | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/1333b13c4f911f3160f1c2822573f70c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/1333b13c4f911f3160f1c2822573f70c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Pristina | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/1a46a20dd7fd087662b800869352303a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/1a46a20dd7fd087662b800869352303a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Ravie | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/2aeca327ae0e8ba04bf305f13cb1d589.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/2aeca327ae0e8ba04bf305f13cb1d589); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Rockwell | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/4aa3e37e571255737e5e6d4e9d9770a5.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/4aa3e37e571255737e5e6d4e9d9770a5); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Rockwell Condensed | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/d007e5e80d2a1e560bcd791f56860028.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/d007e5e80d2a1e560bcd791f56860028); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Rockwell Extra Bold | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/4aa3e37e571255737e5e6d4e9d9770a5.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/4aa3e37e571255737e5e6d4e9d9770a5); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Script MT Bold | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/b1f909b1cb3adb801a92229ea92613e1.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/b1f909b1cb3adb801a92229ea92613e1); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Showcard Gothic | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/d66fa62dabed66f2226a1b2d17da0579.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/d66fa62dabed66f2226a1b2d17da0579); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Snap ITC | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/d0725db773b460ed3a456370c80f875d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/d0725db773b460ed3a456370c80f875d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Stencil | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/93038c0f04a41f5be19797ba18f1bbbc.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/93038c0f04a41f5be19797ba18f1bbbc); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| tahoma | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/cd0381aa3322dff4babd137f03829c8c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/cd0381aa3322dff4babd137f03829c8c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Tw Cen MT | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9b62dc86f936227b3f7b367bd0b6c05e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9b62dc86f936227b3f7b367bd0b6c05e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Tw Cen MT Condensed | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/a6e52658c34e3c3b5aab798f098593dc.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/a6e52658c34e3c3b5aab798f098593dc); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Viner Hand ITC | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/6b836dcad1979649aaa53bc8187c9a0d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/6b836dcad1979649aaa53bc8187c9a0d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Vivaldi | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/c9a2e344b85728402fb6b8e2afa7f754.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/c9a2e344b85728402fb6b8e2afa7f754); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Wide Latin | 2 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/5afcc6e055927a510d82c2ced10c01c9.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/5afcc6e055927a510d82c2ced10c01c9); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| 20faces | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/945256fbf686f39790043c7f1330a600.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/945256fbf686f39790043c7f1330a600); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Adobe Caslon Pro | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/29e49f6f4a693b2e8b913296fa6afd37.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/29e49f6f4a693b2e8b913296fa6afd37); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Adobe Caslon Pro Bold | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/29e49f6f4a693b2e8b913296fa6afd37.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/29e49f6f4a693b2e8b913296fa6afd37); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Adobe Fan Heiti Std B | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/5d71ab7f83a03ffd46d4160baecbb594.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/5d71ab7f83a03ffd46d4160baecbb594); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Adobe Garamond Pro | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/51157ad3dd3870275b23205b1fe962bf.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/51157ad3dd3870275b23205b1fe962bf); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Adobe Garamond Pro Bold | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/51157ad3dd3870275b23205b1fe962bf.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/51157ad3dd3870275b23205b1fe962bf); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Aharoni | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/f93b4e44db468940f41bf6580ec35968.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/f93b4e44db468940f41bf6580ec35968); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Alienator | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/c014327cac8be50a8656d033ce24288e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/c014327cac8be50a8656d033ce24288e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Andalus | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/8269ea45efd48785edffb3ed85a5dc8a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/8269ea45efd48785edffb3ed85a5dc8a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| AngsanaUPC | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/070f54e1790ee0a495de46729b974b57.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/070f54e1790ee0a495de46729b974b57); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| AnimalTracks | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9a4989d9060a01b397f182b24ad8e966.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9a4989d9060a01b397f182b24ad8e966); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Animations | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/76819622c5429f897cc183240b1c4fa9.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/76819622c5429f897cc183240b1c4fa9); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Aparajita | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/0a5b5c55b73f577ff4ac8c9c31b4c183.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/0a5b5c55b73f577ff4ac8c9c31b4c183); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| AppleSDGothicNeo-Regular | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/af573d25034c111598327fb4d8c11a5a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/af573d25034c111598327fb4d8c11a5a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Arabic Typesetting | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/280e0867d189623928fcc0d7cfdaaa47.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/280e0867d189623928fcc0d7cfdaaa47); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| ArborisFolium | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/fae56d70bb34eef578c6a4e83aaaccd8.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/fae56d70bb34eef578c6a4e83aaaccd8); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Arctic | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/7d16b1fafe2c4fc457d289b606d1cee9.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/7d16b1fafe2c4fc457d289b606d1cee9); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Astro-SemiBold | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/7caf6389e7cbb791a523c546781a0221.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/7caf6389e7cbb791a523c546781a0221); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Bembo | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/b08fda9062b6b94ec9e02d3080016531.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/b08fda9062b6b94ec9e02d3080016531); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Birch Std | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/6f0607d5cef611ac79ee8b28f5c75b1b.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/6f0607d5cef611ac79ee8b28f5c75b1b); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Blackoak Std | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/f7e03688d14c563a628f3825cbc23c7f.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/f7e03688d14c563a628f3825cbc23c7f); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| BlockUp | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/d201527981049e1a86137f04b1b27dd0.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/d201527981049e1a86137f04b1b27dd0); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Border Corners | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/a3a93a658e3a5f7e176145114c4b7b8f.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/a3a93a658e3a5f7e176145114c4b7b8f); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Browallia New | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/558e1f7c26d9405ac41942266ebac11d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/558e1f7c26d9405ac41942266ebac11d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| BrowalliaUPC | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/81ceb3132faffd2044411342d8a0e0f4.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/81ceb3132faffd2044411342d8a0e0f4); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Brush Script Std | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/d44cb84b16ccdec82fa437a516e62b9a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/d44cb84b16ccdec82fa437a516e62b9a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Calibri Light | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/a78cfad3beb089a6ce86d4e280fa270b.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/a78cfad3beb089a6ce86d4e280fa270b); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Chaparral Pro | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/82707ca0316a93875e231b32340c919e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/82707ca0316a93875e231b32340c919e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Charlemagne Std | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/5f81cce6bdf565e42056ab354b37b83c.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/5f81cce6bdf565e42056ab354b37b83c); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Chinese Pinyin | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9cc6d1698ea207f1563855cdd7391ced.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9cc6d1698ea207f1563855cdd7391ced); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Classified | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e8458e6406a7244d6fc96461481cc63e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e8458e6406a7244d6fc96461481cc63e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| CordiaUPC | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/176f351ee3a9cbbef0b8b8c6a32679a9.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/176f351ee3a9cbbef0b8b8c6a32679a9); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| DaunPenh | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/5a68a4e0bd54f918326fdcf96028e3ff.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/5a68a4e0bd54f918326fdcf96028e3ff); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| David | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/b7a8f333c3a51885cf591cb7f00d6458.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/b7a8f333c3a51885cf591cb7f00d6458); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Davys | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/6e51d8139f13fb854162df7ce4ba5d71.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/6e51d8139f13fb854162df7ce4ba5d71); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| DFKai-SB | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/fe4f9dac99fb6b607c03981e6ce16869.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/fe4f9dac99fb6b607c03981e6ce16869); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| DilleniaUPC | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/00baad9fcb954edda045da7514566983.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/00baad9fcb954edda045da7514566983); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| DIN-Regular | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/43c793eb9fcfce5efd986389cceb93d0.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/43c793eb9fcfce5efd986389cceb93d0); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| DokChampa | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/119688cc24c7a1c78a469b0ed365edd7.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/119688cc24c7a1c78a469b0ed365edd7); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Earwig Factory | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/89805337c6b1530a814b180a8fed8cd6.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/89805337c6b1530a814b180a8fed8cd6); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| EastMarket | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/2ffa9c3db97bd6f11c13816a77cb0acc.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/2ffa9c3db97bd6f11c13816a77cb0acc); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Ebrima | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/1ba82d324736a8a9d4327d482c4627c4.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/1ba82d324736a8a9d4327d482c4627c4); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Euclid | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/c0152cb744b60409569eeee46b8897f3.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/c0152cb744b60409569eeee46b8897f3); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| EucrosiaUPC | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/ae32da51cb4715541b6a9f2a5e3939f0.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/ae32da51cb4715541b6a9f2a5e3939f0); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Euphemia | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/5c81010800152b142ea357ccbee8c40e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/5c81010800152b142ea357ccbee8c40e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| FrankRuehl | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/ca7f7cedd1df47077bfcf74dea2107dd.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/ca7f7cedd1df47077bfcf74dea2107dd); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Freemason | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/7e8e32987f0f5b99bbdddc1fd37a5d86.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/7e8e32987f0f5b99bbdddc1fd37a5d86); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| FreesiaUPC | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/6eb86166c2fa9fa798aae167631d6396.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/6eb86166c2fa9fa798aae167631d6396); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Futura Std Book | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/921351f146d78d55c8030239527bf2d6.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/921351f146d78d55c8030239527bf2d6); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Fuzzed | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/70238ee297ab526e426b08267472dcec.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/70238ee297ab526e426b08267472dcec); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Gabriola | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/b26b20369adfef2b3d65d266e0625fe2.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/b26b20369adfef2b3d65d266e0625fe2); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Gallaudet | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/de797677b588a7e5385c0a7e11cca694.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/de797677b588a7e5385c0a7e11cca694); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Giddyup Std | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/e67f8fef8c09b32f7a78c5809d3dab97.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/e67f8fef8c09b32f7a78c5809d3dab97); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Gisha | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/d3e5565884b751094df6825c37eeac5e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/d3e5565884b751094df6825c37eeac5e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| GoodDogBones | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/3860486dcb36184280243ae6846fcd66.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/3860486dcb36184280243ae6846fcd66); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| GulimChe | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/4e3a169357eb6823a72217333737dcf8.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/4e3a169357eb6823a72217333737dcf8); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Halloween | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/12d7d4ad920533927f5551fccb458ec6.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/12d7d4ad920533927f5551fccb458ec6); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Hazard | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/c4edf1d27452cd7159ce526bb947f357.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/c4edf1d27452cd7159ce526bb947f357); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Helvetica Narrow | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/92f56c1c4c3594cb41e7f09fdd4f7b9a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/92f56c1c4c3594cb41e7f09fdd4f7b9a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Helvetica-Condensed | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/a54e659b9ee7f19197420d01bc92dc9e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/a54e659b9ee7f19197420d01bc92dc9e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Matisse ITC | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/ecf665ef7aa7551d8e7603ad8d56c26e.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/ecf665ef7aa7551d8e7603ad8d56c26e); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Moebius | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/378341f1a5b64ec05da305fbd03ca93d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/378341f1a5b64ec05da305fbd03ca93d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| MS-Gothic | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/f4221c695de0fe4bd63bf82813b53175.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/f4221c695de0fe4bd63bf82813b53175); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Myriad Condensed Web | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/50545262a6164daad980a395b00d0c55.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/50545262a6164daad980a395b00d0c55); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Myriad Web | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/ef165517fbf966dc1b46f335ed7ea412.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/ef165517fbf966dc1b46f335ed7ea412); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| News Gothic MT | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/0fb6570a60563eb5563ea22165640c31.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/0fb6570a60563eb5563ea22165640c31); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Segoe UI Symbol | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/150ed9b2a009a71d2d819b5561167302.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/150ed9b2a009a71d2d819b5561167302); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| SpoqaHanSans-Bold | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/9595d09ddd4503301fa0572d7cb3df77.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/9595d09ddd4503301fa0572d7cb3df77); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Swis721 BT Italic | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/d97281294b63b449a93162fccd6121ec.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/d97281294b63b449a93162fccd6121ec); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| VnTime | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/3eb25089a2de0e6676d50fc28f44d88d.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/3eb25089a2de0e6676d50fc28f44d88d); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| VnTimeH | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/29650136413db4775953ef462bda019a.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/29650136413db4775953ef462bda019a); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |
| Westminster | 1 | 가능 | 라이선스 검토 필요 | OnlineWebFonts | [파일](https://db.onlinewebfonts.com/t/787588e6c611222861fa21b3ff9c12e4.woff2) | OnlineWebFonts WOFF2 응답 확인 (https://www.onlinewebfonts.com/download/787588e6c611222861fa21b3ff9c12e4); 실제 서비스 배포 전 원 권리자 사용 허가 확인 필요 |

## 사용 빈도 상위 30개

| 글꼴 | 사용 문서 | 다운로드 | 웹폰트 사용 | 웹폰트 판정 |
| --- | ---: | --- | --- | --- |
| 한양신명조 | 7549 | 불가 | 불가 | not-found |
| 명조 | 7533 | 불가 | 불가 | not-found |
| 휴먼명조 | 6369 | 불가 | 불가 | not-found |
| 바탕 | 5921 | 불가 | 불가 | not-found |
| HCI Poppy | 5732 | 불가 | 불가 | not-found |
| 굴림 | 5610 | 불가 | 불가 | not-found |
| 한컴바탕 | 5248 | 가능 | 가능 | available |
| 한양중고딕 | 5170 | 불가 | 불가 | not-found |
| 함초롬바탕 | 4370 | 가능 | 가능 | available |
| 맑은 고딕 | 3708 | 불가 | 불가 | not-found |
| 돋움 | 3374 | 불가 | 불가 | not-found |
| 함초롬돋움 | 3336 | 가능 | 가능 | available |
| HY헤드라인M | 3328 | 불가 | 불가 | not-found |
| 산세리프 | 3291 | 불가 | 불가 | not-found |
| HY중고딕 | 2803 | 불가 | 불가 | not-found |
| 바탕체 | 2590 | 가능 | 가능 | available |
| 돋움체 | 2589 | 가능 | 가능 | available |
| 굴림체 | 2506 | 불가 | 불가 | not-found |
| HY견고딕 | 2218 | 불가 | 불가 | not-found |
| HY신명조 | 2055 | 불가 | 불가 | not-found |
| HY견명조 | 1852 | 불가 | 불가 | not-found |
| 한양견고딕 | 1823 | 불가 | 불가 | not-found |
| 신명 견명조 | 1711 | 불가 | 불가 | not-found |
| 신명 태명조 | 1515 | 불가 | 불가 | not-found |
| 세명조 | 1459 | 불가 | 불가 | not-found |
| 한양견명조 | 1438 | 불가 | 불가 | not-found |
| 윤고딕130 | 1414 | 불가 | 불가 | not-found |
| HCI Tulip | 1410 | 불가 | 불가 | not-found |
| 신명조 | 1403 | 불가 | 불가 | not-found |
| 휴먼고딕 | 1309 | 불가 | 불가 | not-found |

## 전수 목록과 재현

전체 1,379개 글꼴의 사용 문서 수, 다운로드 가능 여부, 웹폰트 사용 가능 여부와 근거, 패키지·버전·라이선스 표기, 검증 URL, 판정 사유는 [TSV 상세 목록](assets/survey_korea_downloads_font_jsdelivr_20260815.tsv)에 기록했다.

`node scripts/survey_korea_downloads_font_jsdelivr.mjs --input <HWP|HWPX|디렉터리>`를 `devel`에서 실행하면 원시 임시 파일 없이 위 Markdown·TSV를 직접 다시 만든다. 실행 전에는 최신 바이너리를 만들기 위해 `cargo build --release`가 필요하다.
