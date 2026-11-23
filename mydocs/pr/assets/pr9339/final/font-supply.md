# PR #9339 정본 보호 글꼴 확인

2026-11-23 메인테이너 결정에 따라 명시적 공급 대상 312개를 모두 확인했다. devel `321ab8f977`의 기존 해시와 일치하는 파일이 305개, 공통으로 허용한 Windows 11 설치본이 6개, 화학물질 표시 기준(`chemical`)에만 허용한 돋움체가 1개다. [공식 공급 조건과 허용 해시 쌍](../../../../verification/protected-font-supply/manifest.json)

| 파일 | 검증 당시 SHA-256 | 함께 허용한 SHA-256 | 해당 문서 |
| --- | --- | --- | --- |
| `batang.ttc` | `0ea38ac53e8d293a9598d913cb49740808164dfef972b10bfd3115159c5ac2c8` | `51d9a8d4205239c865e24b6eb7e6e6dbacfd59c3e2c46f623db0af45e169c9d8` | chemical, outline, sample16-hwp2022, sample16-hwp2024, sample16-hwpx |
| `H2MJSM.TTF` (HY신명조) | `49dd1ef0caa6989a7dd95b484fdd052cbc846f6514c0c08421ed03a6cb1669cc` | `f78f538885b29f956cf84286e430898462d11430604625857ad587c0ea73925b` | decree |
| `arial.ttf` | `c9b76220a5be42ead4733611e417cd65c5fd8aeaa33eb56576ac378a37d130a1` | `edfbd22a3d088fc2f5d7c79b8c9c732df6dc878a60f0a80be0f92a3c7a634404` | chemical, sample16-hwp2022, sample16-hwp2024, sample16-hwpx, seo-helvetica-styles |
| `arialbd.ttf` | `766f06ac8761f82f25d032a220e89438f6064591af9915061f20b949efdedf69` | `4d59199834e7439c67ea4949407639961060b21b173833f566eb60ac6fd8f5af` | chemical, sample16-hwp2022, sample16-hwp2024, sample16-hwpx, seo-helvetica-styles |
| `arialbi.ttf` | `e432c1b3568bd207e30e3d52d41e6431ea53a679eb52dd79b1a36aa6a4f3b994` | `d68bdf8ebfe67a72e6db02b104e8062de5e75c358ba9bb7ef5d0e5c70272a1c5` | chemical, sample16-hwp2022, sample16-hwp2024, sample16-hwpx, seo-helvetica-styles |
| `ariali.ttf` | `79a5742b865cf06891d4ae43e42cac0e5169c7b7b06b5b3c24ef8e0966d27b62` | `4487a06c1b8de98deb0cab2c04a876cbc8f31b7e397dc1f2b2021c0746321d7c` | chemical, sample16-hwp2022, sample16-hwp2024, sample16-hwpx, seo-helvetica-styles |
| `dotumche.ttf` | `097a83c15227aa0b01bba1c446d2cf67ac04694052e60d41fef557ed5a0a8a62` | `c2c030ca896d46ff28dae17663e9fe22ab2fc6e6518c96d75e15ca8fb441bd2f` | chemical 한정 |

Windows 설치본으로 공급이 가능해진 문서 6종을 기준 devel과 후보에서 다시 출력했다. Native·fresh WASM 각각 276쪽 모두 두 소스의 PNG가 같았다. 쪽별 공식 하한에 미달한 쪽은 두 소스에서 똑같이 Native 65쪽, WASM 63쪽이었다. [실제 공급 해시·WASM face·문서별 결과](protected-windows-fonts.json) · [쪽별 점수와 PNG 해시](protected-windows-fonts.tsv)

## 직접 대조한 파일

19:29 KST에 검증 당시 해시와 일치한 파일은 `ttfs` 안에서 296개, 그 밖의 로컬 경로나 복원 결과에서 9개였다. 표의 Windows 설치본은 이번에 각각 `C:\Windows\Fonts`에서 다시 읽어 SHA-256을 계산했다. [312개 전체의 기존 SHA-256과 폴더 내 일치 경로](ttfs-inventory.tsv)

