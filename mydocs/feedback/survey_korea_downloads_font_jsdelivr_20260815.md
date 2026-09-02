# korea_downloads HWP/HWPX 글꼴과 jsDelivr 전수 조사

- **생성 시각**: 2026-08-15T08:31:40.211Z
- **기준 커밋**: `8dbb2557ee6e` (local `devel`)
- **입력**: `/Users/tsjang/Downloads/korea_downloads`의 HWP/HWPX 10,000건
- **파서**: `/Users/tsjang/rhwp/target/release/rhwp`의 `batch info --json --threads 8`
- **글꼴 범위**: HWP/HWPX DOCINFO의 한글·영어·한자·일어·기타·기호·사용자 7개 글꼴군 전체. 문서 내부 중복은 문서별 1회만 센다.
- **jsDelivr 판정**: Fontsource 카탈로그 2,096건, `font-loader.ts`에 등록된 jsDelivr GitHub 글꼴, jsDelivr 웹 검색 후보를 조사하고, 후보는 jsDelivr Data API의 파일 목록과 실제 CDN 글꼴 파일 응답까지 확인했다.

## 결과

| 지표 | 건수 |
| --- | ---: |
| 입력 문서 | 10,000 |
| 파싱 성공 | 9,948 |
| 파싱 실패 | 52 |
| 고유 선언 글꼴 | 1,414 |
| jsDelivr에서 다운로드 확인 | 85 |
| 검증 가능한 배포본 미발견 | 1,327 |
| 조회 오류 | 2 |

