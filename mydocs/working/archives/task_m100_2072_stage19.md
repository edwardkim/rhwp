# Task M100 #2072 Stage 19 - 부트스트랩과 memory 권위 충돌 제거

## 목표

최종 감사에서 확인된 `CLAUDE.md`, `manual/codex`, `manual/memory`의 과거 절차와 현재 canonical 문서 간
충돌을 제거한다. 과거 기록은 삭제하지 않고 historical archive로 보존한다.

## 변경 범위

- `CLAUDE.md`를 문서·CLI·Git 절차를 중복하지 않는 저장소 부트로더로 축소
- HWP3 파서 책임 경계를 `tech/parser_architecture.md`의 장기 기술 계약으로 이동
- rhwp-studio UI 명칭과 CSS 접두어를 별도 manual reference로 이동
- `manual/codex/README.md`를 현행 진입점으로 추가
- 종료 세션과 task memory를 `manual/codex/archive/`로 이동하고 historical metadata 부여
- 기존 `manual/codex/MEMORY.md`는 외부·개인 설정 호환을 위한 redirect로 유지
- `manual/memory/MEMORY.md`를 historical provenance 색인으로 명확화

## 검증 계획

- canonical 지도와 새 문서 링크 검사
- 문서 metadata 검사
- 활성 부트스트랩에서 개인 절대경로, `local/devel` 직접 push 절차, 종료 task 상태가 제거됐는지 확인
- archive 이동 전 경로에 대한 내부 참조가 남지 않았는지 확인
- `actionlint`와 `git diff --check`

## 결과

- `CLAUDE.md`에서 중복 CLI·Git·검증 절차와 과거 `local/devel` 직접 push 예제를 제거했다.
- HWP3 파서 경계와 rhwp-studio UI 명칭을 각각 canonical tech 문서와 manual reference로 보존했다.
- 활성 부트스트랩에서 개인 절대경로, 종료 task, `local/task*`·`local/devel` 절차가 검출되지 않았다.
- 과거 Codex memory 7개를 `archive/`로 이동하고 이전 내부 경로 참조가 0건임을 확인했다.
- 기본 링크 311개와 메타데이터 205개 검사가 통과했다.
- Python 구문 검사, `actionlint`, `git diff --check`가 통과했다.
