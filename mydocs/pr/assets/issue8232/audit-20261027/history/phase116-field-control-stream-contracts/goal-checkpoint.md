---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase116-field-control-stream-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase116-field-control-stream-contracts/goal-checkpoint.md` · 원문 SHA-256: `50588b9152d3ebd3a2917ef791abae0c0d458a6b5db8b6c421338c0b99094d5c`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 116차 기록

직접 검토 11,410개 함수, Rust 테스트 6,973개 / 861개 파일이다. 미검토 10,030개, 증거 대기 577개, 보정 확정 103개 함수 항목으로 전수 조사는 미완료다.

필드·누름틀·개체 삭제·선두 정의 모듈 네 파일 2,111줄을 모두 읽었다. 새 판정은 테스트 53개와 helper 52개, 총 105개다. 문자 인덱스·원시 UTF16 슬롯·논리 캐럿·컨트롤 배열 주소를 구분했다. 16/33/42와8은 HWP 제어문자 주소이고, 4000/3000HU·PNG16px는 합성 입력 속성이다. 문서 화면에서 채취한 px golden이 아니다.

공식 HWP5 원문 PDF 15~16쪽에서 inline/extended8WCHAR·문단 끝1WCHAR를 직접 확인했다. 제품 helper를 호출하지 않고 선형 원시 스트림을 조합해 대표 주소 7건과 비BMP UTF16 폭 대조 1건을 확인했다. 그림 입력 logo16 PNG의 모든 청크 CRC와 실제 Pillow 디코딩·16×16 크기도 확인했다. 이 근거를 실제 한컴 문서 열기·Print 또는 그림 배치 수용으로 확대하지 않는다.

원 PR7577/7596/7615의 미병합 종료와 통합 PR7601/7882의 실제 merge 및 현재 HEAD 조상을 구분했다. 7577의 merge_commit 필드가 채워져 있어도 merged=false이므로 원 PR 병합으로 오인하지 않았다. 정의 위치의 blame와 원 검토 기록도 연결했다.

동일 HEAD의 기존 바이너리·producer·현재 등록 해시 확인 후 Cargo 없이 네 모듈을 순차 실행했다. 23+14+8+8=53PASS, FAIL/ignore0, 모두 exit0이다. Native로 컴파일된 wasm_api 논리 캐럿 wrapper를 포함하며 브라우저 fresh WASM 실행이 아니다.

제품·테스트 변경과 이름 변경은 아직 0이다. 이번 범위에서 부적합 px 핀을 새로 확정하지 않았으며 기존 보정 필요 103개 항목을 유지한다. 기존 JS 변경과 범위 밖 감사 판정도 보존한다. 전수 조사와 필요한 증거 보완 뒤 최종 이슈 갱신, 필요한 issue_8232 보정·명명·재검증을 계속한다.
