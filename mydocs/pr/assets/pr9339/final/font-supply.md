# PR #9339 정본 보호 글꼴 확인

2026-11-23 19:29 KST 재확인 결과, devel `321ab8f977`의 고유 프로그램 312개 중 305개는 로컬 파일의 SHA-256을 확인했다. 나머지 7개가 필요한 문서 7종은 백엔드별 379쪽이다. [공식 공급 조건](../../../../verification/protected-font-supply/manifest.json)

| 파일 | 필요한 SHA-256 | 해당 문서 |
| --- | --- | --- |
| `0ea38ac53e8d-batang.TTC` | `0ea38ac53e8d293a9598d913cb49740808164dfef972b10bfd3115159c5ac2c8` | chemical, outline, sample16-hwp2022, sample16-hwp2024, sample16-hwpx |
| `49dd1ef0caa6-H2MJSM.TTF` | `49dd1ef0caa6989a7dd95b484fdd052cbc846f6514c0c08421ed03a6cb1669cc` | decree |
| `arial.ttf` | `c9b76220a5be42ead4733611e417cd65c5fd8aeaa33eb56576ac378a37d130a1` | chemical, sample16-hwp2022, sample16-hwp2024, sample16-hwpx, seo-helvetica-styles |
| `arialbd.ttf` | `766f06ac8761f82f25d032a220e89438f6064591af9915061f20b949efdedf69` | chemical, sample16-hwp2022, sample16-hwp2024, sample16-hwpx, seo-helvetica-styles |
| `arialbi.ttf` | `e432c1b3568bd207e30e3d52d41e6431ea53a679eb52dd79b1a36aa6a4f3b994` | chemical, sample16-hwp2022, sample16-hwp2024, sample16-hwpx, seo-helvetica-styles |
| `ariali.ttf` | `79a5742b865cf06891d4ae43e42cac0e5169c7b7b06b5b3c24ef8e0966d27b62` | chemical, sample16-hwp2022, sample16-hwp2024, sample16-hwpx, seo-helvetica-styles |
| `dotumche.ttf` | `097a83c15227aa0b01bba1c446d2cf67ac04694052e60d41fef557ed5a0a8a62` | chemical |

## 직접 대조한 파일

필수 고유 프로그램 312개를 기준으로 보면 `ttfs` 안에서 296개, 그 밖의 로컬 경로나 복원 결과에서만 9개를 확보했다. 미확보 7개는 위 표와 같다. [312개 전체의 SHA-256과 폴더 내 일치 경로](ttfs-inventory.tsv)

폴더 대조 중 devel에 병합된 #9359(`321ab8f977`)의 목록도 확인했다. 23종 문서와 두 백엔드의 비교에서 바뀐 값은 검증 기록 파일의 SHA-256 46곳뿐이다. 새 기록 파일의 실제 해시를 검증했고, 공급 모드·파일명·글꼴 SHA-256·순서와 4,046행 보호 하한표가 같음을 확인했다. [검증 기록 정리 PR](https://github.com/edwardkim/rhwp/pull/9359)

18:16 조사 때는 로컬 `ttfs`의 하위 폴더와 ZIP 2개의 글꼴 후보가 1,068개였다. 19:29 재확인 때는 `hft-fonts-cmap-v2`의 387개를 포함해 1,455개였다. 새 `hft-fonts-cmap-v2/SPJGT.otf`의 SHA-256은 `6f6cc714a13a2a68f4f9446c082ff7e6514d3088b14a954edf82933a0417b51a`로 정본과 같다. 기존 `hft/SPJGT.otf`는 `484d6046…`이다. 화학물질 문서의 양쪽 공급 목록에는 새 파일을 반영한 뒤에도 다른 글꼴 6개가 미확보다.

처음 찾던 11개 중 이 폴더에서 해시가 일치한 것은 새 SPJGT 한 개다. Arial 네 파일과 바탕·굴림 TTC는 동명 파일의 해시가 다르다. 필수 굴림 파일은 별도 로컬 변환기 자료에서 찾아 SHA-256 `4b9ac63e8920ed1bae29c068025ed30493464c18ee18617887daa18e59189226`을 확인했다.

두 SPJGT의 내부 이름은 모두 `HY Junggothic`이다. 문서별 정본에서 화학물질 표시 기준은 `6f6cc714…`, 편람 HWP와 영월 문서는 `484d6046…`를 명시한다. 선호 유니코드 문자표의 항목 수는 기존 536개, 새 파일 893개다. 새 파일의 두 유니코드 문자표에는 `○`(U+25CB)가 `hft343B`에 연결되어 있고, `•`(U+2022)는 두 파일 모두 매핑이 없다.

돋움체는 `ttfs/windows/gulim.ttc`의 네 번째 서체(index 3, Version 5.04)로 확인됐다. 이 서체를 단일 TTF로 추출한 SHA-256은 `3674103f23382f9bf40e650d8860b9ce2338ae6bf9400bdcf0b56688743f8f53`로, 정본의 `dotumche.ttf`와 다르다. [파일별 해시 대조](font-program-lookup.tsv)

공개 생성법과 같은 빈 HWPX로 한양중고딕 독립 표본을 만들었다. 입력 SHA-256은 `79c2714845d7ed0941534bc765665e7f0302dac1333abfce84d21daef3d10592`이고, 한컴 2020 11.0.0.9136에서 재인쇄한 7쪽의 본문 11,187자가 생성 순서와 일치했다. 새 PDF의 SHA-256은 `618695320aee01224d30c14511ce319fa4b0883e1c97cb45d5bf8d40741c44c3`이다. 독립 자료의 모든 비어 있지 않은 Type 3 글리프 프로그램을 당시 해시와 대조했다. [표본 생성 기록](../../issues7754_7765/source44/source56-independent-specimen-jobs.json)

| 복원한 프로그램 | 검증한 비어 있지 않은 글리프 | 생성 시각 (Unix 초) | 다시 생성한 파일의 SHA-256 |
| --- | ---: | ---: | --- |
| `independent-symbols-regular.otf` | 14 | 1795330713 | `72bd3af45659c2f7bc6eafaf7cd859f525089581b89d2136da99fa0ed93b37f2` |
| `independent-junggothic-regular.otf` | 11,186 | 1795336888 | `91161f1cd99235e25d93d3b317961ea6b06811f72ca96ad7367c61f4d63ca405` |

각 생성 스크립트의 글리프 순서·외곽선·전진폭·이름 테이블과 당시 생성 시각을 사용했다. 재인쇄본의 글리프별 해시와 최종 OTF의 전체 바이트 해시가 각각 기록과 일치했다. 검증용 프로그램은 로컬에서만 사용했다. [기호 생성법](../../issues7754_7765/source44/reproduction/build_source57_independent_symbols.py.txt) · [한글 생성법](../../issues7754_7765/source47/reproduction/build_source93_independent_junggothic.py.txt)
