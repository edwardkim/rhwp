# 문서별 검증 글꼴 구성 보정

검증일: 2026-11-05 KST. 제품 source `6337fe4d10`은 변경하지 않았다.

초기 공통 10개 글꼴은 108.90 MiB였다. Native의 `TypefaceCatalog::load`는 파일 32 MiB·입력 합계 및 retained copy 64 MiB 한도를 적용하므로 목록 마지막의 HCR 바탕을 수용하지 못했다. 초기 TSV는 최종 증거에서 제외하고, **문서별 한도 내 글꼴 구성으로 base와 후보 Native/fresh WASM을 모두 재출력**했다. 한도를 늘리거나 없는 글꼴의 예외를 승인한 것이 아니다.

- HCR 바탕만 필요한 문서는 원본 `HANBatang.ttf` 27.10 MiB, 바탕쪽 문서는 여기에 `HANDotum.ttf`를 더한 48.32 MiB를 사용한다.
- PDF 명시 TextPaint 대조군은 저장소의 Noto Sans KR을 공급한다. 이미지 대조군은 custom 글꼴이 필요하지 않다.
- 여러 글꼴이 함께 있는 표 문서는 HCR 바탕·한컴바탕·바탕의 **검증용 로컬 subset**과 HY신명조·HY헤드라인M·Palatino Linotype·맑은 고딕·휴먼명조 원본을 공급한다. 합계는 22.79 MiB다. 원본과 subset은 공개 저장소·배포물에 넣지 않는다.
- subset은 해당 표 문서의 실제 Render Tree 문자열에 포함된 문자 합집합(ASCII 포함 390개)을 사용한다. FontTools에서 `retain_gids`, 전체 layout feature·name 보존, `recalcBBoxes=False`, `recalcTimestamp=False`를 적용한다. 원본에 존재하는 대상 codepoint마다 분해된 contour 명령과 hmtx, unitsPerEm의 불변을 실제 assertion으로 확인했다. 다른 문서의 일반 배포 글꼴로 사용하지 않는다.
- 독립 outline PDF의 내장 HCR 바탕 부분 글꼴과 로컬 원본의 `가` contour가 같음을 확인했다. Native에 HCR 바탕만 명시한 600 DPI 확대 출력에서 여섯 outline 종류와 glyph 형태를 대조했다. 600 DPI 이미지는 미세 획 판독의 보조 자료이며 96 DPI 전쪽 gate를 대체하지 않는다.

전체 경로·원본/로컬 subset SHA-256·입력 bytes·requested family·원본 face index·outline/metric 검사 수는 [font-profiles.json](validation/font-profiles.json)에 있다. subset 파일의 SHA는 로컬 검증 재현 식별자이며 재배포 권한을 뜻하지 않는다. 편람 383쪽은 별도의 기존 고정 KoPub/HCR 환경으로 검증했으므로 이 공통 10개 글꼴 오류의 최종 재측정 대상이 아니다.
