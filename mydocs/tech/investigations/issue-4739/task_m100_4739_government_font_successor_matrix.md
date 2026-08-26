---
kind: investigation
status: active
canonical: mydocs/tech/font_fallback_strategy.md
last_verified: 2026-08-15
---

# #4739 정부상징 폰트 후계·대체 매트릭스

## 1. 결론

`정부상징 부처명_16040911`과 `ROKG_R`은 이름만 다른 동일 바이너리가 아니다. 그러나 동일한
힌팅 프로그램, 같은 Panose·weight·embedding 설정, 거의 같은 공통 글자 폭과 높은 글립 유사도는
`ROKG_R`이 구형 부처명 face와 조판 호환성을 의도한 현재 배포 face라는 강한 증거다.

따라서 향후 폴백은 단순 alias가 아니라 다음 **가용성·provenance별 선택 사다리**로 설계한다.

1. 문서가 선언한 구형 face가 설치돼 있으면 그 exact face
2. 구형 face가 없고 현재 공식 face가 설치돼 있으면 명시적으로 큐레이션한 `ROKG` successor
3. 둘 다 없으면 문서가 선언한 `substFont`인 `한컴바탕`
4. 그마저 없으면 rhwp의 portable open-source fallback

이 순서는 구현 승인 전 조사 결론이다. `ROKG`를 모든 코드 포인트와 세로 메트릭까지 구형 face의
전역 별칭으로 합치거나, 한컴 missing-font PDF를 exact-font 정답지로 간주하지 않는다.

## 2. 입력과 provenance

폰트와 공식 배포 파일은 저장소 밖 `/home/edward/mygithub/ttfs/gov`에 보존한다. 바이너리는 rhwp
저장소·PR에 추가하거나 재배포하지 않는다.

| 파일 | 크기 | SHA-256 | 역할 |
| --- | ---: | --- | --- |
| `정부상징 부처명_16040911.ttf` | 226,636 | `9ff914274d89c97abe3c22934c1f5f049d5c82de3cf0a3bc6053ac139b8a111a` | HWPX 선언 face와 같은 이름의 구형 비교 자산 |
| `ROKG_R.ttf` | 2,027,152 | `849c61ec05c9b468266a6ee3e7020ddc7c1696c9b3b29469b4986cab5e243a50` | 현재 공식 배포 filename과 일치하는 비교 자산 |
| `정부상징체-official-mcst.zip` | 2,208,353 | `594ca3048a20788eb94afa65bde06da99f56bc02731b49ae4e1acc3b9ffd2cef` | 문화체육관광부 페이지에서 받은 공식 배포 ZIP |
| `대한민국정부상징체-Installer-Windows.exe` | 2,997,366 | `93a56bccf44837a69b43a37596d84e77c2292d0772bd4e4eb71a365e7e742e4a` | 공식 ZIP 안의 Windows 설치 프로그램 |

공식 ZIP에는 InstallShield 설치 프로그램 하나가 들어 있다. 설치 프로그램 문자열과 설치 대상
정보에는 `ROKG_R.ttf`, Windows Fonts 디렉터리와 Fonts registry가 나타난다. 즉 현재 공식 배포가
`ROKG_R.ttf`를 설치한다는 근거는 확보했다. 다만 설치 프로그램 내부 압축 payload를 추출하지
못했으므로, 로컬 `ROKG_R.ttf`와 공식 payload의 byte hash가 같다고 주장하지 않는다.

## 3. 공식 사용 범위

