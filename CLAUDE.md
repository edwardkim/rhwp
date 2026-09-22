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

## 빌드 산출물 재사용

- 로컬 Native·WASM·테스트 빌드는 `target/pr-review`을 공용 target directory로 쓴다.
  이슈별·검토별 `target/<name>`을 새로 만들지 않는다. Native `release`와 WASM
  `wasm32-unknown-unknown`을 같은 경로에서 재사용해 재빌드를 피한다.
- `target/pr-review`은 공유 캐시다. 실행 중인 Cargo 작업의 소유·상태를 먼저 확인하고,
  그 경로를 임의로 삭제·초기화하지 않는다. 별도 target은 사용자가 명시한 경우에만 쓴다.
- `rhwp-studio` 개발 서버에서 Rust/WASM 변경을 확인할 때는 반드시 **저장소 루트
  (`/Users/tsjang/rhwp`, `scripts/`·`pkg/`·`rhwp-studio/`가 함께 있는 디렉터리)**에서
  `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`를
  실행한다. `rhwp-studio/` 안에서 실행하면 wrapper 경로와 출력 `pkg/`가 모두 달라져 실패하거나
  개발 서버가 이전 bundle을 읽는다. 이 wrapper가 루트 기본 `pkg/` web package를 만든 뒤 `rhwp.js`와 `rhwp_bg.wasm`을
  `rhwp-studio/public/`에도 자동 동기화한다. SHA-256 일치와 브라우저 새로고침 뒤 실제
  동작을 확인한다. target 산출물만 만든 상태는 Studio 반영 검증이 아니다.

## 수정 전에 확정할 것

- 이슈의 실제 입력, 기대 결과, 수정 범위와 비범위를 확인한다. 구현 결과를 보고 기대값을
  역으로 정하지 않는다. 사양, 유효한 기준 문서 또는 독립적인 기존 계약으로 정답을 정한다.
- 증상이 발생한 계층과 원인을 추적한다. 렌더링·조판 수정과 그 PR review에는
  [공통 조판 원칙](AGENTS.md#조판-수정과-검토-원칙)을 적용한다.
- 저장 조판 정보를 바꾸면 [유효성 확인](AGENTS.md#저장-조판-정보의-유효성-확인)을 구현 전에 수행하고
  입력 생성 방식·독립 기준·관측값을 기대 결과에 연결한다.
- 성공 사례뿐 아니라 변경 조건이 적용되면 안 되는 반례와 관련 경계를 정한다.
  일반/특수 객체 혼재, 반복 호출, 빈 값/누락 값, 저장 후 재열기처럼 해당 수정에 필요한 사례를 고른다.
- 좌표·높이를 바꿀 때는 [소비 지점 확인](AGENTS.md#좌표높이-변경의-소비-지점-확인)을 실행한다.
  측정 helper에서 멈추지 말고 실제 paint 직전 원점 선택까지 따라가며, 그 사이 상태를 혼동하는
  조건을 찾는다. 예를 들어 `가시 글자 없음`이 `줄 점유 높이 없음`을 뜻하는지 대조한다.

## 구현과 검증의 완료 기준

- 한 번만 처리할 데이터는 실제 소유 범위에서 한 번 처리하며 내부 반복문마다 초기화하지 않는다.
- 파서와 저장 변경에서는 null, 빈 문자열, 0, 참조 번호를 구별한다. 문자열 길이의 바이트/UTF-16
  단위, 재귀 깊이와 자원 제한, round-trip 보존을 관련 계약에 맞춰 다룬다.
- 회귀 테스트는 수정 전 결함을 드러내고 수정 후 독립적인 기대 결과를 만족해야 한다.
  fixture 누락 시 조용히 return하거나 실행 대상이 0건인 결과를 통과 증거로 삼지 않는다.
- 위 소비 경로에서 찾은 반례는 제출 전에 정식 `tests/cases/` 검사로 실행한다. 정상 입력의
  Top/Center/Bottom만 통과한 결과로 빈 줄 등 다른 상태의 원점 선택까지 검증했다고 쓰지 않는다.
  완료 보고에는 실제 검사 이름과 수정 전후 결과를 연결한다. CI 녹색은 누락한 assertion을 대신하지 않는다.
- 렌더링의 공통 결과·반례 검증·baseline 변경 근거는 위 공통 조판 원칙을 따른다.
  차트 변경은 종류와 축 등 해당 변경이 주장한 의미도 직접 확인한다.
- **렌더링 변경은 [Visual Sweep](mydocs/manual/verification/visual_sweep_guide.md)으로 검증한다.**
  동일 입력·페이지의 기준 PDF와 수정 전후 Native/fresh WASM 출력을 비교하고, compare·standalone
  overlay·review를 산출해 직접 연다. 표 외곽·앞뒤 문단·줄바꿈·겹침·누락 등 변경 주장을 판독한다.
  실행한 source SHA·입력/PDF·페이지·명령·대표 PNG와 남은 차이를 기존 결과보고에 연결한다.
  reviewer가 실제 결과를 열어 볼 수 있게 대표 review·overlay PNG를 PR 본문에 Markdown 이미지로
  표시한다. 경로·임시 output·review 문서 링크만으로 대신하지 않고, PR head repository와 정확한
  head SHA로 고정한 raw URL을 사용한다. code head가 바뀌면 캡처와 본문 URL도 갱신한다.
  코드 변경 뒤에는 영향 페이지를 다시 캡처하며, CI 녹색·자동 픽셀 점수만으로 대체하지 않는다.
  직접 판독에서 큰 차이가 남으면 비용이 큰 전체 회귀를 시작하기 전에 원인을 수정하고 다시
  캡처한다. 작은 정렬 불변식의 PASS로 외곽선·뒤 문단·줄바꿈 차이를 승인하지 않는다.
  기준 입력을 재생성할 때는 기존 실패 입력을 보존하고 바뀐 저장 정보를 공개하며, 새 입력의
  시각 일치를 원본의 일치로 보고하지 않는다.
  유효한 기준 PDF가 없거나 합성 입력 자체의 출력이 비정상이면 그 한계를 기록하고 시각 일치
  완료를 선언하지 않는다. 증적 보존은 위 공통 원칙과 시각 증적 정책을 따른다.
- 분할·이어받기 변경은 [경로·컷·높이·종료 입증](AGENTS.md#분할이어받기-변경의-입증)을 제출 전
  수행한다. helper를 공유했다는 설명 대신 일반/특수 호출 분기와 실제 소비·예약·배치 값을 대조한다.
  목표 셀뿐 아니라 앞뒤 조각과 다음 내용의 보존을 먼저 작은 테스트·Visual Sweep으로 확인한다.
- 완료 보고 전에 [구현 주장과 검증 증거의 대조](AGENTS.md#구현-주장과-검증-증거의-대조)를 수행한다.
  증적의 존재뿐 아니라 실제 검사 항목이 PR 본문의 주장을 입증하는지 확인하고 부족한 범위는
  미검증으로 남긴다. 메인터너 보정에도 같은 기준을 적용한다.
- 원본과 대응하는 유효한 한컴 PDF가 있으면 재사용한다. PDF 형식 버전, Creator의 한컴 연도나
  `0.0.0.0` 표기만으로 기준을 폐기하거나 MCP 재변환을 요구하지 않는다. 형식 버전·생성 제품
  정보·MCP engine을 구분하고 공통 기준 PDF 재사용 절차를 따른다.
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
