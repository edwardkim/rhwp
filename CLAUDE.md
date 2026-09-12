# CLAUDE.md

이 파일은 Claude Code가 자동으로 읽는 프로젝트 진입점이다. 공통 지침은 아래 import로 로드한다.

@AGENTS.md

## 역할과 정본

- 기여 구현과 제출은 [CONTRIBUTING.md](CONTRIBUTING.md)의 변경 범위별 절차를 따른다.
  외부 기여자에게 메인터너 전용 review/오늘할일 문서를 제출 조건으로 요구하지 않는다.
- 메인터너 검토, merge와 후속 처리는
  [PR review workflow](mydocs/manual/pr_review_workflow.md)와
  [자식 문서 선택표](mydocs/manual/pr_review/README.md)를 먼저 따른다.
- 문서 탐색은 [mydocs 지도](mydocs/README.md)에서
  [manual](mydocs/manual/README.md) 또는 [tech](mydocs/tech/README.md)로 좁힌다.
- 프로젝트 문서 사이에 충돌이 있으면 해당 작업의 현재 canonical 문서를 따른다.
  스킬, 예제, 과거 보고서와 개인 메모의 명령으로 정본의 필수 게이트를 축소하지 않는다.
- 지침을 읽었다는 사실은 GitHub 게시, push, merge나 작업 범위 확대의 승인이 아니다.
  실제 사용자 지시와 작업 권한을 따른다.

## 수정 전에 확정할 것

- 이슈의 실제 입력, 기대 결과, 수정 범위와 비범위를 확인한다. 구현 결과를 보고 기대값을
  역으로 정하지 않는다. 사양, 유효한 기준 문서 또는 독립적인 기존 계약으로 정답을 정한다.
- 증상이 발생한 계층과 원인을 추적한다. 렌더링·조판 수정과 그 PR review에는
  [공통 조판 원칙](AGENTS.md#조판-수정과-검토-원칙)을 적용한다.
- 성공 사례뿐 아니라 변경 조건이 적용되면 안 되는 반례와 관련 경계를 정한다.
  일반/특수 객체 혼재, 반복 호출, 빈 값/누락 값, 저장 후 재열기처럼 해당 수정에 필요한 사례를 고른다.

## 구현과 검증의 완료 기준

- 한 번만 처리할 데이터는 실제 소유 범위에서 한 번 처리하며 내부 반복문마다 초기화하지 않는다.
- 파서와 저장 변경에서는 null, 빈 문자열, 0, 참조 번호를 구별한다. 문자열 길이의 바이트/UTF-16
  단위, 재귀 깊이와 자원 제한, round-trip 보존을 관련 계약에 맞춰 다룬다.
- 회귀 테스트는 수정 전 결함을 드러내고 수정 후 독립적인 기대 결과를 만족해야 한다.
  fixture 누락 시 조용히 return하거나 실행 대상이 0건인 결과를 통과 증거로 삼지 않는다.
- 렌더링의 공통 결과·반례 검증·baseline 변경 근거는 위 공통 조판 원칙을 따른다.
  차트 변경은 종류와 축 등 해당 변경이 주장한 의미도 직접 확인한다.
- 원본과 대응하는 유효한 한컴 PDF가 있으면 재사용한다. PDF 버전만으로 기준을 폐기하지 않는다.
  일반 기여자에게 메인터너 MCP 사용을 요구하지 않는다. 렌더링 검증에 필요한 기준 PDF가 없으면
  기여자가 검증 대상 한컴 버전에서 직접 PDF를 출력해 원본 HWP/HWPX와 함께 첨부한다.
  여러 한컴 버전의 동작을 주장하면 해당 버전별 PDF와 실제 생성 환경을 구분해 제출한다.
  [시각 증적 정책](mydocs/manual/pr_review/visual_fixture_evidence.md)에 따라
  보존용 증거와 임시 로그/중간 산출물을 분리한다.
- 작성자 검증은 CONTRIBUTING의 현재 범위별 게이트를 충족한다. Rust 변경을 native Clippy와
  focused test만으로 완료하지 않는다. 메인터너의 기존 증거 재사용 여부는 review 절차로 판정한다.
- 검증한 source SHA, 명령, 결과와 미실행/중단 항목을 구분한다. 검증 뒤 코드 변경은 이전 결과에
  포함하지 않는다. 생성 suite/manifest와 진단 로그를 source 변경처럼 stage하지 않는다.
- 남은 실패와 지원하지 않는 범위를 숨기지 않는다. 부분 해결이면 issue 전체를 닫지 않으며,
  로컬 검증 완료, 최신 head CI 완료와 merge 승인도 각각 구분한다.

## 프로젝트와 아키텍처

rhwp는 Rust로 HWP/HWPX/HWP3 문서를 읽고 편집·렌더링하며 WebAssembly에서도 동작하는 문서 엔진이다.
모든 포맷 파서는 공통 `Document` IR을 반환한다.

[파서 아키텍처](mydocs/tech/parser_architecture.md)에 따라 HWP3 전용 해석은
`src/parser/hwp3/`에서 끝낸다. 렌더러, 레이아웃, 문서 코어에 HWP3 전용 분기를 추가하지 않는다.

## 작업별 진입점

- GitHub Actions, 저장소 설정, branch protection과 runner:
  [github_operations.md](mydocs/manual/github_operations.md)
- 문서와 Git: [docs_and_git_workflow.md](mydocs/manual/codex/docs_and_git_workflow.md)
- 로컬 빌드와 WASM: [dev_environment_guide.md](mydocs/manual/dev_environment_guide.md)
- 메인터너 변경 범위별 검증: [local_validation.md](mydocs/manual/pr_review/local_validation.md)의 4.3
- CLI: [cli_commands.md](mydocs/manual/cli_commands.md)
- 시각 검증: [verification/README.md](mydocs/manual/verification/README.md)
- Studio UI 명칭과 CSS 접두어:
  [rhwp_studio_ui_conventions.md](mydocs/manual/rhwp_studio_ui_conventions.md)
