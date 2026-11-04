# PR #8366 검토 — fix(renderer): 입체선 글자·문단 테두리는 긋지 않는다

## 최종 판정

**메인터너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 원 기여 commit을 보존한 누적 후보에서 기능·전쪽 Native/fresh WASM·필수 회귀·lint를 완료했다. 원 PR의 오래된 head를 직접 강제 merge하는 판정이 아니며, 통합 후보의 원격 처리 승인과 해당 head CI를 선행한다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8366, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `5993e4d32e08cce8785e79364dd6ac66eab80dfb`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:27:05.379731+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: Build & Test: failure, build-test-archive-c / Build test archive (c): cancelled.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 글자·문단과 칸·쪽 테두리의 구분

공통 `text_border_is_drawn`을 글자와 문단 테두리의 실제 배치 분기가 소비한다. line type 14–17의 입체선은 Mac Print와 같이 글자·문단에서 표시하지 않으며 칸·쪽 테두리의 입체선 계약을 변경하지 않는다. 비교 문서 1쪽의 Native/fresh WASM raw는 95.61/96.79%이고, 대상 14–17 영역은 직접 판독했다.

동일 문서의 일반 둥근 점선·물결선에 남은 차이는 이번 type 14–17 수정 전후 동일하며 기존 열린 #8368의 범위다. 이 11번째 PR을 이번 10건에 추가하거나 이번 작업으로 해결됐다고 주장하지 않는다. Archive C 실패는 checkout의 30분 시간 초과여서 테스트는 시작되지 않았다. 최신 통합 후보 검증으로 처리하고 기여자 코드 오류 댓글을 요청하지 않는다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·새 통합 PR·merge·후속 댓글은 현재 아직 실행하지 않았다. 승인 뒤 해당 제출 head의 CI와 최신 base의 충돌 여부를 다시 확인한다.
