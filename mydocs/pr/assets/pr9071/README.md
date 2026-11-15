# PR #9071 바탕쪽·표 뒤 문단 검증

공개 원본과 정상 한컴 저장 대조군을 독립 한컴 2020 11.0.0.9136 one-up Print PDF에 대조했다. Windows 호스트의 Hwp.exe·COM·자동 인쇄는 실행하지 않았다. 원 기여의 표 속성·색 없음·바탕쪽 슬롯·각주 저장 계약을 보존하며 추가 발견한 배치 문제를 공통 로직에서 보완했다.

## 전쪽 결과

2026-11-14, compat 2022·96dpi·색상 경계 보정 전 2px 내용 실루엣 raw. [입력·기준 PDF 해시 및 전쪽 결과](focused-results.json). 각 입력은 독립 PDF와 3쪽이 일치한다.

| 입력 | Native 최저 | 최적화 fresh WASM 최저 | 원본 TSV |
| --- | --- | --- | --- |
| Both HWPX | 100.00% | 100.00% | [Native](native/both.tsv) · [WASM](wasm/both.tsv) |
| Both 실제 HWP 저장본 | 100.00% | 100.00% | [Native](native/both-hwp.tsv) · [WASM](wasm/both-hwp.tsv) |
| Even HWPX | 100.00% | 100.00% | [Native](native/even.tsv) · [WASM](wasm/even.tsv) |
| 대칭 여백 | 100.00% | 100.00% | [Native](native/symmetric.tsv) · [WASM](wasm/symmetric.tsv) |
| 반전 여백 | 100.00% | 100.00% | [Native](native/reverse.tsv) · [WASM](wasm/reverse.tsv) |
| 실제 사각형 한컴 재저장본 | 100.00% | 100.00% | [Native](native/rectangle.tsv) · [WASM](wasm/rectangle.tsv) |
| 실제 표 한컴 재저장본 | 100.00% | 100.00% | [Native](native/table.tsv) · [WASM](wasm/table.tsv) |
| 서식2-간단 원 HWP | 100.00% | 100.00% | [Native](native/color.tsv) · [WASM](wasm/color.tsv) |
| 서식2-간단 실제 HWPX 저장본 | 100.00% | 100.00% | [Native](native/color-hwpx.tsv) · [WASM](wasm/color-hwpx.tsv) |

합계는 경로별 27쪽이며 누락·90% 미달·쪽수 불일치가 없다. 새 회귀 등록 전 이 범위를 먼저 측정·직접 판독했다. [바탕쪽 그림/사각형/표의 짝수 쪽 비교](frame-contact.png), [표 뒤 본문의 3쪽 비교](color-contact.png), [3쪽 확대](color-page3-zoom.png)를 확인했다. 용지 경계에서의 막대 절단, 개체 너비·세로 위치, 본문 보존과 두 후속 문단의 순서·누락·중복을 대조했다. 점수만으로 알려진 차이를 면제하지 않았다.

## 독립 근거와 수정 전후

Both의 원 HWPX/HWP는 보완 전 짝수 쪽 Native 47.76%였다. 대칭 여백이면 이동하지 않고 반전 여백이면 반대 방향으로 이동한다. 실제 그림뿐 아니라 정상 한컴을 거친 사각형·표도 같은 프레임 계약을 확인했다. [원인·호출 경로](../../archives/pr_9071_review_impl.md), [한컴 대조군 job·출력 해시](hancom-object-controls.json), [입력 출처](../../../../samples/pr9071/master-frame-controls/README.md).

서식2-간단은 최신 devel과 보완 전 후보에서 Native 100.00%·84.89%·16.74%, WASM 100.00%·84.28%·16.64%였다. 원본 저장 줄이 TAC 표와 바깥 여백을 혼자 소유하고 후속 저장 축이 다시 시작하는 경우, 앞 줄이 소유한 후행 간격을 두 소비 경로가 함께 보존하도록 고쳤다. 두 후속 문단은 원본·실제 HWPX 저장본 모두 3쪽에 있고 원문·빈 문단을 보존한다. [원인·선행 해결 PR·공통 소비](../../archives/pr_9071_color_rendering_finding.md).

## 생산자와 글꼴

WASM은 최종 제품 source `eeaecff59e834f28437c98b509089bbb36a7682d`에서 Docker wasm-pack·실제 wasm-opt를 완료했다. [JS/WASM 해시](wasm-producer.json). Native도 같은 source에서 lib/bin을 함께 빌드했다. [실제 실행 파일 해시](native-producer.json). 회귀 검증 head `3b5e750d66`와 제품 경로의 Git diff는 0이다. 최종 집중 27쪽의 PNG가 앞서 직접 판독한 출력과 모두 같음은 [전쪽 비교](final-pixel-preservation.json)로 확인했다.

공급 글꼴은 로컬 Windows `HANBatang.ttf`이며 두 경로에 같은 프로그램을 공급했다. 글꼴 파일은 포함하지 않는다. 이 시각 비교와 상용 글꼴을 숨긴 최소 글꼴 Rust 회귀는 별도의 검증이다. 전체 바탕쪽 11조합·저장 재열기와 정본 보호 검증 결과는 최종 검토에 별도로 기록하며 이 27쪽 결과로 대신하지 않는다.


## 원 신고 #8307의 추가 확인

원 HWP와 정상 한컴 HWPX를 신고 당시 실제 한컴 2018 Print에 대조한 경로별 6쪽도 전쪽 100.00%다. 집중 검증 합계는 Native/fresh WASM 각각 33쪽이다. [원본 해시·2018 PDF 출처·전쪽 TSV·직접 판독](issue8307/README.md)을 보존하며 새 2018 앱 실행을 주장하지 않는다.

TAC의 초기 원점 판정은 최종 source에서 원점 0 재시작과 단순 양수 되감김을 구분하도록 보완했다. 기존 #8191/#7445 기대값은 유지했고 최종 전체 회귀 13,574개에서 해당 기존 검사와 새 검사 3개가 모두 PASS했다. 특별 보호값 재현은 [공통 원장](../jinho-von-choi-20261114/README.md)의 별도 보류 조건이다.
