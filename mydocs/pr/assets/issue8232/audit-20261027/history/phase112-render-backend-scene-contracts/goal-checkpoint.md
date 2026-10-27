---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase112-render-backend-scene-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase112-render-backend-scene-contracts/goal-checkpoint.md` · 원문 SHA-256: `2f7cf9aef45bf62e19d3a47303fb8763b2dfa21cc7fca90c7e499d4974e01031`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 112차 기록

직접 검토 11,249개 함수, Rust 테스트 6,883개/855개 파일이다. 미검토 10,128개, 증거 대기 567개, 보정 확정 103개 함수 항목으로 전수 조사는 미완료다.

M06-f 검사 10개 파일 2006줄·289개 함수를 읽고 기존 판정 159개를 보존했다. 남은 130개(테스트 126·helper 4)를 새 판정했다. 동일 HEAD source producer·실제 등록·binary SHA를 확인한 90개만 실행해 90 PASS / 0 FAIL / 0 IGNORE다. 나머지 36개는 source 판정이며 현재 CI 미등록으로 단정하지 않았다.

합성 SceneSpec JSON 196개의 schema/id/순서·양수 유한 크기·bbox와 재생 plane의 기대 trace를 Python 파서로 독립 재계산했다. 196개 모두 expectedTrace가 있고, generator의 순수 build_scenes 결과와도 같았다. generator main·write 함수를 실행하지 않았다. 67바이트 TINY_PNG는 세 chunk의 CRC와 Pillow verify가 정상이고 1×1 RGBA(0,0,0,0)다. 이는 유효한 투명 입력이며 가시 image ink의 검증은 아니다.

좌표의 두 자리 trace·합성 page size·opcode/개수·capability 광고·오류/생명주기는 실제 문서에서 채취한 px 핀이 아니었다. SVG markup 관측을 실제 glyph·가시 ink·전체 raster 수용으로 확대하지 않는다. PNG feature guard 내부 실행 여부도 별도 계측하지 않았다.

원 PR #5474는 직접 merge되지 않고 #5530에 통합됐음을 확인했다. 과거 7693 PASS나 CI를 현재 실행으로 쓰지 않았다. 추가 빌드·제품/테스트 변경·이름 변경은 0이며 기존 JS 변경은 보존했다. 미검토/보류 조사를 마친 뒤 최종 이슈 갱신·issue_8232 명명 보정·실제 재검증을 진행한다.
