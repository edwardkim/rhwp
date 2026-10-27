---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase113-shaping-input-source-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase113-shaping-input-source-contracts/goal-checkpoint.md` · 원문 SHA-256: `82b6ab4a1250622f2bb084a10bd3c221939c985fb58e4b4898a266a787b18182`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 113차 기록

직접 검토 11,279개 함수, Rust 테스트 6,899개/855개 파일이다. 미검토 10,112개, 증거 대기 567개, 보정 확정 103개 함수 항목으로 전수 조사는 미완료다.

셰이핑 요청 모듈 2001줄·55개 함수를 읽고 기존 25개 판정을 보존했다. 남은 30개(테스트 16·helper 14)를 새 판정했다. Cargo에 별도 test target으로 등록된 것은 확인했지만 보유 감사 metadata 10개의 실행 근거에서 해당 target을 찾지 못해 Rust를 실행하지 않았다. CI 미등록으로 단정하지 않았고 새 빌드도 하지 않았다.

세 원본 font를 FontTools로 독립 파싱했다. 모두 UPEM1000·GSUB/GPOS가 있고 Noto/OldHangul은 vhea/vmtx, OldHangul은 VORG가 있다. 가변 font의 wght/opsz는 min/default400·max900이며 vhea/vmtx가 없다. 400/650/900은 fvar 축 단위, glyph ID/UTF8 cluster는 주소, bbox91/199와 origin100/200·size10·pitch12는 authored fallback/요청 입력으로 구분했다. 실제 문서에서 채취한 px 핀으로 취급하지 않았다.

red/absent라는 함수명만 보고 기능 부재를 고정한 검사로 오인하지 않았다. 실제 다섯 assertion은 모듈·owner·commit·publication·양쪽 feature 문자열의 존재를 요구한다. 이것은 source 표면 검사이며 실제 제품 동작이나 CanvasKit 실행 증거로 승격하지 않는다.

제품/테스트 변경·이름 변경·추가 빌드는 모두 0이고 기존 JS 변경을 보존했다. 전수 조사와 보류 증거 정리 후 최종 이슈 갱신·issue_8232 명명의 필요한 보정·실제 재검증을 진행한다.