폴더 대조 중 devel에 병합된 #9359(`321ab8f977`)의 목록도 확인했다. 23종 문서와 두 백엔드의 비교에서 바뀐 값은 검증 기록 파일의 SHA-256 46곳뿐이다. 새 기록 파일의 실제 해시를 검증했고, 공급 모드·파일명·글꼴 SHA-256·순서와 4,046행 보호 하한표가 같음을 확인했다. [검증 기록 정리 PR](https://github.com/edwardkim/rhwp/pull/9359)

18:16 조사 때는 로컬 `ttfs`의 하위 폴더와 ZIP 2개의 글꼴 후보가 1,068개였다. 19:29 재확인 때는 `hft-fonts-cmap-v2`의 387개를 포함해 1,455개였다. 새 `hft-fonts-cmap-v2/SPJGT.otf`의 SHA-256은 `6f6cc714a13a2a68f4f9446c082ff7e6514d3088b14a954edf82933a0417b51a`로 정본과 같다. 기존 `hft/SPJGT.otf`는 `484d6046…`이다. 이후 화학물질 문서의 Native·WASM 공급 목록에서 Arial 4종과 바탕 TTC는 이번에 허용된 설치본으로 확인했다.

처음 찾던 11개 중 이 폴더에서 해시가 일치한 것은 새 SPJGT 한 개다. Arial 네 파일과 바탕·굴림 TTC는 동명 파일의 해시가 다르다. 필수 굴림 파일은 별도 로컬 변환기 자료에서 찾아 SHA-256 `4b9ac63e8920ed1bae29c068025ed30493464c18ee18617887daa18e59189226`을 확인했다.

두 SPJGT의 내부 이름은 모두 `HY Junggothic`이다. 문서별 정본에서 화학물질 표시 기준은 `6f6cc714…`, 편람 HWP와 영월 문서는 `484d6046…`를 명시한다. 선호 유니코드 문자표의 항목 수는 기존 536개, 새 파일 893개다. 새 파일의 두 유니코드 문자표에는 `○`(U+25CB)가 `hft343B`에 연결되어 있고, `•`(U+2022)는 두 파일 모두 매핑이 없다.

돋움체는 `ttfs/windows/gulim.ttc`의 네 번째 서체(index 3, Version 5.04)로 확인됐다. 이 서체를 단일 TTF로 추출한 SHA-256은 `3674103f23382f9bf40e650d8860b9ce2338ae6bf9400bdcf0b56688743f8f53`로, 정본의 `dotumche.ttf`와 다르다. [파일별 해시 대조](font-program-lookup.tsv)

화학물질 표시 기준에 필요한 단일 TTF의 내부 서체명은 `DotumChe`·`돋움체`다. 한컴오피스 2020에서 조사한 경로는 `/opt/hnc/hoffice11/Shared/TTF/Install/dotumche.ttf`이며, 현재 정본은 Native와 WASM 모두 53개 파일 중 50번째에 같은 SHA-256을 지정한다. [당시 파일 경로·해시·서체명](../../issue_8151/font-programs.tsv) · [문서별 공급 목록](../../../../verification/protected-font-supply/manifest.json)

| 11월 23일 추가 대조 | 내부 서체명·버전 | 유니코드 문자표의 한글 완성형 음절 | SHA-256 |
| --- | --- | ---: | --- |
| 처음 제공된 `ttfs/chemical/dotumche.ttf` (11,548바이트) | DotumCheUT 001.000 | 0 / 11,172 | `0f93ebba3db375268404e834f11eaefc349252799d6ba3cef2906cd16451f4b1` |
| EverythingFonts의 공개 미리보기 WOFF2 | DotumChe 2.21 | 11,172 / 11,172 | `2170a738fb09250e6f099e560b259a04ec4ea7cd21b2f3bb5bfd55d07508c873` |
| 이번에 제공된 `ttfs/chemical/DotumChe-Regular.ttf` (4,568,508바이트) | DotumChe Regular 2.21 | 11,172 / 11,172 | `c2c030ca896d46ff28dae17663e9fe22ab2fc6e6518c96d75e15ca8fb441bd2f` |

