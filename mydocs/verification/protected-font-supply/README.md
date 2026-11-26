# 정본 보호 문서의 글꼴 공급 자료

[manifest.json](manifest.json)은 Linux·Windows·macOS의 정본 보호 검증에서 사용하는
문서별 Native·WASM 글꼴 공급 조건이다. 경로는 저장소 루트를 기준으로 해석한다.

- `documents[].backends`에서 문서 key와 backend를 선택한다.
- `supply_mode`가 `explicit_files`이면 `font_set`의 파일명·SHA-256·배열 순서로
  `--font-path`를 지정한다. 자동 선택 조건에서는 `font_set`을 공급 환경 확인에 사용한다.
- Arial 4종, 바탕 TTC, HY신명조는 `accepted_sha256_pairs`에 파일별로 등록한 두 SHA-256을 모두 허용한다.
  공급 목록이 쌍의 한 해시를 지정하면 같은 쌍의 다른 해시도 그 순서와 face에 공급할 수 있다.
  실제 파일을 해시하여 선택한 SHA-256을 기록하고, 같은 공급 조건에서 기준과 후보의 쪽별 하한을 검증한다.
- 문서 항목의 `accepted_sha256_pairs`는 그 문서의 Native·WASM 공급 목록에서만 적용한다.
  화학물질 표시 기준(`chemical`)의 50번째 파일에는 정본 `dotumche.ttf` 또는
  명시된 해시의 `DotumChe-Regular.ttf`를 공급할 수 있다.
  종합부동산세법(`issue7905`)에는 Windows 10의 Arial·Courier New·Times New Roman 각 4종과
  `gulim.ttc`의 검증된 해시를 허용한다. `sample10`의 Native 공급 목록에도 같은 해시를 허용한다.
  `yeongwol`과 `sample16` 저장본 3종은 설치본 `HYHWPEQ.TTF`(`a3a3cd992a89…`)를 기준으로 사용한다.
  이전 해시 `c4f768c176ce…`는 같은 문서의 검증된 공급 파일에서만 허용한다. 수식 글꼴은 다운로드
  폴더에서 직접 공급하지 않고 사용자·시스템 설치 경로나 해시를 확인한 검증용 사본을 사용한다.
- 시행령(`decree`)과 중첩 표(`nested`)의 Native·WASM은 Mac 사용자 설치본과 같은
  Palatino Linotype Regular(`pala.ttf`, SHA-256 `e47d594e2e2b…`)를 공급한다.
  시행령의 휴먼고딕 `HMKMG.TTF`와 휴먼명조 `HMKMM.TTF`는 각각 사용자 글꼴 폴더의
  `HMKMG_MAC.TTF`, `HMKMM_MAC.TTF`도 공급할 수 있다. 문서별 허용 SHA-256을 확인한다.
  Mac 설치본을 공급한 Native·fresh WASM의 74쪽 표시값이 기존 공급 조건과 모두 같았다.
- Mac용 `HMK*_MAC.TTF`는 Native·fresh WASM의 전쪽 표시값이 모두 일치한 문서의
  `accepted_sha256_pairs`에 등록한다. Mac에서는 해당 파일을 사용자 글꼴 폴더에서 직접
  공급한다. Windows TTF에 대응하는 Mac 파일은 시행령, 화학물질 표시 기준, 영월, 개요 번호,
  sample16 저장본 3종 및 행정업무운영 편람 HWP·HWPX에서 검증했다. 두 경로의 2,296쪽
  표시값이 각각 기존 기준과 일치했다. 영월과 sample16 저장본 3종은 Mac 파일로 해당 OTF까지
  함께 대체한 392쪽의 값이 모두 일치해 그 허용 쌍도 등록했다. 중첩 표의 휴먼고딕은
  `HMKMG.otf`를 유지한다.
