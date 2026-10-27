---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase119-visual-sweep-png-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase119-visual-sweep-png-contracts/goal-checkpoint.md` · 원문 SHA-256: `abee8911a5ff9c88bec8a744bf82146c6fa5d0acf1f7c4c5a652ff425e283ed1`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 119차 기록

직접 분석은 11,482개 함수입니다. Rust 테스트는 6,987개 / 864개 파일이며, 이번에 Python 검사 28개와 helper 7개를 새로 분석했습니다. 미검토는 9,996개, 증거 대기는 580개입니다. 보정 확정은 108개에서 109개 함수 항목으로 늘었으며 전수 조사는 미완료입니다.

Visual Sweep PNG·글꼴 공급 검사 모듈 전체 621줄과 공급 구현 530줄, dispatch와 실제 CI 자동 수집 경로를 읽었습니다. 기존 Python 검사는 28 PASS / 0 FAIL / 0 skip, 1.903초였습니다. Rust·새 빌드·실제 문서 시각 캡처·fresh WASM은 실행하지 않았습니다.

새 문제는 test_extracted_face_preserves_glyph_tables_and_header_metadata의 누락 검출 공백입니다. 두 face가 있는 TTC를 입력하지만 catalogue의 face 개수를 확인하지 않아 빈 목록이면 비교 반복문 전체를 건너뜁니다. 실제 기존 검사 본문을 수정하지 않고 font_catalog 한 호출의 결과만 faces=[]로 바꾼 독립 대조군도 1 PASS였습니다. 제품이 실제로 빈 결과를 만든다는 주장은 아닙니다. 향후 보정은 원본의 face 개수·index 집합과 각 face의 유일성을 확인하고 기존 glyph/hmtx/head bytes 보존 검사를 유지하는 방식입니다.

두 원본의 세 face를 별도 원시 SFNT directory/checksum/hmtx 판독기로 검증하고 실제 준비 program과 테이블 bytes를 대조했습니다. 1,000 UPEM·advance·합성 outline 크기는 font 설계 단위이며, mock PNG 3×4 크기·3쪽 manifest는 입력 프로토콜입니다. 이를 실물 페이지 px 핀으로 잘못 제거하지 않습니다.

원 공급 보존 PR #7907과 후속 통합 #8191은 실제 merged 상태와 현재 HEAD 조상을 확인했습니다. 기록은 ignored output에만 추가했으며 제품·테스트 변경과 이름 변경은 0입니다. 기존 rhwp-studio/public/rhwp.js 변경은 보존했습니다. 감사 완료 뒤 최종 이슈 갱신과 필요한 보정·issue_8232 명명·재검증을 계속합니다.