`미발견`은 인터넷의 임의 GitHub 저장소까지 부정하는 판정이 아니다. 공개 Fontsource 카탈로그와 jsDelivr 웹 검색, 기존 등록 GitHub 배포본을 이 스크립트의 동일 알고리즘으로 확인했을 때, **글꼴 바이트 파일을 실제로 내려받을 수 있는 jsDelivr URL을 검증하지 못했다**는 뜻이다. 패키지가 존재해도 원 글꼴과 동일한 서체인지, 라이선스가 해당 사용 목적을 허용하는지는 각 배포본의 라이선스를 별도로 확인해야 한다.

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
| 바탕체 | 2590 | jsDelivr 웹 검색 | `@noonnu/bareun-batang` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/bareun-batang@0.1.0/fonts/bareunbatang-400.woff) |
| 돋움체 | 2589 | jsDelivr 웹 검색 | `@noonnu/yi-sun-shin-dotum-m` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/yi-sun-shin-dotum-m@0.1.0/fonts/yisunshindotumm-normal.woff) |
| Times New Roman | 889 | jsDelivr 웹 검색 | `@canvas-fonts/times-new-roman` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/times-new-roman@1.0.4/Times New Roman.ttf) |
| KoPub바탕체 Light | 582 | jsDelivr 웹 검색 | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubBatang-Light.ttf) |
| KoPub돋움체 Light | 503 | jsDelivr 웹 검색 | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubDotum-Light.ttf) |
| 한컴돋움 | 495 | jsDelivr GitHub | `projectnoonnu/noonfonts_four` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff) |
| Arial | 464 | jsDelivr 웹 검색 | `@canvas-fonts/arial` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/arial@1.0.4/Arial.ttf) |
| 나눔고딕 Bold | 251 | Fontsource npm | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| KoPubWorld돋움체 Bold | 214 | jsDelivr 웹 검색 | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Dotum-Bold.otf) |
| 나눔고딕 | 205 | Fontsource npm | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| KoPub돋움체 Bold | 201 | jsDelivr 웹 검색 | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubDotum-Bold.ttf) |
| KoPub돋움체 Medium | 143 | jsDelivr 웹 검색 | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubDotum-Medium.ttf) |
| Arial Narrow | 141 | jsDelivr 웹 검색 | `@canvas-fonts/arial-narrow-bold` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/arial-narrow-bold@1.0.4/Arial Narrow Bold.ttf) |
| 나눔명조 | 116 | Fontsource npm | `@fontsource/nanum-myeongjo` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-myeongjo@5.3.0/files/nanum-myeongjo-0-400-normal.woff) |
| 나눔고딕 ExtraBold | 91 | Fontsource npm | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| 나눔고딕 Light | 65 | Fontsource npm | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| Garamond | 38 | jsDelivr 웹 검색 | `@fontsource/cormorant-garamond` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/cormorant-garamond@5.3.0/files/cormorant-garamond-cyrillic-300-italic.woff) |
| Courier New | 37 | jsDelivr 웹 검색 | `@canvas-fonts/courier-new` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/courier-new@1.0.4/Courier New.ttf) |
| KoPub바탕체 Bold | 34 | jsDelivr 웹 검색 | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubBatang-Bold.ttf) |
| KoPub바탕체 Medium | 29 | jsDelivr 웹 검색 | `font-kopub` | [파일](https://cdn.jsdelivr.net/npm/font-kopub@1.0.2/fonts/KoPubBatang-Medium.ttf) |
| MS Mincho | 27 | jsDelivr 웹 검색 | `@fontsource/shippori-mincho` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/shippori-mincho@5.3.0/files/shippori-mincho-0-400-normal.woff) |
| Arial Black | 23 | jsDelivr 웹 검색 | `@canvas-fonts/arial-black` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/arial-black@1.0.4/Arial Black.ttf) |
| 나눔명조 ExtraBold | 19 | Fontsource npm | `@fontsource/nanum-myeongjo` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-myeongjo@5.3.0/files/nanum-myeongjo-0-400-normal.woff) |
| 나눔고딕_코딩 | 18 | Fontsource npm | `@fontsource/nanum-gothic-coding` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic-coding@5.3.0/files/nanum-gothic-coding-0-400-normal.woff) |
| 한컴산뜻돋움 | 16 | jsDelivr GitHub | `projectnoonnu/noonfonts_four` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff) |
| Baskerville BT | 13 | jsDelivr 웹 검색 | `@fontsource/libre-baskerville` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/libre-baskerville@5.3.0/files/libre-baskerville-latin-400-italic.woff) |
| Comic Sans MS | 12 | jsDelivr 웹 검색 | `@canvas-fonts/comic-sans-ms` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/comic-sans-ms@1.0.4/Comic Sans MS.ttf) |
| KoPubWorld돋움체 Light | 12 | jsDelivr 웹 검색 | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Dotum-Light.otf) |
| KoPubWorld바탕체 Light | 12 | jsDelivr 웹 검색 | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Batang-Light.otf) |
| Bodoni Bd BT | 11 | jsDelivr 웹 검색 | `@fontsource/bodoni-moda` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/bodoni-moda@5.3.0/files/bodoni-moda-latin-400-italic.woff) |
| Bodoni Bk BT | 11 | jsDelivr 웹 검색 | `@fontsource/bodoni-moda` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/bodoni-moda@5.3.0/files/bodoni-moda-latin-400-italic.woff) |
| BrushScript BT | 10 | jsDelivr 웹 검색 | `@fontsource/nanum-brush-script` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-brush-script@5.3.0/files/nanum-brush-script-0-400-normal.woff) |
| KoPubWorld돋움체 Medium | 10 | jsDelivr 웹 검색 | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Dotum-Medium.otf) |
| MS Gothic | 10 | jsDelivr 웹 검색 | `@fontsource/zen-maru-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/zen-maru-gothic@5.3.0/files/zen-maru-gothic-10-300-normal.woff) |
| 나눔명조OTF ExtraBold | 9 | jsDelivr 웹 검색 | `@kfonts/nanum-myeongjo-otf` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-myeongjo-otf@0.2.0/src/NanumMyeongjoExtraBold.otf) |
| Times New Roman Bold | 8 | jsDelivr 웹 검색 | `@canvas-fonts/times-new-roman-bold` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/times-new-roman-bold@1.0.4/Times New Roman Bold.ttf) |
| 다음_SemiBold | 7 | jsDelivr 웹 검색 | `alibabapuhuiti-3-75-semibold` | [파일](https://cdn.jsdelivr.net/npm/alibabapuhuiti-3-75-semibold@1.0.0/AlibabaPuHuiTi-3-75-SemiBold.otf) |
| MS UI Gothic | 6 | jsDelivr 웹 검색 | `@fontsource/zen-maru-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/zen-maru-gothic@5.3.0/files/zen-maru-gothic-10-300-normal.woff) |
| SimHei | 6 | jsDelivr 웹 검색 | `react-native-font-sim` | [파일](https://cdn.jsdelivr.net/npm/react-native-font-sim@2.0.1/fonts/SimHei.ttf) |
| Calisto MT | 5 | jsDelivr 웹 검색 | `@fontsource/calistoga` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/calistoga@5.3.0/files/calistoga-latin-400-normal.woff) |
| Helvetica Neue | 4 | jsDelivr 웹 검색 | `@marcius-studio/font` | [파일](https://cdn.jsdelivr.net/npm/@marcius-studio/font@0.0.1/HelveticaNeueCyr/HelveticaNeueCyr-Black.ttf) |
| KoPubWorld바탕체 Medium | 4 | jsDelivr 웹 검색 | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Batang-Medium.otf) |
| 에스코어 드림 3 Light | 3 | jsDelivr 웹 검색 | `@noonnu/s-core-dream-3-light` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/s-core-dream-3-light@0.1.0/fonts/s-coredream-3light-normal.woff) |
| Bodoni MT | 3 | jsDelivr 웹 검색 | `@fontsource/bodoni-moda` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/bodoni-moda@5.3.0/files/bodoni-moda-latin-400-italic.woff) |
| Century Schoolbook | 3 | jsDelivr 웹 검색 | `centschbook-mono` | [파일](https://cdn.jsdelivr.net/npm/centschbook-mono@3.2.1/Century-Schoolbook-Monospace-BT.ttf) |
| Cooper Black | 3 | jsDelivr 웹 검색 | `fonts-archive-cooper-black` | [파일](https://cdn.jsdelivr.net/npm/fonts-archive-cooper-black@0.0.0/CooperBlack Italic-Regular.otf) |
| Helvetica | 3 | jsDelivr 웹 검색 | `helvetica-original` | [파일](https://cdn.jsdelivr.net/npm/helvetica-original@1.0.0/Black/Helvetica-Black.ttf) |
| KoPubWorld바탕체 Bold | 3 | jsDelivr 웹 검색 | `font-kopubworld` | [파일](https://cdn.jsdelivr.net/npm/font-kopubworld@1.0.3/fonts/KoPubWorld-Batang-Bold.otf) |
| MT Extra | 3 | jsDelivr 웹 검색 | `@fontsource/fira-sans-extra-condensed` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/fira-sans-extra-condensed@5.3.0/files/fira-sans-extra-condensed-cyrillic-100-italic.woff) |
| Myeongjo | 3 | jsDelivr 웹 검색 | `@fontsource/nanum-myeongjo` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-myeongjo@5.3.0/files/nanum-myeongjo-0-400-normal.woff) |
| Segoe UI | 3 | jsDelivr 웹 검색 | `@fontpkg/segoe-ui` | [파일](https://cdn.jsdelivr.net/npm/@fontpkg/segoe-ui@5.67.0/segoeui.ttf) |
| 나눔바른고딕 Light | 2 | jsDelivr 웹 검색 | `@kfonts/nanum-barun-gothic` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-barun-gothic@0.3.0/NanumBarunGothicLight.woff) |
| arial | 2 | jsDelivr 웹 검색 | `@canvas-fonts/arial` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/arial@1.0.4/Arial.ttf) |
| MS Song | 2 | jsDelivr 웹 검색 | `@fontsource/song-myung` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/song-myung@5.3.0/files/song-myung-10-400-normal.woff) |
| NanumGothic | 2 | Fontsource npm | `@fontsource/nanum-gothic` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/nanum-gothic@5.3.0/files/nanum-gothic-0-400-normal.woff) |
| Noto | 2 | jsDelivr 웹 검색 | `@fontsource/noto-sans-jp` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/noto-sans-jp@5.3.0/files/noto-sans-jp-0-100-normal.woff) |
| Vladimir Script | 2 | jsDelivr 웹 검색 | `fonts-archive-vladimir-script` | [파일](https://cdn.jsdelivr.net/npm/fonts-archive-vladimir-script@0.0.1/VladimirScript.ttf) |
| 62570체 | 1 | jsDelivr 웹 검색 | `@noonnu/62570che` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/62570che@0.1.0/fonts/62570-normal.woff) |
| 나눔고딕OTF | 1 | jsDelivr 웹 검색 | `@kfonts/nanum-gothic-otf` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-gothic-otf@0.2.0/src/NanumGothic.otf) |
| 나눔고딕OTF Bold | 1 | jsDelivr 웹 검색 | `@kfonts/nanum-gothic-otf` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-gothic-otf@0.2.0/src/NanumGothicBold.otf) |
| 나눔바른고딕OTF | 1 | jsDelivr 웹 검색 | `@kfonts/nanum-barun-gothic-yet-hangul-otf` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-barun-gothic-yet-hangul-otf@0.2.0/src/NanumBarunGothic-YetHangul.otf) |
| 나눔스퀘어OTF | 1 | jsDelivr 웹 검색 | `@kfonts/nanum-square-otf` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-square-otf@0.2.0/src/NanumSquareB.otf) |
| 새바탕 | 1 | jsDelivr GitHub | `projectnoonnu/noonfonts_2104` | [파일](https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff) |
| Apple SD 산돌고딕 Neo 일반체 | 1 | jsDelivr 웹 검색 | `font-applesdgothicneo` | [파일](https://cdn.jsdelivr.net/npm/font-applesdgothicneo@1.0.3/fonts/100_AppleSDGothicNeo-Thin.otf) |
| Arial (W1) | 1 | jsDelivr 웹 검색 | `@canvas-fonts/arial` | [파일](https://cdn.jsdelivr.net/npm/@canvas-fonts/arial@1.0.4/Arial.ttf) |
| DejaVu Serif | 1 | Fontsource npm | `@fontsource/dejavu-serif` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/dejavu-serif@5.3.0/files/dejavu-serif-latin-400-italic.woff) |
| FangSong | 1 | jsDelivr 웹 검색 | `@fontpkg/zhuque-fangsong-technical-preview` | [파일](https://cdn.jsdelivr.net/npm/@fontpkg/zhuque-fangsong-technical-preview@0.212.0/ZhuqueFangsong-Regular.ttf) |
| Futura Hv BT | 1 | jsDelivr 웹 검색 | `futura-font` | [파일](https://cdn.jsdelivr.net/npm/futura-font@1.0.0/FuturaBT-Medium.ttf) |
| Futura Std ExtraBold | 1 | jsDelivr 웹 검색 | `fonts-archive-futura-std` | [파일](https://cdn.jsdelivr.net/npm/fonts-archive-futura-std@0.0.0/FuturaStd-ExtraBold.otf) |
| Futura Std Light | 1 | jsDelivr 웹 검색 | `fonts-archive-futura-std` | [파일](https://cdn.jsdelivr.net/npm/fonts-archive-futura-std@0.0.0/FuturaStd-Light.otf) |
| Futura Std Medium | 1 | jsDelivr 웹 검색 | `fonts-archive-futura-std` | [파일](https://cdn.jsdelivr.net/npm/fonts-archive-futura-std@0.0.0/FuturaStd-Medium.otf) |
| HCRDotum | 1 | jsDelivr 웹 검색 | `@noonnu/hcr-dotum` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/hcr-dotum@0.1.0/fonts/hcrdotum-normal.woff) |
| Helvetica 65 Medium | 1 | jsDelivr 웹 검색 | `@duppla-font/helvetica-now` | [파일](https://cdn.jsdelivr.net/npm/@duppla-font/helvetica-now@1.0.0/files/HelveticaNowTextMedium.otf) |
| KBIZ한마음명조 R | 1 | jsDelivr 웹 검색 | `@noonnu/kbiz-hanmaum-myungjo` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/kbiz-hanmaum-myungjo@0.1.0/fonts/kbizhanmaummyungjo-normal.woff) |
| KoPubDotumMedium | 1 | jsDelivr 웹 검색 | `@noonnu/kopubdotummedium` | [파일](https://cdn.jsdelivr.net/npm/@noonnu/kopubdotummedium@0.0.1/KoPubDotumMedium.woff) |
| Nanum Barun Gothic | 1 | jsDelivr 웹 검색 | `@kfonts/nanum-barun-gothic` | [파일](https://cdn.jsdelivr.net/npm/@kfonts/nanum-barun-gothic@0.3.0/NanumBarunGothic.woff) |
| Noto Sans CJK JP Regular | 1 | jsDelivr 웹 검색 | `noto-sans-cjk-jp` | [파일](https://cdn.jsdelivr.net/npm/noto-sans-cjk-jp@1.0.1/fonts/NotoSansCJKjp-Regular.woff) |
| Noto Sans KR Medium | 1 | Fontsource npm | `@fontsource/noto-sans-kr` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/noto-sans-kr@5.3.0/files/noto-sans-kr-0-100-normal.woff) |
| Pretendard | 1 | Fontsource npm | `@fontsource/pretendard` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/pretendard@5.3.0/files/pretendard-latin-100-normal.woff) |
| Pretendard Light | 1 | Fontsource npm | `@fontsource/pretendard` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/pretendard@5.3.0/files/pretendard-latin-100-normal.woff) |
| Roboto | 1 | Fontsource npm | `@fontsource/roboto` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/roboto@5.3.0/files/roboto-cyrillic-100-italic.woff) |
| Yu Mincho | 1 | jsDelivr 웹 검색 | `@fontsource/shippori-mincho` | [파일](https://cdn.jsdelivr.net/npm/@fontsource/shippori-mincho@5.3.0/files/shippori-mincho-0-400-normal.woff) |

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
| 바탕체 | 2590 | available |
| 돋움체 | 2589 | available |
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
