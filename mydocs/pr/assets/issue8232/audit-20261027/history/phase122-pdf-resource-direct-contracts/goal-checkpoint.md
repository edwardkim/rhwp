---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase122-pdf-resource-direct-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase122-pdf-resource-direct-contracts/goal-checkpoint.md` · 원문 SHA-256: `144fcc8b499bd1ef890aa6c70985068e69a7c76499f2590ff88bd216c7b80f0d`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 122차 기록

직접 검토는 11,591개 함수, Rust 테스트 7,045개 / 870개 파일입니다. 미검토 9,936개·증거 대기 581개이며 전수 조사는 미완료입니다. 보정 확정은 109개에서 111개 함수 항목으로 늘었습니다. 이번 두 항목은 같은 PDF 구조 검출 공백의 helper와 호출 검사이며 독립 결함 두 종류라는 뜻이 아닙니다.

Native direct PDF 전체 685줄·28개 함수와 자원 검사 21개 본문·helper 15개를 분석해 새 64개 함수를 판정했습니다. 현재 같은 HEAD의 정상 Rust 검사는 14 PASS / 0 FAIL / 0 ignore입니다. 자원 검사 21개는 보유 실행 target이 없어 새로 빌드하지 않았고 실행 통과로 세지 않았습니다.

assert_complete_pdf는 시작/끝 표시만 확인해 구조 없는 15바이트를 수용합니다. 공개 CLI 재매핑 환경 변수로 기존 caller 하나에 그 바이트를 공급하자 실제 기존 검사가 1 PASS였습니다. PDFium과 pypdf는 모두 거부했습니다. 현재 실제 Native CLI의 별도 PDF는 두 reader 모두 1쪽으로 읽었고 text 212자를 확인했으므로 제품 출력 결함과 검사 검출 공백을 구분합니다.

독립 한컴 Print 5개에서 위→아래 10행·11패치의 RGB 550개를 읽어 기존 색·끝값을 대조했습니다. 13개 JPEG의 원본 SOF 크기와 외부 Pillow decode도 확인했습니다. 이미지 자체 크기·codec bytes·glyph/Unicode 주소·RGB/alpha 채널 오차 및 직접 작성한 32×32 scene의 중앙 16,16은 실물 페이지의 잠정 원점 핀이 아닙니다. 정상 계약을 일괄 삭제하거나 ±2 CSS px로 바꾸지 않습니다.

원 PR #2372와 후속 색조/자원 PR #7913·#7970·#8120·#8191의 실제 merge 조상을 확인했습니다. 현재 #8232의 검사 방식 보정 범위를 읽었고 #8266은 inline-control-move의 별도 배치 이슈로 구분했습니다. 원본·사용자 JS·공유 target을 보존했고 추가 빌드·제품/테스트 수정·이름 변경은 0입니다. 새 full Visual Sweep이나 fresh WASM을 실행한 것으로 표시하지 않습니다.

전수 조사 후 최종 이슈 갱신과 필요한 보정·issue_8232 명명·재검증을 계속합니다.
