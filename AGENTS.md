# rhwp 작업 지침

이 파일은 저장소 안에서 재현 가능한 작업 부트스트랩이다. 세부 절차는 아래 권위 문서를 우선한다.

> **에이전트 진입점**: rhwp 를 도구로 부리는 에이전트는 루트 [`llms.txt`](llms.txt)와
> [에이전트 지식 지도](mydocs/manual/agent_knowledge_map.md)에서 시작한다.

## 문서 로딩 순서

1. `CLAUDE.md`
2. Codex는 프로젝트 메모리 덤프 `mydocs/manual/codex/MEMORY.md`
3. `mydocs/README.md`
4. 작업 성격에 맞는 `mydocs/manual/README.md` 또는 `mydocs/tech/README.md`
5. GitHub Actions·저장소 설정·branch protection·cache·runner 운영은
   `mydocs/manual/github_operations.md`
6. 개발·문서·Git 작업은 `mydocs/manual/codex/docs_and_git_workflow.md`
7. PR 검토·merge·후속 처리는 `mydocs/manual/pr_review_workflow.md`를 먼저 읽고,
   `mydocs/manual/pr_review/README.md`의 선택표가 지정한 기본·보조 자식 문서를 작업 전에 모두 읽는다.
   rhwp 첫 외부 contributor PR이면
   `mydocs/manual/pr_review/first_time_contributor.md`를 추가로 읽는다.
8. 로컬 빌드·WASM 검증은 `mydocs/manual/dev_environment_guide.md`,
   변경 범위별 검증 게이트는 `mydocs/manual/pr_review/local_validation.md`의 4.3
9. CLI 작업은 `mydocs/manual/cli_commands.md`
10. 시각 검증은 `mydocs/manual/verification/visual_verification_governance.md`와 `mydocs/manual/verification/visual_sweep_guide.md`

더 구체적인 문서가 이 요약과 다르면 그 문서를 따른다.

## 공통 원칙

- 구현 전에 관련 이슈, 기존 계획·보고서·트러블슈팅을 확인한다.
- 사용자 또는 다른 도구가 만든 변경은 임의로 되돌리거나 삭제하지 않는다.
- 작업 브랜치는 최신 `upstream/devel`을 기준으로 만들고, 일반 변경은 PR로 통합한다.
- collaborator·maintainer의 예외 처리와 오늘할일·PR review 문서는 `pr_review_workflow.md`의
  라우팅 결과에 해당하는 역할별 자식 절차를 따른다. 모 문서만 읽고 세부 경로를 추정해 진행하지 않는다.
- 작업 단계가 바뀌면 현재 단계의 변경을 커밋한 뒤 다음 단계 문서를 시작한다.
- GitHub comment, remote push, PR 생성은 사용자 승인을 받은 뒤 수행한다.
- Windows PowerShell에서 한글을 포함한 여러 단락의 PR·review·comment 본문은 here-string을
  `gh --body-file -`로 직접 pipe하지 않는다. UTF-8 **without BOM** 임시 Markdown 파일을
  `--body-file`로 전달하고, 게시 뒤 API로 한글·선두 BOM·`??` 치환 여부를 확인한다. 정확한
  명령과 정리 절차는 `mydocs/manual/pr_review_workflow.md`의 3.4.1을 따른다.

## 조판 수정과 검토 원칙

