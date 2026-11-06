# 보고서 수식·빈 꼬리 프레임의 R25 전쪽 검증

같은 원본과 기존 독립 한컴 Print PDF를 96 DPI·Print 프로필·compat2022의
색상 경계 보정 전2px 내용 실루엣으로 비교했다. 임계값·원본·PDF는 변경하지 않았다.

| 입력 | 독립 PDF 쪽수 | Native 전쪽 최저 | fresh WASM 전쪽 최저 | 전쪽 TSV |
| --- | ---: | ---: | ---: | --- |
| report-628 | 195 | 92.99% | 96.06% | [Native](report-628-native.tsv) · [WASM](report-628-wasm.tsv) |
| report-838 | 109 | 94.99% | 94.94% | [Native](report-838-native.tsv) · [WASM](report-838-wasm.tsv) |
| stored-band | 2 | 100.00% | 100.00% | [Native](stored-band-native.tsv) · [WASM](stored-band-wasm.tsv) |

모든 입력의 양쪽 쪽수가 독립 Print와 일치한다. 각306쪽/backend에서90%미만은 없고
R11 표시값 하락은0쪽이다. [행별 비교](whole-comparison.json),
[원본·PDF·실제 렌더 명령과 환경](input-provenance.json),
[Native producer](native-producer.json), [fresh WASM producer](wasm-producer.json)를 따른다.
Native 실행 파일 SHA-256은 `2a536938d8f6e3d3fecfcc9d10ef81b371ad71b24e2e263aba316cfbf8b86d3f`다.

이 폴더의18·122쪽은 수식의 한글·첨자와 셀 프레임을,123~125쪽은 뒤 표4-12~4-14·본문과
각주72~74의 소속·테두리·간격을 양쪽 review/overlay로 직접 대조한 증거다.
잉크 두께의 차이가 남아 있으며100% 동일 출력으로 주장하지 않는다.
R25 결과를 이후 제품 코드의 전쪽 통과로 자동 승격하지 않는다. 최종 후보 R26의 전쪽 비교는 별도다.

[브라우저 CanvasKit 기록](canvaskit-equation-consumer.json)과
[그 출력](canvaskit-equation-consumer-p018.png)은 별도 소비 경로 진단이다.
R26c fresh WASM의 실제 원본18쪽 수식 layoutBox를 실제 Chromium의
CanvasKitLayerRenderer software에서 SVG 우회 없이 재생했다.
4개 설치 프로그램을 실제 host 공급자로 준비해 수식1건의 layout replay 성공·실패0건을 확인했다.
좌표·기준선·기울임은 원본 producer가 만든 값을 유지했다. 글꼴 프로그램은 배포하지 않는다.
이 진단 PNG를 전체 Print 대조 또는 Canvas2D 화면 캡처로 표시하지 않는다.
