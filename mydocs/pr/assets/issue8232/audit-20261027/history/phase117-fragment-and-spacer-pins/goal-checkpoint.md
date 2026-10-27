---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase117-fragment-and-spacer-pins/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase117-fragment-and-spacer-pins/goal-checkpoint.md` · 원문 SHA-256: `9a388aefdcdb9ebfac1fe871135a2543427d96b44de31a98b5f4423486010bfe`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 117차 기록

직접 검토는 11,439개 함수, Rust 테스트 6,985개 / 862개 파일입니다. 미검토 10,024개, 증거 대기 579개, 보정 확정 108개 함수 항목이며 전수 조사는 미완료입니다.

실물 표 조각·spacer 세 모듈 890줄과 34개 함수를 모두 읽었습니다. 새 29개와 기존 보류 5개를 판정했고, 보정 필요 테스트 5개를 추가 확인했습니다. 대상은 7062의 3쪽 상자·2~9쪽 기준선 핀, masked의 1쪽 셀 선택·행 좌표 핀, 음성 대조의 고정 y 구간 관측입니다.

현재 HEAD의 frozen Native로 canonical 192 DPI / PDFium / 2px 검증을 순차 수행했습니다. 7062 전10쪽 최저는 92.79138%이고 90% 미만은 없지만, 3~9쪽은 98% 미만입니다. masked 1쪽은 2020·2022 기준 PDF 모두 73.73956%, 2쪽은 99.99172%입니다. 두 PDF의 페이지 PNG는 바이트가 같습니다. 3조건 14쪽(중복 기준을 제외한 입력 12쪽)의 TSV·PNG를 보존했습니다. masked의 capture·TSV exit 1은 90% 미달 gate이며 크래시가 아닙니다.

7062의 3·10쪽과 masked 1쪽의 review·standalone overlay를 직접 판독했습니다. 7062의 글자폭·굵기·일부 기준선 차이와 masked의 PDF에 없는 회색 그림 자리표 등을 확인했습니다. 현재 WASM 시각 수용이나 새 제품 회귀라고 단정하지 않습니다. 기존 Native Rust 14개는 모두 PASS / exit 0입니다. #7097의 3개 root는 보유한 동일 HEAD target의 실행 근거가 없어 미실행으로 남겼으며 새 Cargo 빌드는 하지 않았습니다.

독립 원본 XML의 rowAddr 2는 저장 줄 끝 25,372 HU + padding 282 HU = 342.0533px이며, 전체 행 합 68,562 HU와 정합합니다. 이 높이 계약은 보존합니다. 7062의 16개 기준선 기대값은 독립 PDF의 문자 원점을 원본 용지 척도로 환산한 값과 최대 0.003784px로 맞습니다. 기대 좌표 자체의 오류와 98% 미만 페이지를 px golden으로 고정한 자격 위반을 구별했습니다.

독립 관측 스크립트에서 rounded 676px를 원시 50,700 HU로 가정한 자체 오류를 발견했습니다. 실제 50,698 HU를 확인해 관측만 정정했고 Rust·시각 검증을 재실행하지 않았습니다. 정정한 관측은 exit 0이며 이력을 보존했습니다. 제품·테스트 변경과 이름 변경은 아직 0입니다.

전수 조사 뒤 최종 이슈 갱신과 필요한 issue_8232 보정·명명·재검증을 계속합니다. 허용치 완화나 검사 제거로 미달을 숨기지 않고, 원본 상대 메트릭·내용 소유·독립 출력의 결함 검출을 보존합니다.
