# 문서별 검증 글꼴 구성 보정

검증일: 2026-11-05 KST. 제품 source `6337fe4d10`은 변경하지 않았다.

초기 공통 10개 글꼴은 108.90 MiB였다. Native의 `TypefaceCatalog::load`는 파일 32 MiB·입력 합계 및 retained copy 64 MiB 한도를 적용하므로 목록 마지막의 HCR 바탕을 수용하지 못했다. 초기 TSV는 최종 증거에서 제외하고, **문서별 한도 내 글꼴 구성으로 base와 후보 Native/fresh WASM을 모두 재출력**했다. 한도를 늘리거나 없는 글꼴의 예외를 승인한 것이 아니다.

- HCR 바탕만 필요한 문서는 원본 `HANBatang.ttf` 27.10 MiB, 바탕쪽 문서는 여기에 `HANDotum.ttf`를 더한 48.32 MiB를 사용한다.
- PDF 명시 TextPaint 대조군은 저장소의 Noto Sans KR을 공급한다. 이미지 대조군은 custom 글꼴이 필요하지 않다.
- 여러 글꼴이 함께 있는 표 문서는 HCR 바탕·한컴바탕·바탕의 **검증용 로컬 subset**과 HY신명조·HY헤드라인M·Palatino Linotype·맑은 고딕·휴먼명조 원본을 공급한다. 합계는 22.79 MiB다. 원본과 subset은 공개 저장소·배포물에 넣지 않는다.
- subset은 해당 표 문서의 실제 Render Tree 문자열에 포함된 문자 합집합(ASCII 포함 390개)을 사용한다. FontTools에서 `retain_gids`, 전체 layout feature·name 보존, `recalcBBoxes=False`, `recalcTimestamp=False`를 적용한다. 원본에 존재하는 대상 codepoint마다 분해된 contour 명령과 hmtx, unitsPerEm의 불변을 실제 assertion으로 확인했다. 다른 문서의 일반 배포 글꼴로 사용하지 않는다.
- 독립 outline PDF의 내장 HCR 바탕 부분 글꼴과 로컬 원본의 `가` contour가 같음을 확인했다. Native에 HCR 바탕만 명시한 600 DPI 확대 출력에서 여섯 outline 종류와 glyph 형태를 대조했다. 600 DPI 이미지는 미세 획 판독의 보조 자료이며 96 DPI 전쪽 gate를 대체하지 않는다.

전체 경로·원본/로컬 subset SHA-256·입력 bytes·requested family·원본 face index·outline/metric 검사 수는 [font-profiles.json](validation/font-profiles.json)에 있다. subset 파일의 SHA는 로컬 검증 재현 식별자이며 재배포 권한을 뜻하지 않는다. 편람 383쪽은 별도의 기존 고정 KoPub/HCR 환경으로 검증했으므로 이 공통 10개 글꼴 오류의 최종 재측정 대상이 아니다.


## PR #8622 보존 검증의 80개 문서 구성

새 base `6e1de406209ba9a58032f6ca6de45b18aa687d1c`의 실제 WASM paint에서 각 입력/compat의 요청 family·codepoint·glyph ID를 수집했다. Fontconfig의 fc-match 성공만으로 Native가 해당 face를 사용했다고 판단하지 않았다. 초기 시스템-only Native 출력은 휴먼명조를 sans로 바꿔 87.41%였으며, 같은 문서에 실제 font program을 공급한 뒤 99.96%로 회복했다. 이 초기 결과는 최종 보존 gate에서 제외했다.

[80개 profile 원본/공급 파일·SHA·family·subset 불변 검사](validation/inherited8622/font-profiles.json)를 보존한다. 최대 합계는 56.37 MiB이며 파일당 32 MiB·합계 64 MiB 한도 안이다. 43개 profile은 실제 필요 문자/glyph ID로 만든 로컬 TTF subset을 사용한다. contours·hmtx·unitsPerEm을 원본과 대조했다. TTC face는 독립 TTF로 포장하며, CFF/로컬 HFT 변환의 영문·한글 family 분할은 원 program을 유지한다. 일부 원본의 알 수 없는 부가 table은 FontTools 경고와 함께 제외되므로 모든 shaping table bytes가 같다고 주장하지 않는다. 실제 406쪽의 결과로 표시를 별도 확인했다.

Native는 첫 공급 경로가 한 family의 style 집합을 소유하므로, 문서별 하나의 디렉터리에 필요한 style/영문·한글 partition을 모아 단일 `--font-path`로 공급했다. 이 구조와 정확한 legacy HY그래픽 원본 family 선택을 유지한다. 모든 font program과 subset은 로컬 검증 전용이며 저장소나 배포물에 넣지 않는다.


재생성에 필요한 [문서별 실제 family·codepoint·glyph ID](validation/inherited8622/font-requests.json)도 font bytes 없이 보존했다. 로컬 원본은 profile의 original SHA와 face index로 확인한다. subset의 입력은 codepoints와 ASCII U+0020–007E의 합집합 및 해당 font glyph order 범위 안의 glyphIds다. FontTools `subset.Options`에서 `retain_gids=True`, `name_IDs=['*']`, `name_languages=['*']`, `name_legacy=True`, `layout_features=['*']`, `notdef_outline=True`, `recalc_timestamp=False`를 적용하고 TTFont는 `recalcBBoxes=False`, `recalcTimestamp=False`로 읽는다. 공급 합계가 60 MiB를 넘거나 단일 program이 32 MiB를 넘을 때 4 MiB 이상의 TrueType program에 적용했다. CFF는 subset하지 않는다. 생성 뒤 profile의 SHA와 위 contour/hmtx/unitsPerEm 불변 검사를 확인하고 문서별 단일 디렉터리로 공급한다.