- 개요 번호(`outline`)와 종합부동산세법(`issue7905`)은 매니페스트에 기록한 OTF 글꼴을 사용한다.
  개요 번호의 OTF 33종과 법령의 OTF 12종은 각 문서의 `accepted_sha256_pairs`에 원래 지정한 해시,
  로컬 소형 OTF의 해시, 로컬 원본 크기 OTF의 해시를 파일별로 등록했다. 지정 순서로 공급하고 같은
  공급본에서 기준과 후보를 비교한다. Native에서는 두 로컬 공급본을 각각 검증했다. 브라우저에서는
  단일 글꼴 32 MiB·등록 요청당 64 MiB 한도를 충족하는 소형 공급본으로 전쪽을 검증했다.
- 개요 탐색 예제 두 문서(`outline-demo`, `outline-minimal`)는 함초롬바탕·함초롬돋움 TTF
  4종과 `KHANB.otf`, `KHANENGB.otf`, `KHANENGL.otf`, `KHANENGM.otf`,
  `KHANL.otf`, `KHANM.otf`를 매니페스트 순서로 공급한다.
  공급 목록에서 `Konghan-HFT.otf`를 제외했다. 공한 OTF는 macOS 설치본 해시를 기본으로 지정하고 기존 배포본 해시도 허용한다.
  두 해시의 파일을 각각 공급한 결과, 두 문서의 Native·fresh WASM 8행이 모두 100.00%였고
  devel·통합 후보의 페이지별 PNG 해시도 일치했다.
- 정본 보호에서 `HBATANG.TTF`는 한컴오피스 2020의 한컴바탕 1.21
  (SHA-256 `35f84328500fc2c3ee0b148aa75de0ed384bf9eee8dee9148c01b0a11a27fe05`)을 공급한다.
  이 파일로 바꾼 중첩 표(`nested`) Native·WASM 34행은 기존 공급본과 소수 둘째 자리
  실루엣 일치율이 모두 같았다. 최저값은 Native 99.64%, WASM 100.00%이며, 같은 글꼴을
  공급한 devel·통합 후보의 페이지별 PNG 해시도 일치했다.
- 중첩 표의 Native·WASM은 21개 파일을 매니페스트 순서로 공급한다.
  `H2MJRE.TTF`, `HDOTUM.TTF`, `gulim.TTC`, `pala.ttf`에는 문서별 해시 쌍을 허용한다.
- `font_environment`가 있으면 해당 JSON을 `--font-environment`로 지정한다.
- 선택 우선순위는 manifest의 `selection_order`를 따른다.
- `accepted_font_set`, `accepted_font_environment`와 `records/`는 수용 당시의 비교 근거이며, 현재 전쪽 재검증 상태는
  `reproduction_validation`에 기록한다.

대체 해시 등록과 검증 산출물 보존에 관한 다음 절차는 정본 보호 문서에만 적용한다.
대체 글꼴 해시를 허용할 때는 `manifest.json`의 해당 보호 문서 항목에 `accepted_sha256_pairs`를 등록한다.
허용 여부를 판단하기 위한 상세 비교 JSON과 실행 로그는 `output/pr-review/<작업명>/`에만 보존한다.
`records/`에는 해당 실행 산출물을 추가하지 않는다.

`environments/`는 실행에 사용할 환경 JSON, `imports.json`은 이전 기록에서 이관한 파일의
원래 경로·SHA-256과 공식 사본의 경로·`official_sha256`이다. 글꼴 프로그램 바이트는 포함하지 않는다.

저장소 루트에서 다음과 같이 문서가 지정한 SHA-256과 설치 파일을 대조할 수 있다.
검사는 허용된 파일의 실제 SHA-256을 출력한다. 아래는 시행령 문서의 HY신명조를 확인하는 예다.

```bash
python3 scripts/check_protected_font.py 49dd1ef0caa6989a7dd95b484fdd052cbc846f6514c0c08421ed03a6cb1669cc /mnt/c/Windows/Fonts/H2MJSM.TTF
```

문서 한정 허용 쌍을 확인할 때는 문서 key를 지정한다.

```bash
python3 scripts/check_protected_font.py --document chemical 097a83c15227aa0b01bba1c446d2cf67ac04694052e60d41fef557ed5a0a8a62 /path/to/DotumChe-Regular.ttf
```

적용·판정 절차는 [Visual Sweep 가이드](../../manual/verification/visual_sweep_guide.md#정본-보호-문서의-글꼴-공급-재현)를 따른다.
