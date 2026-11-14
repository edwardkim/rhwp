# PR #9070 괘선 보완 검증

원 기여의 7개 일반 기호와 잘린 교차 참조 방어를 보존한다. 77개 괘선은 독립 한컴 HWPX의 실제 저장 문자로 복원하며, 실제 공급한 SFNT를 공통 PUA 측정·paint 경로가 소비한다. 기존 11개 기호의 공급 순서와 공개 문자 대체는 유지한다.

## 입력과 독립 근거

- 원 #8912 합성 입력은 용지 크기 0·잘못된 글자 비율이므로 parser 코드 대조용으로만 보존한다. 이를 정상 Print 검증으로 사용하지 않는다.
- 정상 검증 입력: `samples/pr9070/hwp3-linechar-specimen.hwp`. 사양의 용지·12pt·100% 장평·160% 줄간격을 명시한 별도 합성 specimen이다. [출처](specimen-source.json).
- `samples/pr9070/hancom-reference/hwp3-linechars-symbols-2020.hwpx`의 [77개 실제 저장값](hancom-decoded-mapping.json)이 문자 기대값의 독립 근거다. 방향만 남기는 LibreOffice의 ASCII 단순화로 굵기·점선 종류를 추측하지 않는다.
- 기준: 메인테이너가 직접 인쇄한 `pdf/pr9070/hwp3-linechar-specimen-2020.pdf`, 한컴 2020 11.0.0.9166, 2026-11-14 18:30:56 KST, 전 1쪽. [수동 인쇄 출처와 해시](manual-print-provenance.json).

## 실제 글꼴과 측정 조건

PDF 리소스가 실제 `Haansoft Batang` TrueType을 포함한다. 공급한 기존 로컬 HBATANG.TTF 1.21의 hmtx와 PDF의 인접 글자 원점을 대조했다. 비공백 전진 차이는 최대 0.071pt이며 HCR Batang 표시 치환의 원 메트릭이 아니다. [프로그램·전진 근거](font-program-evidence.json).

따라서 이 독립 Print 비교에서만 [명시적 글꼴 환경](font-environment.json)의 자연 프로그램 전진을 사용한다. 제품 기본값·원본 저장 폭·IR·PDF를 변경하지 않았다. 글꼴 파일은 커밋하지 않는다. 같은 환경과 실제 프로그램을 Native/fresh WASM에 공급했다.

초기 HCR 프로그램 공급(85.22%/84.80%)과 Haansoft 표시용 치환의 원 메트릭 보존(81.63%/82.26%)은 실패한 진단이며 통과 증거로 쓰지 않는다.

## 전쪽 결과와 직접 판독

2026-11-14, 96dpi·compat 2022, Native Skia PNG / 최적화 fresh WASM Canvas PNG, 독립 PDF pdftoppm raster, 2px 내용 실루엣 raw:

| 경로 | 전체/비교 쪽수 | 전쪽 최저 | 원본 TSV | 직접 비교 |
| --- | --- | --- | --- | --- |
| Native | 1/1 | 98.56783% | [TSV](native/specimen_silhouette.tsv) | [review](native/specimen_review_001.png), [overlay](native/specimen_overlay_001.png) |
| fresh WASM | 1/1 | 98.55072% | [TSV](wasm/specimen_silhouette.tsv) | [review](wasm/specimen_review_001.png), [overlay](wasm/specimen_overlay_001.png) |

TSV는 canonical `--silhouette-only --png-pair`로 동일 raster에서 산출한 공식 두 자리 값이며, 위 세부 값은 같은 실행의 raw metric이다. 전쪽 90% 미만·누락·쪽수 불일치가 없다. 7종의 연결 방향·점선·선 굵기·이중선과 일반 기호 7개를 같은 배율로 직접 비교했다. 초기 후보의 `0x3032` 이중 실선 추정과 `0x305E` 가는 선 추정이 실제 점선·굵은 선으로 복원됐다. 좌표 핀은 추가하지 않았다.

실제 보정 바이너리·JS/WASM 해시 및 측정 manifest는 로컬 ignored 로그에 보존했다. 이 소규모 통과를 최종 head의 전체 회귀·보호 3,542쪽·편람 gate 통과로 확대하지 않는다. 해당 필수 검증은 계속 진행한다.