이 절은 Claude·Codex의 공통 구현 원칙이며, 모든 정식 PR review는 역할에 관계없이
[조판 원칙 준수 검토](mydocs/manual/pr_review/intake_and_review.md#27-조판-원칙-준수-검토)로
적용 여부와 근거를 확인한다. 세부 검증 명령·증적 보존·최종 판정은 해당 canonical 절차를 따른다.

### 구현 근거와 범위

- 수정 전에 위반된 조판 규칙, 사양·유효한 한컴 출력 등 독립적인 근거, 원인 계층과 반례를
  짧게 정리한다. 이후 승인된 범위의 구현·검증을 계속하며 같은 작업의 재확인을 반복하지 않는다.
- 특정 문서 ID·수치·우연한 속성 조합으로 화면을 맞추는 예외를 추가하지 않는다. 새 분기는
  조판 규칙상 필요한 조건과 적용되지 않아야 할 사례를 설명한다. 일반성 문제가 지적되면
  조건을 더 붙여 대상을 좁히기 전에 잘못된 가정을 제거한다.
- 잘못된 좌표를 clamp하거나 출력을 숨겨 겹침만 없애는 처리를 원인 해결로 간주하지 않는다.
  기존 줄 구성 정보를 재사용하거나 현재 수정 범위에서 공통 결과를 마련한다.
  전체 엔진 재작성은 이 원칙의 필수 조건이 아니다.

### 측정과 배치의 공통 결과

- 측정과 실제 배치는 동일한 줄 구성·메트릭 결과를 소비한다. 각 단계가 줄 소속을 따로 추측하거나
  기존 배치의 제한을 측정 조건에 복제하는 방식으로 일관성을 대신하지 않는다.
  레이아웃·paint·각 출력 backend의 적용 조건과 좌표계도 일치하는지 확인한다.
- `treat_as_char`는 글줄 흐름 참여 속성이다. 문단의 모든 표가 같은 줄에 있다는 근거로 사용하지 않는다.
  유효한 저장 LineSeg를 사용하는 경로는 줄 소속·위치를 보존하고, 재조판 경로는 텍스트와 객체의
  순서·명시적 개행·가용 너비·바깥여백을 반영한 줄 나눔 결과로 소속을 결정한다.
- 줄 높이는 해당 줄 요소들의 기준선 정렬과 여백을 포함한 점유 영역으로 계산한다.
  같은 줄의 표 높이를 세로로 합산하거나 선언 높이의 단순 최댓값만으로 결정하지 않는다.
  여러 줄의 문단 높이는 각 줄의 점유 높이와 줄간격을 반영한다.

### 좌표·높이 변경의 소비 지점 확인

- 구현 전에 변경할 값 하나를 골라 `생산 결과 → 측정의 소비 위치 → 실제 배치의 소비 위치 →
  최종 원점/높이를 바꾸는 분기`를 코드 위치로 추적한다. 같은 helper를 호출해도 그 뒤에 별도
  앵커 선택·덮어쓰기·clamp가 있으면 결과 공유가 끝난 것이 아니다. 기존 PR 설명이나 작업 기록에
  이 연결을 짧게 남기며 별도 양식·보고서 파일을 의무로 늘리지 않는다.
- 글자가 보이는지, 줄이 공간을 점유하는지, 개체가 흐름을 전진시키는지는 서로 다른 상태다.
  빈 문자열·공백만 있는 문단도 유효한 줄 상자·간격을 가질 수 있고 배경 개체는 보이면서도 흐름을
  밀지 않을 수 있다. 한 상태를 다른 상태의 대용으로 쓰는 조건이 변경 경로에 있으면, 두 상태가
  갈리는 입력을 먼저 실행한다. 모든 수정에 모든 조합을 요구하지 않는다.
- 측정에서 기존 배치의 제한을 흉내 내는 예외를 덧붙이지 않는다. 해당 범위의 실제 줄·개체 원점과
  점유 끝점을 공통 결과로 만들고, 측정과 배치가 이를 소비하게 한다. 남는 좌표 보정은 적용 경로와
  이유를 명시하고 같은 경계에서 최종 출력이 계약을 지키는지 확인한다.
- 제출 전 해당 반례를 `tests/cases/`의 정식 회귀 검사에 포함하고 실제 최종 좌표·점유 높이를
  검사한다. 진단 스크립트·문서 표·helper 단위 assertion만으로 끝내지 않는다. 수정 전 FAIL / 수정 후
  PASS와 정상 대조군 결과를 기존 검증 기록에 연결한다. 예상값은 구현이 계산한 높이만 재인용하지
  말고 저장 줄 메트릭·독립 출력·적용 가능한 정렬 불변식으로 설명한다.

### 분할·이어받기 변경의 입증

- 분할·이어받기에 영향을 주는 pagination, rowspan, clipping, continuation 변경은 구현자와 reviewer 모두 **실제 호출
  경로**를 대조한다. 공통 helper의 이름이나 같은 반환형만으로 공통 결과라고 판정하지 않는다.
  적용되는 일반/특수 분기별로 `시작·끝 컷과 소유 유닛 → 요구 높이 → 누적 예약 높이 →
  예산 실패 시 컷·이월 → 실제 배치`를 코드 위치와 함께 기록한다. 비해당 경로는 이유를 적는다.
- 이미 소비한 내용과 물리 공간을 구분한다. 내용 컷으로 표현되지 않는 빈 밴드, 시작 행의 남은
  물리 높이, 패딩·행 간격을 어디서 계상하는지 확인한다. 누적 높이는 실제 수용한 조각을 반영해야
  하며, 요구 높이가 안 맞는다는 이유로 작은 원래 높이를 수용한 뒤 paint에서 다시 늘리지 않는다.
- 해당 경계의 실행 증거를 남긴다: 원래 높이만 fit하는 예산, 시작/끝 컷이 있는 조각, 같은 조각에서
  여러 rowspan이 끝나는 경우, 마지막 유닛을 소비한 뒤의 종료. 관련 없는 조합을 전수 생성할 필요는
  없지만, 적용되는 경계를 테스트하지 않았다면 미검증으로 남긴다. 합성 계약의 기대값은 독립 근거로 정한다.
- 완료 조건은 목표 글자의 셀 내부 표시뿐 아니라 조각의 본문 점유와 다음 내용까지 포함한다.
  앞 조각의 끝과 다음 조각의 시작에서 유닛 누락·중복, 표 외곽·뒤 문단, 불필요한 빈 페이지를 확인한다.
  종료를 바꿀 때는 남은 물리 공간·캡션·각주 등 실제 후속 내용의 보존도 대조한다.
- 작은 경계 테스트와 영향 페이지의 직접 Visual Sweep을 먼저 수행해 보정 방향을 확인한 뒤
  비용이 큰 전체 검증을 진행한다. 선행 진단은 최종 head의 필수 회귀·lint·fresh WASM·시각 증적을
  대체하지 않는다. 코드가 다시 바뀌면 영향받은 경계·시각 확인도 다시 수행한다.
  제출·검토 기록에는 위 자료를 짧은 표나 기존 증적 링크로 남기며 체크 표시만으로 대체하지 않는다.

### 저장 조판 정보의 유효성 확인

- 저장 LineSeg·폭·캐시의 수용 조건을 바꾸기 전에 입력이 실제 저장본인지, 수동 작성·수정한
  합성 입력인지 확인한다. 수동 메타데이터만으로 수용 조건을 완화하지 않는다. 독립적인 사양·
  정상 생성본·기준 출력과 다르면 입력의 가정부터 재검토한다.
- 저장 정보 재사용과 편집 후 재조판은 각각의 계약으로 검증한다. 두 경로가 공유하는 조판 규칙은
  독립 근거로 확인하고, 한 경로의 통과를 다른 경로의 증거로 대신하지 않는다.
- 구현 전에 `입력 생성 방식 → 독립 기준 출처와 관측값 → 기대 결과`를 짧게 기록하고,
  검증 뒤 수정 전후 결과를 연결한다. 근거가 부족한 범위는 미검증으로 남기며 시각 개선 완료로
  판정하지 않는다. 이 기준은 작성자·reviewer·메인터너 보정 모두에 적용한다.

### 증거와 기준값

- 기대값은 수정 구현과 독립적인 근거에서 정한다. 중첩 표의 줄 구성 수정은 원본 사례와 함께 같은 줄의
  여러 표, 명시적 개행, 너비 부족에 따른 줄바꿈, 앞뒤 텍스트·여백 혼재를 검증한다.
  관련 없는 변경에는 이 사례들을 기계적으로 요구하지 않는다.
- 합성 입력의 계약 테스트와 정상 한컴 문서·출력 증거를 구분한다. 자료가 없는 사례와 미실행 경로는
  미검증으로 기록하고, 테스트 통과를 한컴 출력과의 일치 증거로 승격하지 않는다.
- 동일 입력의 기준 출력과 변경 전후 실제 출력을 같은 페이지·영역에서 직접 비교한다.
  표 외곽·뒤 문단 위치·겹침·누락·줄바꿈 등 주장한 의미를 확인하며, 페이지 수·텍스트 추출·해시·
  픽셀 점수 또는 빈 줄에 보이는 글자가 없다는 이유만으로 시각 통과를 선언하지 않는다.
- 렌더링 변경은 [Visual Sweep](mydocs/manual/verification/visual_sweep_guide.md)을 실행한다.
  영향 페이지의 Native/fresh WASM compare·standalone overlay·review를 산출해 직접 확인하고,
  source SHA·입력/기준 PDF·페이지·명령·대표 PNG·남은 차이를 결과보고에 연결한다. PR review를
  요청하기 전에는 대표 review·overlay PNG를 **PR 본문에서 실제 Markdown 이미지로 표시**한다.
  경로·임시 output·review 문서 링크만으로 대체하지 않으며, PR head repository와 정확한 head SHA로
  고정한 raw URL을 쓴다. code head가 바뀌면 시각 증적과 본문 URL도 다시 만든다. merge 뒤에는
  같은 asset을 merge SHA로 고정한 URL로 contributor comment에 다시 남긴다.
  변경 후 이전 캡처를 재사용하지 않으며 CI나 자동 점수만으로 직접 판독을 대신하지 않는다.
  영향 페이지에서 큰 위치·줄바꿈·외곽선 차이가 보이면 전체 회귀보다 이 차이의 원인 확인과
  재캡처를 먼저 한다. 기존 차이 또는 합성 입력이라는 분류만으로 보류 사유를 해소하지 않는다.
  기준 입력의 저장 정보가 무효라면 원본과 실패 증거를 보존하고, 독립적으로 재생성한 대조군의
  생성 절차와 변경점을 공개한다. 대조군 통과를 원본 입력의 일치로 바꾸어 보고하지 않는다.
- 기준 PDF는 형식 버전이나 Creator의 한컴 연도·빌드만으로 제한하지 않는다. `Hwp 2020
  0.0.0.0` / PDF 1.4도 출처·원문 대응·정상 출력이 확인되면 재사용한다. 실제 출력 결함과
  버전 표기를 구분하며, [기준 PDF 재사용](mydocs/manual/pr_review/visual_fixture_evidence.md)의
  절차를 따른다.
- 테스트 통과를 목적으로 baseline·golden·래칫 허용치를 완화하지 않는다. 변경이 필요하면
  발생 환경·전후 노드와 경계 좌표·독립적인 기준 출력·의도된 변화의 근거를 분리해 제시한다.
  실제 회귀나 원인 미상의 증가분을 기준값 갱신으로 숨기지 않는다.
  [조판 규칙과 기준값 변경 증거](mydocs/manual/pr_review/visual_fixture_evidence.md#조판-규칙과-기준값-변경-증거)를 따른다.

### 완료 보고와 review

- 적용한 규칙과 공통 결과, 검증한 source SHA·명령·결과·증적, 기준값 변경 사유와 미검증 범위를
  기록한다. 검증 후 코드 변경에는 이전 결과를 그대로 적용하지 않는다.
- 실행으로 검출한 회귀, 코드 검토상 우려, 필수 증거 부족을 구분한다. 원본 샘플의 개선이나
  CI 통과만으로 원칙 준수를 단정하지 않으며, reviewer는 정확한 검토 head의 코드와 증거로 판정한다.

### 구현 주장과 검증 증거의 대조

조판 변경의 작성자, reviewer, 메인터너 보정에 동일하게 적용한다. 기존 테스트·PR 본문·증적의
구체적인 위치를 연결하며, 같은 내용을 별도 보고서나 표에 중복 작성하지 않는다.

- 검증 범위는 수정한 함수뿐 아니라 변경된 값·조건을 소비하는 실제 호출 경로에서 정한다.
  공통 helper 이후의 값 재계산·덮어쓰기, 조기 반환, 별도 배치 경로도 확인한다.
- 변경한 동작별로 `구현 주장 → 적용/비적용 경로 → 독립 기대값 → 실제 검사 항목 →
  수정 전후 관측값`을 연결한다. 주장한 경로 중 증거가 없는 부분은 미검증으로 남긴다.
- 반례는 구현의 핵심 가정을 깨는 사례로 고른다. 좌표 겹침만으로 객체 소유를, 같은 형태만으로
  같은 생성 출처나 배치 규칙을 입증할 수 없다. 해당 변경이 사용하는 가정의 반례를 검증하며,
  관련 없는 조합을 전수 생성하지 않는다.
- 결함 검출을 주장하는 회귀 테스트는 수정 전 코드에서 의도한 원인으로 실패하고 수정 후
  통과하는지 확인한다. 빌드·환경 실패는 결함 재현으로 세지 않는다. 수정 전에도 통과하면
  해당 결함의 검출 증거로 인정하지 않는다. 실행하지 못한 경우 미검증 사유를 기록한다.
- 검사 항목이 구현 주장의 의미를 직접 확인하는지 대조한다. 내용 존재 검사는 위치·테두리의
  정확성을, 단조 증가하는 컷은 유닛의 완전한 보존을, 오류 건수 유지는 오류 크기의 무회귀를
  입증하지 않는다. Visual Sweep은 주장한 의미에 맞는 같은 페이지·영역의 실제 출력을 확인한다.
- reviewer는 작성자의 경로 목록을 실제 코드와 대조하고 핵심 가정이 깨지는 적용 경계가
  증거에 포함됐는지 확인한다. 없으면 필요한 반례를 실행하거나 해당 범위를 미검증으로 판정한다.
  기존 증거가 충분하면 같은 검사를 반복 실행할 필요는 없다.
- 판정은 `충족 / 미충족 / 미검증 / 비해당`으로 구분한다. 미충족에는 실행으로 확인한 결함과
  코드 검토상 규칙 위반을 구별해 적고, 증거 부족은 미검증으로 남긴다. CI 통과·증적 문서 존재·
  체크 표시만으로 미검증을 충족으로 바꾸지 않는다.
- 필수 증거가 부족한 범위는 완료·승인으로 보고하지 않는다. 부분 개선을 제출할 경우 해결 범위와
  남은 문제를 명시하고, PR 본문과 최종 merge 메시지의 이슈 종료 표현도 그 범위에 맞춘다.

## 문서와 검증

- **로컬 Rust·WASM 산출물 재사용**: 일반 개발·이슈 수정·PR review의 기본
  `CARGO_TARGET_DIR`/`--target-dir`는 항상 `target/pr-review`다. 이 경로의
  `release`, `release-test`, `debug`, `wasm32-unknown-unknown`을 Native와 WASM이 함께
  재사용한다. 이슈 번호나 review 이름으로 `target/<name>`을 새로 만들지 않는다.
  다른 실행 중인 Cargo 작업의 산출물과 충돌할 우려가 있으면 새 경로를 만드는 대신
  실행 중인 작업·소유자를 먼저 확인하고, 필요할 때만 사용자가 별도 경로를 지시한다.
  `target/pr-review`은 공유 캐시이므로 임의로 삭제·초기화하지 않는다.
- **Studio 개발 서버에 WASM 반영**: Rust/WASM 변경을 `npx vite --host 0.0.0.0 --port 7700`
  같은 `rhwp-studio` 개발 서버에서 확인할 때는 반드시
  `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`를
  쓴다. 이 wrapper는 성공한 기본 `pkg/` web package의 `rhwp.js`·`rhwp_bg.wasm`을
  `rhwp-studio/public/`에도 자동 동기화한다. SHA-256 일치 및 브라우저 새로고침 뒤 실제
  동작을 확인한다. Rust target만 빌드하거나 wrapper 밖에서 `pkg/`만 갱신한 상태를 Studio
  검증으로 보고하지 않는다.
- **Rust source 또는 Rust test/baseline helper를 바꾼 모든 PR·push 직전 필수**: 포맷만 확인하고
  Clippy를 CI에 넘기지 않는다. PR review worktree에서 파생 integration suite를 준비한 뒤 아래
  Rust lint 묶음을 **순차로** 모두 통과시킨다. `cargo clippy -- -D warnings`만으로는
  WASM 전용 cfg와 workspace member·integration target을 놓치므로 CI `Lint (fmt, clippy, WASM
  check)`의 세 Clippy 단계를 각각 확인한다.
  ```
  git fetch upstream devel
  rhwp_review_base_sha="$(git rev-parse upstream/devel)"
  node scripts/rust-test-suite-manifest.mjs --prepare
  cargo fmt --all
  cargo fmt --all -- --check
  cargo clippy --locked --target-dir target/pr-review -- -D warnings
  cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
    --target-dir target/pr-review -- -D warnings
  cargo build --locked --workspace --target-dir target/pr-review
  cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
  node scripts/rust-test-suite-manifest.mjs --check --base-ref "${rhwp_review_base_sha:?검증할 PR base SHA를 먼저 고정하세요}"
  ```
  새 integration test source를 추가한 경우 `--prepare`가 만든 파생 파일은 검증 뒤 review
  worktree에서만 복원하고 PR에 stage하지 않는다. 한 단계라도 실패하면 수정·재실행 전에는 push 또는
  PR을 만들지 않는다. 세부 범위와 예외는 `mydocs/manual/pr_review/local_validation.md`의 4.3을
  따른다.
- **source-side test 변경 시 추가**: `src/**` 또는 `crates/*/src/**`의 `#[cfg(test)]`를 변경하면
  `node scripts/rust-unit-test-tiers.mjs --check --base-ref "${rhwp_review_base_sha:?검증할 PR base SHA를 먼저 고정하세요}"`를 실행한다. 이 검사는 source와 정책만 읽고
  파생 inventory를 만들지 않는다. 진단용 `--generate` 결과는 `tests/generated/unit-test-tiers.json`에
  남으며 커밋하지 않는다.
- 정책 검사에는 위에서 고정한 PR base SHA를 전달한다. `--check` 단독은 PR base 대비 증가를
  검사하지 않아 CI와 동등하지 않다. base가 바뀌면 해당 비교를 다시 실행하고 base/head SHA를
  기록한다. 준비·예외·기여자 경로는 [정책 base 비교](mydocs/manual/pr_review/local_validation.md#정책-검사의-base-고정)를 따른다.
- **review·maintainer worktree와 CI 전용**: 새 integration source는 `tests/cases/` 원본만 PR에
  포함한다. `node scripts/rust-test-suite-manifest.mjs --prepare`와 manifest `--check`는
  파생 suite를 준비한 review worktree와 CI에서만 수행한다. generated suite·manifest는
  검증 증적일 뿐 source PR에 stage하지 않는다. 기본 `--prepare`는 root `Cargo.toml`을 바꾸지 않으며,
  통합 불가 예외 target registry를 갱신하는 메인터너 전용 PR만 `--sync-cargo-targets`로 Cargo marker
  블록을 동기화할 수 있다. 세부 절차는 `CONTRIBUTING.md`와
  `mydocs/manual/pr_review/local_validation.md`의 integration test 절을 따른다.
- 문서 역할·생명주기·canonical 관계는 `mydocs/README.md`의 manifest를 따른다.
- 문서 이동·정보구조 리팩토링의 링크와 메타데이터 검사는
  `mydocs/manual/markdown_link_check_guide.md`를 따른다. 일반 Markdown 추가·수정에는 자동 CI를 실행하지 않는다.
- 렌더링·레이아웃 변경은 시각 검증 정책에 따라 PDF/SVG 또는 동등한 근거를 남긴다.

## 작업 증빙 — 에이전트 기본 경로 (권장)

LLM 에이전트가 이 저장소에서 문서를 실제로 편집·생성하는 작업의 기본 경로다.
전부 devel 에 병합된 기능이며, **권장이지 제출 조건이 아니다**. 도구별 지침 파일
(`.github/copilot-instructions.md`·`.cursor/rules/rhwp.mdc`·`GEMINI.md`·
`.windsurfrules`·`.clinerules`)은 전부 이 절을 가리키는 얇은 포인터다 — 실질
내용은 여기 한 곳에서만 고친다.

이 경로는 [에이전트 작업 표준(AWS) 1.0](mydocs/tech/standards/agent_work_standard.md)의
**레퍼런스 구현**이다 — 아래 영수증·계보·서명·앵커·정산이 표준의 AW-L1~AW-L5 다.
표준은 도구 무관하고 열려 있다(준수는 강요가 아니라 유용성 때문). 기계용 정본은
[`agent_work_standard.json`](mydocs/tech/standards/agent_work_standard.json).

- **영수증**: 편집 계획을 세우고 실행은 캡슐과 함께 남긴다 —
  `rhwp replay --plan-json <계획> --capsule work.capsule.json --json`.
  캡슐 하나가 "어떤 입력에서 어떤 계획으로 어떤 산출이 나왔나"를 3해시
  (입력·계획·산출)로 고정하고, 제3자가 재실행으로 검증할 수 있다.
- **계보**: 연속 작업은 `--parent 이전.capsule.json` 으로 잇고
  `rhwp lineage <머리캡슐> --json` 으로 검증한다.
- **회계**: 캡슐 폴더의 전수 재검증은 `rhwp audit <폴더> --json` — 재현율이
  수치로 나온다.
- **PR 증빙**: 관련 `--json` 봉투 원문 또는 캡슐 파일을 PR 본문에 붙인다
  (PR 템플릿 체크리스트). 리뷰어는 주장 대신 재계산으로 확인한다.
- **확장 축**: 서명·앵커·게이트·연합·선택적 공개·정산·감사 표준은 검증 사다리
  좌표 [#4463](https://github.com/edwardkim/rhwp/issues/4463)를 본다 — 병합 전
  기능은 규약이 아니라 로드맵이다.
- 기여 절차 전체는 Claude Code 스킬 `rhwp-contributor` 가 체크리스트로 안내한다.
