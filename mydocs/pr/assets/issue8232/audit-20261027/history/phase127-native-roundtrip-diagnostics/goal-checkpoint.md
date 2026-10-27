---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase127-native-roundtrip-diagnostics/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase127-native-roundtrip-diagnostics/goal-checkpoint.md` · 원문 SHA-256: `c22e9d67b557dd2b8de4184e61887746d035bf2e9516b157da5f6c1eac7a3f5f`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 127차 기록

직접 검토는 11,893개 함수·helper, Rust 테스트 7,084개 / 870개 파일입니다. 미검토 9,680개·증거 대기 581개이며 전수 조사는 미완료입니다. 보정 필요 확정은 114개에서 115개 함수 항목으로 늘었습니다.

실물 표/저장 원 root 6개 1,059줄을 분석했고 각 root의 소유 cwd에 정확한 입력 사본과 output을 두어 실제 6 PASS를 확인했습니다. 원본 부재로 건너뛴 경우는 0입니다. 이 중 5개는 차이 출력용 진단이며 차이·본문 동등성을 단언하지 않으므로 완전한 회귀 보호로 부르지 않습니다. 나머지 재직렬화 검사는 문단 수 보존을 assert하지만 필수 입력 부재 시 SKIP 출력 뒤에도 1 PASS가 됐습니다. 이 입력 liveness 공백을 추가했습니다.

별도 CFB reader로 원본·동봉 참조3개와 실제 저장 산출물6개를 읽었습니다. 원본4×3표12셀, 참조와 두 병합 출력11셀 및 row/col/span 대응, raw 보존/제거 출력의 root 문단31개를 확인했습니다. 파일명·주석을 독립 한컴 Print 영수증으로 오인하지 않으며 모든 문단 내용·수식·그림·전쪽 시각 일치로 확대하지 않습니다. record index·tag·HU·byte 출력 한도는 실물 페이지 px 핀이 아닙니다.

같은 merge 파일명을 쓰는 원 root는 별도 output에서 순차 실행했습니다. repository output의 고정 4경로와 samples, 사용자 JS를 보존했습니다. Native build의 실제 2546c8b 출처·현재 head의 1,305개 컴파일 입력 동일성과 binary를 다시 검증해 재사용했고 새 빌드·fresh WASM·제품/테스트 수정·이름 변경은 0입니다.

재직렬화 원 함수의 최초 import f0f7f1a4b 원문에서 같은 누락 return과 문단 수 assertion을 확인했습니다. rustfmt line blame을 원 해결 PR로 오인하지 않았으며 import 이전 PR은 확인하지 못했습니다.

전수 조사 후 최종 이슈 갱신과 필요한 보정·issue_8232 명명·재검증을 계속합니다.