- 문화체육관광부의 [정부상징 소개·서체 배포 페이지](https://www.mcst.go.kr/site/s_about/intro/symbol.jsp)는
  정부상징 전용서체를 훈민정음 창제기의 글꼴을 현대적으로 재해석한 제목용 서체로 소개하고
  다운로드를 제공한다. 페이지의 이용 조건도 함께 확인해야 하며, 바이너리 재배포는 이 조사
  범위가 아니다.
- [정부상징 디자인 지침서 2017](https://www.dapa.go.kr/dapa/files/05/%EC%A0%95%EB%B6%80%EC%83%81%EC%A7%95_%EB%94%94%EC%9E%90%EC%9D%B8_%EC%A7%80%EC%B9%A8%EC%84%9C_2017.pdf)는
  `대한민국정부상징체 R`을 중앙행정기관·소속기관 명칭 표기에 사용하고 일반 제목·본문에는
  사용하지 않도록 규정한다.

따라서 successor 규칙은 정부상징·기관명 face의 명시적 이름 집합에만 적용한다. “제목용”이라는
인상이나 획 모양만으로 일반 문서 글꼴을 ROKG에 보내지 않는다.

## 4. TTF 구조 비교

### 4.1 이름·범위·테이블

| 항목 | `정부상징 부처명_16040911` | `ROKG_R` |
| --- | --- | --- |
| family/full name | `Government_16040911`, `정부상징 부처명_16040911` | `ROKG`, `대한민국정부상징체`; `ROKG R`, `대한민국정부상징체 R` |
| PostScript / version | `Government_16040911` / `1.00` | `ROKGR` / `1.0.0` |
| UPEM | 1,024 | 1,000 |
| glyph / cmap | 1,808 / 1,805 | 18,794 / 18,791 |
| 현대 한글 음절 cmap | 1,698 | 11,172 전부 |
| 한자 / PUA / 호환 자모 | 없음 / 없음 / 없음 | 4,621 / 1,238 / 94 |
| 일본어 | 없음 | 히라가나 83, 가타카나 86 |
| 세로 메트릭 테이블 | `vhea`, `vmtx` 있음 | 없음 |
| vendor / 저작권 | `SJFG`, 일반적인 sparse metadata | `Typo`, 2016 대한민국정부·TypoDesign lab 표기 |

구형 cmap 1,805자는 전부 ROKG cmap에도 존재하며 ROKG에만 16,986자가 더 있다. `cvt `, `fpgm`,
`prep`, `gasp` 테이블 checksum은 두 파일이 같다. Panose `(2,2,5,3,2,1,1,2,1,1)`, weight 400,
width class 5, `fsSelection=64`, `fsType=4`도 같다.

### 4.2 폭과 커닝

공통 1,805자의 UPEM 정규화 advance 가운데 1,804자는 차이가 `0.001em` 이하다. 공통 현대 한글
1,698자는 모두 구형 `934/1024 = 0.912109375em`, ROKG `912/1000 = 0.912em`이다.

유일한 큰 예외는 `U+3000 IDEOGRAPHIC SPACE`다.

| face | U+3000 advance |
| --- | ---: |
| 구형 | `307/1024 = 0.2998046875em` |
| ROKG | `912/1000 = 0.912em` |

구형 653개, ROKG 648개 kerning pair 중 정상 공통 648개는 모두 `0.001em` 이내다. 구형에만 있는
5개 pair는 `glyph65535` artifact를 포함한다. 이 결과는 한글 행폭 호환성을 지지하지만 U+3000을
포함한 모든 메트릭의 전역 alias를 허용하지 않는다.

### 4.3 글립과 coverage

정규화 outline hash가 완전히 같은 공통 글자는 3/1,805뿐이므로 bit-exact 동일 폰트가 아니다.
다만 192px raster IoU는 1,743자가 0.95 이상이고 1,751자가 0.90 이상이다. 나머지 54자는 구형 cmap에
매핑돼 있지만 outline이 비어 있고 ROKG에는 실제 글립이 있다.

구형에서 비어 있는 현대 한글 54자는 다음과 같다.

```text
랏 랫 럇 럿 렇 렉 렌 렘 렙 렛 렝 렷 롄 롑 롓 릿 찻 챗 첫 쳇 칫
탓 탯 텃 텍 텐 템 텝 텟 텡 톈 팃 할 핥 핫 핸 핼 햇 헐 헒 헛 헥
헨 헬 헴 헵 헷 혈 혓 혠 혤 혭 힐 힛
```

따라서 ROKG는 구형 coverage를 단순 복제한 것이 아니라 빈 매핑을 보완하고 한자·기호·다국어
범위를 크게 확장한 face다.

### 4.4 대표 문자열

HWPX의 실제 표장 문자열 `행정안전부장관` 7자는 모두 contour topology가 같고, advance는 구형
`0.912109375em`, ROKG `0.912em`이다. 192px raster IoU는 다음과 같다.

| 글자 | 행 | 정 | 안 | 전 | 부 | 장 | 관 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| IoU | 0.984431 | 0.996299 | 0.996408 | 0.995486 | 0.987248 | 0.991747 | 0.979030 |

이 표장은 ROKG successor paint가 구형 exact paint에 매우 가깝다는 직접 증거다. 그러나 outline
hash는 0/7 exact이므로 시각적 호환과 동일 바이너리를 구분한다.

### 4.5 세로 메트릭 경계

| 정규화 값 | 구형 | ROKG |
| --- | ---: | ---: |
| hhea ascender | 0.859375 | 0.880 |
| hhea descender | -0.205078 | -0.305 |
| OS/2 typo ascender | 0.858398 | 0.858 |
| OS/2 typo descender | -0.141602 | -0.142 |
| win descent | 0.205078 | 0.305 |

typographic ascender/descender는 가깝지만 hhea·win descent와 font bbox는 다르다. 구형 문서의
행높이를 ROKG의 세로 메트릭으로 무조건 재계산하면 horizontal compatibility와 별개로 조판이
변할 수 있다.

## 5. HWPX와 한컴 PDF 오라클

`samples/2025 행정업무운영 편람(최종).hwpx`는 HANGUL face로
`정부상징 부처명_16040911`을 선언하고 embedded하지 않는다. 같은 FontFace의 `substFont`는
`한컴바탕`이며, charPr 386의 실제 run은 인라인 정부상징 그림 뒤 `행정안전부장관`이다. 원문은
`ROKG` 이름을 선언하지 않는다.

대응하는 한글 2020 KoPub 설치 PDF 물리 145쪽과 한글 2010 KoPub 미설치 PDF 물리 147쪽은 이
문자열을 `Haansoft Batang` glyph로 subset embed한다. 두 PDF의 7글자 outline과 advance는 7/7
일치하고 모두 `1.0em`이다. 반면 구형 TTF의 7글자는 `0.912109em`이며 Haansoft glyph와 outline·
advance 모두 0/7 일치한다.

따라서 오라클은 다음처럼 분리한다.

| profile | face | 의미 |
| --- | --- | --- |
| source exact | `정부상징 부처명_16040911` | 원문이 선언한 face를 설치한 환경 |
| official successor | `ROKG` / `대한민국정부상징체 R` | 구형 face 부재, 현재 공식 face 설치 환경 |
| Hancom missing-font | `한컴바탕` / `Haansoft Batang` | HWPX substFont와 관측 PDF를 재현하는 환경 |
| portable | rhwp open-source chain | 위 face가 전부 없는 결정적 실행 환경 |

한컴 PDF는 missing-font 동작의 정답이지, source exact나 ROKG successor의 모양을 부정하는 정답이
아니다.

## 6. 현재 코드 접합면과 결손

- `src/renderer/style_resolver.rs:453-463`은 `FontFace.name`과 내장 치환표만 사용하고
  non-embedded `FontFace.subst_font`를 표시 폴백 체인으로 전달하지 않는다.
- `src/renderer/style_resolver.rs:677-715`의 TTF 치환에는 정부상징 이름·ROKG successor가 없다.
- `src/renderer/mod.rs:1227-1283`의 설치 face alias는 `한양중고딕` 계열만 다루며 정부상징
  successor가 없다. generic 체인은 정부상징 이름을 sans 계열로 분류한다.
- `rhwp-studio/src/core/font-substitution.ts:21-179`의 `SUBST_TABLES`에는 정부상징 규칙이 없다.
  `fontFamilyChainForDisplay()`는 `resolveFont()` 결과와 일반 OS chain만 조합한다
  (`rhwp-studio/src/core/font-substitution.ts:310-347`).
- `resolveLocalFont()`는 수집한 name alias의 exact match만 사용한다
  (`rhwp-studio/src/core/local-fonts.ts:854-868`). 따라서 설치된 `ROKG`가 구형 이름 요청에 자동
  매칭되지 않는다.
- `src/tools/font_metric_gen.rs:378-408`은 수집된 현대 한글 subset이 모두 같은 폭이면 그 폭을
  전체 현대 한글의 압축값으로 만들 수 있다. 구형의 빈 54자와 불완전 coverage를 별도 표시하지
  않으면 존재하지 않는 exact metric을 합성할 위험이 있다. U+3000은 별도 range에 들어가므로
  (`src/tools/font_metric_gen.rs:618-630`) origin별 차이를 보존해야 한다.

현재 Studio에서 구형 face는 없고 ROKG만 설치된 경우, 구형 이름의 exact local match가 실패하고
정부상징 치환도 없어 Malgun Gothic 중심의 generic sans chain으로 간다. 또한 source의 명시적
`한컴바탕` substFont도 우선순위에 들어가지 않는다.

## 7. 제안하는 폴백 계약

### 7.1 이름 집합

| 역할 | 인식할 이름 |
| --- | --- |
| legacy exact | `정부상징 부처명_16040911`, `Government_16040911` |
| current successor | `ROKG`, `ROKG R`, `대한민국정부상징체`, `대한민국정부상징체 R`, `ROKGR` |
| document substitute | HWP/HWPX `substFont.face`; 이 문서는 `한컴바탕` |

legacy와 successor는 같은 face key로 합치지 않는다. 해소 결과에는 선택 provenance를 남겨 paint와
layout profile이 같은 결정을 공유하게 한다.

### 7.2 순서와 조건

1. 요청 face의 exact family/full/PostScript name과 style을 찾는다.
2. 요청 이름이 위 legacy 집합에 속할 때만 current successor 집합을 순서대로 찾는다.
3. successor가 없을 때 문서가 선언한 `substFont.face`를 찾는다.
4. 마지막에만 generic portable chain을 붙인다.

ROKG의 glyph coverage가 더 넓다는 이유로 exact legacy가 설치된 상태에서 run 단위로 두 face를
섞지 않는다. 구형 cmap에 있으나 outline이 빈 54자는 별도의 missing-glyph 정책과 시각 판정이
필요하다.

### 7.3 metric guardrail

- 한글 advance 호환값은 legacy `0.912109375em`, ROKG `0.912em`을 profile별로 보존한다.
- U+3000은 legacy `0.2998046875em`, ROKG `0.912em`을 절대 합치지 않는다.
- legacy profile의 불완전 cmap을 전체 11,172자 coverage로 확장하지 않는다.
- ROKG hhea/win 세로 메트릭을 legacy 문서 행높이에 자동 적용하지 않는다.
- `한컴바탕` missing-font profile의 `1.0em`은 한컴 PDF provenance에 묶고 exact·ROKG profile에
  역적용하지 않는다.

## 8. 구현 전 RED 계약

메인테이너 승인 뒤 다음 실패를 먼저 고정한다.

1. legacy exact 설치 시 successor보다 exact face를 선택한다.
2. legacy가 없고 ROKG의 family/full/PostScript 이름 중 하나만 설치돼도 ROKG를 선택한다.
3. exact와 ROKG가 없을 때 source `substFont`를 generic보다 먼저 선택한다.
4. 정부상징 이름이 아닌 일반 제목·본문 face에는 ROKG successor 규칙을 적용하지 않는다.
5. `행정안전부장관`의 face provenance와 profile별 advance를 고정한다.
6. U+3000의 legacy/ROKG 폭 차이와 구형 빈 54자 coverage를 고정한다.
7. local-font snapshot이 첫 Canvas paint 전에 준비되고 새 감지 뒤에는 한 번만 repaint한다.

## 9. 한계와 다음 판정

- 구형 TTF의 공식 배포 provenance는 아직 확정하지 못했다. 파일명과 HWPX 선언의 일치만으로
  정부 공식 원본이라고 단정하지 않는다.
- 공식 설치 프로그램 payload를 추출하지 못해 로컬 `ROKG_R.ttf`와 공식 배포본의 byte identity는
  미확정이다.
- 글립 유사도와 폭 호환성은 successor 후보를 지지하지만 “공식적으로 구형 face를 이름만 바꾼
  것”이라는 발표를 찾은 것은 아니다.
- 최종 수용은 exact legacy, ROKG successor, Hancom missing-font 세 profile의 동일 표장 구간을
  실제 Canvas/PDF로 나란히 검증한 뒤 메인테이너가 판정한다.
