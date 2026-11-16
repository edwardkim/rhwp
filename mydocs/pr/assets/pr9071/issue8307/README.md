# #8307 원 신고 정본 대조

[#8307](https://github.com/edwardkim/rhwp/issues/8307)의 공개 ZIP에서 정상 한컴 2018 HWPX를 확보했다. HWP는 기존 #8318 검증 원본과 SHA-256이 같아 `samples/pr9071/issue8318/서식2-간단.hwp`를 재사용한다. 원 PDF의 Creator는 한컴오피스 한글 2018, Producer는 Hancom PDF 1.3.0.515, 생성 시각은 2026-10-27 14:17:42 KST다. [출처·원본/PDF 해시·실제 결과](provenance.json).

최종 제품 source `eeaecff59e`의 Native/fresh WASM에서 원 HWP와 정상 HWPX를 전 3쪽씩 비교했다. 두 경로 각각 6쪽·전쪽 최저 raw 100.00%이며 PDF와 쪽수도 같다. compat 2022·96dpi, 색상 경계 보정 전 2px 내용 실루엣이다. 실제 2018 Print와 제품의 비교이며 2018 프로그램을 새로 실행했다는 주장은 아니다. 소스 생성 방식·저장 LineSeg를 바꾸지 않았다.

[Native HWP](native/hwp-source.tsv) · [Native HWPX](native/hwpx-source.tsv) · [WASM HWP](wasm/hwp-source.tsv) · [WASM HWPX](wasm/hwpx-source.tsv).

2쪽 표 외곽·뒤 본문이 없는 영역, 3쪽의 두 후속 문단·순서·간격을 독립 PDF와 직접 판독했다. [Native 2쪽](native/hwpx-source-page2.png) · [Native 3쪽](native/hwpx-source-page3.png) · [WASM 2쪽](wasm/hwpx-source-page2.png) · [WASM 3쪽](wasm/hwpx-source-page3.png).

기존 #8307의 원점·간격 관측은 이번 공통 TAC 보완의 독립 근거와 일치한다. HWP5 원본과 HWP5 출신 HWPX를 보완하고, 이미 정상이던 순수 HWPX의 3쪽 배치는 유지한다. 이 근거를 확인한 뒤 기존 관계형 회귀에 정상 HWPX 대조를 추가한다. 기준값이나 픽셀 기대값을 낮추지 않는다.
