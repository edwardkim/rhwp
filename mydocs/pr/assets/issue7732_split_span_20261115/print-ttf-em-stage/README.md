# 구형 Print의 실제 TTF 표시 em 단계 보정

이 문서는 보정 과정의 분석 기록이다. 보호 검증이나 PR 준비 완료 보고가 아니다.
쪽수·입력·독립 Print PDF·PDFium·96dpi·2px raw·정본 보호값을 변경하지 않았다.

입력은 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 기준은
`pdf/review-integration-20261026/issue2007_nested_cell_pagination_42065-2020.pdf`다.
문서 글꼴 환경은 devel의 `nested-symbol-slot-environment.json`을 그대로 사용한다.

## 원인과 공통 소비

독립 한컴 2020 Print 표본에서 8·10·11·13pt의 `Tf`가 각각
600dpi 장치 단위 67·83·92·108로 확인됐다. [단위 영수증](independent-unit-receipt.json)은
그 표본의 SHA·engine·제품을 기록한다. 대상 문서의 좌표·글리프를 추출해 이 규칙을 만든 것이 아니다.

공통 paint에서 실제 정적 TrueType 프로그램이 공급된 구형 Print의 표시 em만
`round(size_px × 600 / 96) × 96 / 600`으로 확정한다. 원문 크기·원점·전진·기준선은
보존하며 `paintStyle`과 Native/SVG/Canvas가 같은 표시 크기를 소비한다.
본문·밑줄·탭·조판부호의 정본 run 사본을 함께 갱신하고, 별도 투영을 가진 겹침·shaping은 유지한다.
2024·Screen·HQ·명시적 잉크 보존·첨자·글자 위치 투영·미공급/CFF/가변/색 글꼴은 이 적용 대상이 아니다.

새 `issue_7732_print_ttf_em_grid`는 MIT 합성 TTF만 공급하는 관계형 단위 검사다.
표시 em·원문 전진·기준선, 실제 문서의 출력 세대, 겹침·밑줄 정본 소비를 검사한다.
첫 단위 검사는 보정 전 1 PASS/1 FAIL이었다. 개발 중 겹침 반례는 3 PASS/1 FAIL로
검출했고, 그 보완을 포함한 5건은 최소 글꼴 환경에서 모두 통과했다.
글자 위치 25%인 실제 문서의 별도 투영을 포함한 재실행도 5건 모두 통과했다.
정식 전체 게이트는 최종 source에서 다시 완료해야 한다.

## 전17쪽 결과와 남은 항목

보정 전 Native와 WASM은 parent `61b7950246a1524a449ed961a232e0fc426df3c3`이다.
보정 후 Native는 그 parent와 `em-grid-repair/v2/producer.json`의 source 파일 SHA·diff SHA로
고정한 로컬 후보다. [보정 전 Native](before-native.tsv)·[보정 후 Native](after-native.tsv)는
실제 전17쪽 TSV 원문이다. [기존 WASM](before-wasm.tsv)은 em 코드 보정 전 결과이며 새 후보의 검증이 아니다.

| Native 쪽 | 보정 전 | em 보정 후 | 보호 하한 | 상태 |
| --- | ---: | ---: | ---: | --- |
| 1 | 98.67 | 98.62 | 98.63 | 미달 |
| 10 | 99.43 | 99.47 | 99.50 | 미달 |
| 11 | 99.63 | 99.68 | 99.67 | 복원 |
| 14 | 99.36 | 99.42 | 99.42 | 복원 |
| 15 | 99.62 | 99.64 | 99.65 | 미달 |

1·10·15쪽의 미달은 면제하지 않는다. 같은 공급 TTF를 공통 decoder로 직접 재생한
추가 Native 대조는 이 세 쪽을 복구하지 못하고 1쪽을 98.61로 낮춰 채택하지 않았다.
해당 진단의 세 소스 파일은 되돌렸고 생성물과 로그는 로컬 `output/`에 보존했다.

## 글꼴 동일성의 범위

최신 기준 생산자의 `gulim.ttc` 파일 SHA와 현재 확보본의 파일 SHA는 다르다.
대상 PDF에서 사용한 굴림 366글리프를 현재 2.21 및 Windows 5.03 원본과 대조했을 때
윤곽·전진은 모두 같았다. 이 사실을 전체 파일이나 모든 글리프의 동일성으로 확대하지 않는다.

독립 표본에서 공급한 로컬 HFT 기호는 5쪽 `※`를 복원해 기존 Native/WASM 모두 100.00%가 됐다.
글꼴 바이너리·윤곽 명령·복원용 PDF는 Git에 넣지 않으며, 필수 검증에 필요한 로컬 공급과
원문 파일의 수정·재배포를 구분한다.
