# Task #7056 구현 계획 — 작성 원칙과 review 판정 연결

- Issue: [#7056](https://github.com/edwardkim/rhwp/issues/7056)
- 수행 계획: [task_m100_7056.md](task_m100_7056.md)

## 파일별 변경

| 파일 | 변경 내용 |
| --- | --- |
| `AGENTS.md` | 조판 수정 공통 원칙을 구현 근거, 공통 결과, 증거·기준값, 완료 보고로 구체화하고 review 절차 연결 |
| `CLAUDE.md` | `@AGENTS.md` 유지, 중복 렌더링 지침을 정본 포인터로 정리하고 일반 파서·저장·검증 규칙 보존 |
| `mydocs/manual/pr_review_workflow.md` | 공통 계약에서 접수 단계의 원칙 준수 검토를 필수로 연결 |
| `mydocs/manual/pr_review/README.md` | 모든 기본 경로에 공통 준수 검토가 적용됨을 선택표 진입점에 명시 |
| `mydocs/manual/pr_review/intake_and_review.md` | 모든 정식 review의 적용 여부 판정, 적용 대상의 원칙별 코드·증거 표와 최종 판정 연결 |
| `mydocs/manual/pr_review/visual_fixture_evidence.md` | 줄 구성 사례의 합성/한컴 증거 구분, baseline·golden·래칫 변경의 독립 근거와 시각 판정 |
| `mydocs/manual/pr_review/local_validation.md` | baseline 검증 경로에서 증거·허용치 심사 연결; 기존 명령·검증 게이트 유지 |

## 설계 경계

- 구현 원칙은 `AGENTS.md`가 소유하고 자식 문서는 reviewer의 확인·기록 절차만 구체화한다.
- 관련 없는 parser/serializer·문서 PR에 중첩 표 4종 시각 검증을 일괄 요구하지 않는다.
- 중첩 표 줄 구성 변경에 저장 LineSeg와 재조판 경로의 적용 여부를 각각 기록한다.
- baseline 수치 증가 자체를 정상으로 승인하지 않는다. 기준 출력과 독립 근거를 확인하며
  기존 회귀를 숨기지 않는다는 로컬 검증 계약을 유지한다.
- 코드 검토상 우려, 필수 증거 부족, 실행으로 재현된 결함을 구분한다.
- 근거 부족을 일으킨 원칙과 해제 조건을 명시하되 기존 판정 세 종류와 원격 승인 경계를 유지한다.

## 검증

문서 이동·정본 연결 변경에 비례해 `git diff --check`, 변경 Markdown의 로컬 링크 검사,
문서 메타데이터 검사를 수행한다. 기존 오류가 있으면 기준선과 비교해 신규 오류를 구분한다.
추가한 anchor는 기존 링크 검사기가 검사하지 않으므로 제목과 직접 대조한다.
문서 정책에 의한 자동 테스트·CI·권한 변화는 추가하지 않는다. Rust/Cargo·WASM·시각 검증은 실행하지 않는다.