제공된 Regular TTF의 유니코드 문자표와 가로 글리프 메트릭은 공개 미리보기와 같다. TTF의 이름 테이블에는 영문 `DotumChe`가 있다. 과거 공급 기록에는 한글 이름 `돋움체`도 있다. 실제 CLI는 `--document chemical`에서 제공본을 허용했고 나머지 22개 문서, 문서 키 생략, WOFF2 제공은 모두 거부했다. 기존 Windows 설치본 6개의 공통 검사와 글꼴·시각 검증 관련 자동 검사 47개도 통과했다. [EverythingFonts 미리보기](https://everythingfonts.com/font/dotumche#web) · [검사 명령](../../../../verification/protected-font-supply/README.md)

화학물질 표시 기준 103쪽에서 Regular TTF를 각 백엔드의 50번째 순서에 넣고 나머지 52개 파일은 정본 또는 허용된 해시로 공급했다. 96 DPI 한컴 Print 대비 쪽별 최저 실루엣 일치율은 Native 99.99%, fresh WASM 100.00%였다(소수 둘째 자리 표시). 두 백엔드 모두 103쪽의 PNG가 기준 devel과 후보에서 각각 같았고 쪽별 하한 미달은 0건이었다. Native 최저인 26쪽에서 표의 제목·두 열의 괘선·뒤 문단 줄바꿈을 인쇄본과 직접 대조했다. [쪽별 하한·실측값·PNG SHA-256](chemical-dotumche-diagnostic.tsv)

같은 후보에서 해당 TTF만 제외하고 나머지 52개를 같은 순서로 공급해 다시 렌더링했다. Native·fresh WASM 각각 103쪽 모두 TTF를 넣을 때와 뺄 때의 PNG가 같았다. 문서별 허용을 반영한 뒤에는 53개 파일의 해시와 순서를 확인하고 기준·후보를 새 출력 경로에서 다시 측정했다. [공급 조건과 재검증 결과](chemical-dotumche-verification.json) · [렌더러별 빌드와 소스 SHA](renderer-builds.json)

Windows 11의 `C:\Windows\Fonts\H2MJSM.TTF`에 기록된 서체명은 한글 `HY신명조`, 영문 `HYSinMyeongJo-Medium`이다. 시행령 문서(`decree`)의 Native·WASM 공급 목록은 `49dd1ef0caa6…`를 지정하며, 이번 검증에서는 설치본 `f78f538885b2…`를 같은 순서에 공급했다.

공개 생성법과 같은 빈 HWPX로 한양중고딕 독립 표본을 만들었다. 입력 SHA-256은 `79c2714845d7ed0941534bc765665e7f0302dac1333abfce84d21daef3d10592`이고, 한컴 2020 11.0.0.9136에서 재인쇄한 7쪽의 본문 11,187자가 생성 순서와 일치했다. 새 PDF의 SHA-256은 `618695320aee01224d30c14511ce319fa4b0883e1c97cb45d5bf8d40741c44c3`이다. 독립 자료의 모든 비어 있지 않은 Type 3 글리프 프로그램을 당시 해시와 대조했다. [표본 생성 기록](../../issues7754_7765/source44/source56-independent-specimen-jobs.json)

| 복원한 프로그램 | 검증한 비어 있지 않은 글리프 | 생성 시각 (Unix 초) | 다시 생성한 파일의 SHA-256 |
| --- | ---: | ---: | --- |
| `independent-symbols-regular.otf` | 14 | 1795330713 | `72bd3af45659c2f7bc6eafaf7cd859f525089581b89d2136da99fa0ed93b37f2` |
| `independent-junggothic-regular.otf` | 11,186 | 1795336888 | `91161f1cd99235e25d93d3b317961ea6b06811f72ca96ad7367c61f4d63ca405` |

각 생성 스크립트의 글리프 순서·외곽선·전진폭·이름 테이블과 당시 생성 시각을 사용했다. 재인쇄본의 글리프별 해시와 최종 OTF의 전체 바이트 해시가 각각 기록과 일치했다. 검증용 프로그램은 로컬에서만 사용했다. [기호 생성법](../../issues7754_7765/source44/reproduction/build_source57_independent_symbols.py.txt) · [한글 생성법](../../issues7754_7765/source47/reproduction/build_source93_independent_junggothic.py.txt)
