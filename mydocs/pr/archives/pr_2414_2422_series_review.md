# kevin9327 3차 연작 검토 — #2414/#2415/#2416+#2421/#2422 (Rust 코어 진입)

하루 3사이클째 — studio TS(1·2차 8건)에 이어 Rust 코어 5건. 전건 CI green +
로컬 일괄 검증(스위트/fmt/clippy) green.

| PR | 본질 | red→green |
|----|------|-----------|
| #2414 | lenient CFB 헤더 미검증 패닉 3종 (WASM 모듈 트랩 = 타 문서 소실) | 가드 제거 변형에서 주장 패닉 3종 그대로 재현 |
| #2415 | HWPX secPr grid/startNum/tabStop 템플릿 상수 고착 — 파서 교차 인용, HWP writer 대조로 "포맷 한계 아님" 확정, tabStop 조건 방어 | 치환 무력화 red → 5/5 |
| #2416+#2421 | **raw_data stale clone 3중 구조** (직렬화 raw_data 우선 + PartialEq 제외) — 화면 정상·저장 소실 | 무효화 줄 제거 red 3건 → 3/3. roundtrip baseline 무회귀 |
| #2422 | 셀 서식 리플로우가 페이지 폭 사용 — do/undo 비대칭, undo 형제 헬퍼로 정합 | red 이식 불가(모듈 import 경계) — 검증된 헬퍼 교체 + 스위트 무회귀로 갈음 (사유 명시) |

#2416/#2421 진단은 트러블슈팅 관례 문서로 등재
(troubleshootings/raw_data_stale_clone_on_mutation.md) — 지식 증류 기여 인정
코멘트 게시 (작업지시자 지시).

## #2420 (postmelee) — wasm-pack 0.15.0 pin (#2233, supersedes #2274)

composite action 단일 진실 + Dockerfile 동기 규칙 + 정책 문서. PR CI run 이
실기동 검증(설치 로그 0.15.0). 원작 #2274(planet6897) cherry-pick -x
provenance — merge 코멘트에 크레딧 명시. **merge 완료.**
