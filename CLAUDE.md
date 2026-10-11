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

## 페이지별 TSV와 PR 재검토 필수 조건

조판·렌더링에 영향을 주는 변경은 **검증 대상 각 문서의 전체 페이지 TSV를 먼저 산출**한다.
`tolerant_content_match_percent`(2px 이웃 관용 내용 실루엣 일치율)를 페이지별로 확인하며,
**한 페이지라도 90% 미만이면 PR을 재검토한다.** 평균값이나 대표 페이지의 높은 점수로
미달 쪽을 상쇄하지 않는다. 정확히 90%는 통과한다.

`re_review_required` 상태에서는 새 PR 생성과 열린 PR의 승인·통합을 보류한다.
작성자는 원본·독립 한컴 Print PDF·overlay를 대조해 원인을 수정하고 새 head에서 다시 산출한다.
누락·측정 불가·전체 쪽수 불일치도 보류 조건이다. 모든 페이지가 90% 이상이어도
변경 영역의 수식·미세 간격을 확대 판독한 뒤 완료 여부를 판단한다.

명령·저장 위치는 [「실루엣 보조값만 빠르게 TSV 산출」](mydocs/manual/verification/visual_sweep_guide.md#실루엣-보조값만-빠르게-tsv-산출)을 따른다.
기여자는 전체 페이지 **Native TSV와 fresh WASM TSV 원본 모두 및 대표 PNG**를 제출한다.
검토자의 Native/fresh WASM 검증은 해당 역할의 절차를 따른다. 해결 불가능한 실제 글꼴 차이는
[정식 글꼴 예외 계약](mydocs/manual/verification/visual_sweep_guide.md#해결-불가능한-글꼴의-pr-제출-예외)의 증거와 gate 판정으로만 처리한다.

## 빌드 산출물 재사용

- macOS·Linux·Windows의 로컬 Native·WASM·테스트 빌드는 각 host의 기본 rhwp 작업공간 아래
  `target/pr-review` 한 곳을 공용 target directory로 쓴다. review worktree에서도 같은 절대 경로를 전달한다.
  이슈별·검토별 target, `target/pr-review/<name>`, worktree별 target을 새로 만들지 않는다. Native `release`와 WASM
  `wasm32-unknown-unknown`을 같은 경로에서 재사용해 재빌드를 피한다.
- `target/pr-review`은 공유 캐시다. 실행 중인 Cargo 작업의 소유·상태를 먼저 확인하고,
  그 경로를 임의로 삭제·초기화하지 않는다. 별도 target은 사용자가 명시한 경우에만 쓴다.
- `rhwp-studio` 개발 서버에서 Rust/WASM 변경을 확인할 때는 반드시 **저장소 루트
  (`scripts/`·`pkg/`·`rhwp-studio/`가 함께 있는 디렉터리)**에서
  `CARGO_TARGET_DIR="${rhwp_review_target_dir:?기본 작업공간에서 공용 target 경로를 먼저 고정하세요}" scripts/wasm-pack-locked.sh --target web --out-dir pkg`를
  실행한다. `rhwp-studio/` 안에서 실행하면 wrapper 경로와 출력 `pkg/`가 모두 달라져 실패하거나
  개발 서버가 이전 bundle을 읽는다. 이 wrapper가 루트 기본 `pkg/` web package를 만든 뒤 `rhwp.js`와 `rhwp_bg.wasm`을
  `rhwp-studio/public/`에도 자동 동기화한다. SHA-256 일치와 브라우저 새로고침 뒤 실제
  동작을 확인한다. target 산출물만 만든 상태는 Studio 반영 검증이 아니다.

## 수정 전에 확정할 것

- 렌더링·조판·페이지 배치 변경의 새 회귀 테스트나 fixture/golden은 같은 원본·독립 한컴 PDF의
  Native/fresh WASM Visual Sweep에서 검증 범위의 **최저 일치율이 90% 이상**일 때만 추가한다.
  관련 모든 페이지·fixture·출력 경로 중 하나라도 90% 미만이거나 측정 불가이면 추가를 보류하고
  실제 출력을 먼저 개선한다. 쪽수 검사는 전체 페이지를 비교한다. 평균값·글꼴 예외·CI 통과로
  대신하지 않는다. 기존 검사·파일의 이관은 아래 별도 이슈 절차로 처리한다. 세부 증거는
  [회귀 추가 선행 조건](mydocs/manual/pr_review/visual_fixture_evidence.md#렌더링-회귀-테스트-신규-추가의-시각-검증-선행-조건)을 따른다.
- 이슈의 실제 입력, 기대 결과, 수정 범위와 비범위를 확인한다. 구현 결과를 보고 기대값을
  역으로 정하지 않는다. 사양, 유효한 기준 문서 또는 독립적인 기존 계약으로 정답을 정한다.
- 관련 기존 회귀 테스트도 잘못된 기대값을 고정했을 수 있다. 독립 PDF와 수정 전·후 Visual Sweep으로
  [기대값의 적절성](mydocs/manual/pr_review/visual_fixture_evidence.md#기존-회귀-테스트의-기대값-재검토)을 확인하고 실제 회귀·기대값 오류·미검증을 구분한다.
  실패한 렌더링·페이지 검사는 **원 회귀 테스트의 원본 입력 출력 보정 → 버전에 맞는 독립 한컴
  Print PDF와 Native/fresh WASM Visual Sweep에서 관련 모든 페이지가 90% 이상인지 확인 →
  독립 기대값에 따라 필요한 회귀 검사 변경** 순서를 따른다. 쪽수 검사는 전체 페이지를 비교한다.
  90% 미만·측정 불가이면 기대값·baseline 갱신은 보류한다. **SHA를 고정한 `upstream/devel`에서
  이미 90% 미만인 원본에 등록되어 있던 회귀 검사**가 부적합한 경우에만 새로운 이슈로 등록하고
  해당 검사와 HWP/HWPX/PDF를 활성 검증 경로에서 제거·이관한다. 같은 원본·독립 Print PDF의
  base/head 비교로 기존 미달과 현재 수정의 회귀를 구분한다. 현재 수정으로 발생한 회귀는 구현을
  고치며, base가 미검증이면 이관 판정을 보류한다. 해당 이슈에서 보정·90% 이상·확대 판독을
  충족한 뒤 관계형 회귀로 복원한다.
  [별도 이슈 이관](mydocs/manual/pr_review/visual_fixture_evidence.md#90-미만-회귀-대상의-별도-이슈-이관)을 따른다.
  정확한 기존 검사는 유지하고 구현을 고친다. 기여자·메인터너·콜라보레이터 모두 적용하며,
  사용자가 승인한 메인터너 보정도 이 선행 조건을 면제하지 않는다. 정확히 90%는 통과한다.
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
- 실물 문서 회귀는 쪽·영역별 문단/개체 소속, 순서·누락·중복, 번호 보존을 검사한다.
  신규 검사와 기존 검사의 변경 모두 화면의 절대 px 좌표·고정 px 영역으로 대상을 선택하거나
  전체 SVG 해시로 잠정 배치를 고정하지 않는다. 배치 결함은 셀 내부
  포함·앞뒤 순서·겹침 같은 관계를 검사하고 실제 위치와 모양은 독립 PDF의 Visual Sweep으로
  확인한다. 기존 px 고정 검사를 관계 검사로 바꾸는 경우에도 위 90% 선행 조건을 먼저 충족한다.
  기존 검사는 승인된 검토 범위에서 근거를 남겨 교정하며 일괄 제거하지 않는다.
- 회귀 테스트는 수정 전 결함을 드러내고 수정 후 독립적인 기대 결과를 만족해야 한다.
  fixture 누락 시 조용히 return하거나 실행 대상이 0건인 결과를 통과 증거로 삼지 않는다.
- 위 소비 경로에서 찾은 반례는 제출 전에 정식 `tests/cases/` 검사로 실행한다. 정상 입력의
  Top/Center/Bottom만 통과한 결과로 빈 줄 등 다른 상태의 원점 선택까지 검증했다고 쓰지 않는다.
  완료 보고에는 실제 검사 이름과 수정 전후 결과를 연결한다. CI 녹색은 누락한 assertion을 대신하지 않는다.
- 렌더링의 공통 결과·반례 검증·baseline 변경 근거는 위 공통 조판 원칙을 따른다.
  차트 변경은 종류와 축 등 해당 변경이 주장한 의미도 직접 확인한다.
- **90% 이상은 시각 완료 판정 전체가 아니다.** 독립 한컴 Print PDF와 변경 전·후 Native/fresh WASM의
  같은 영역을 같은 배율로 확대해 수식 기호·장평·정체/이탤릭·기준선·첨자·화살표·장식과 관계 기호
  양쪽·앞뒤 본문·보기 번호·탭·줄 사이 간격을 확인한다. 변경 영역·알려진 차이·사용자 지적 영역을
  반드시 포함하고 점수와 직접 판독을 별도로 기록한다. 결함이 보이면 점수 통과로 면제하지 않고
  출력 보정·재검증 후 회귀 추가/변경·완료·승인 여부를 판정한다. 기여자·메인터너·콜라보레이터와
  승인된 보정 모두 [90% 통과 후 확대 판독](mydocs/manual/pr_review/visual_fixture_evidence.md#90-통과-후-수식간격의-확대-판독)을 따른다.
- 조판·렌더링 영향 변경은 파일 경로와 관계없이 Native/fresh WASM Visual Sweep·TSV를 반드시 실행한다.
  편집 command·parser·model·serializer도 실제 조판 소비 경로에 영향을 주면 적용한다.
  기준 PDF는 저장 버전에 맞는 한컴 [Print 출력 계약](mydocs/manual/mcp_hwp2024Convert_usage.md#기준-pdf-인쇄-계약)을 따른다.
  편집 변경은 동일하게 편집한 저장본을 Print 출력한다. PDF 부족은 미검증이며 비해당이 아니다.
  TSV·실행 로그·중간 JSON은 ignored `output/pr-review/<id>/`에 보존하고 Git에 커밋하지 않는다.
- 페이지별 TSV 명령·저장 위치는 [「실루엣 보조값만 빠르게 TSV 산출」](mydocs/manual/verification/visual_sweep_guide.md#실루엣-보조값만-빠르게-tsv-산출)를 따른다. Native/fresh WASM 예제를 각각 실행한다.
  검증 대상 전체 페이지의 Native/fresh WASM TSV에서 최저값·90% 미만·누락 쪽을 기록하고,
  미달/구조 차이/대표 경계만 비교 PNG를 추가 생성해 직접 판독한다. 대표 PR 이미지 제출은 유지한다.
  실루엣 계산 방법·이진화 원값·색상 경계 대조 픽셀 수도 함께 기록한다.
  TSV 성공·`not_evaluated`만으로 승인하지 않으며 각주 수량·문단 소속·누락·중복·전체 쪽수를 별도 검증한다.
  기존 PNG 재사용 결과와 최신 코드 재출력을 구분하고 입력 해시·빌드/글꼴 출처를 보존한다.
- **렌더링 변경은 [Visual Sweep](mydocs/manual/verification/visual_sweep_guide.md)으로 검증한다.**
  PDF 비교에는 Native와 fresh WASM의 인쇄 프로필을 사용한다. 빈 누름틀 안내문은
  출력 단계에서 제외하고 실제 입력된 본문은 비교한다. 프로필과 증적 기록 방식은
  [인쇄 프로필 절](mydocs/manual/verification/visual_sweep_guide.md#pdf와-같은-인쇄-프로필)을 따른다.
  외부 기여자는 [CONTRIBUTING.md의 렌더링 PR 제출 절차](CONTRIBUTING.md#메인터너-검토-기록과의-구분)에 따라
  원 PR을 생성·갱신하기 전에 제출할 code head의 영향 페이지를 캡처한다. 대표 review·overlay PNG를
  안정 경로에 커밋하고, 원 PR 본문에 그 head의 repository·SHA로 고정한 raw URL의 실제
  `![...](...)` 이미지 두 종류와 입력·기준 PDF·페이지·결과를 넣는다. 게시 후 본문과 이미지를
  실제 PR 화면에서 확인한다. 증적이나 기준 PDF가 없으면 제출 완료로 표시하지 않는다.
  동일 입력·페이지의 기준 PDF와 수정 전후 Native/fresh WASM 출력을 비교하고, compare·standalone
  overlay·review를 산출해 직접 연다. 표 외곽·앞뒤 문단·줄바꿈·겹침·누락 등 변경 주장을 판독한다.
  실행한 source SHA·입력/PDF·페이지·명령·대표 PNG와 남은 차이를 기존 결과보고에 연결한다.
  reviewer가 실제 결과를 열어 볼 수 있게 대표 review·overlay PNG를 PR 본문에 Markdown 이미지로
  표시한다. 경로·임시 output·review 문서 링크만으로 대신하지 않고, PR head repository와 정확한
  head SHA로 고정한 raw URL을 사용한다. code head가 바뀌면 캡처와 본문 URL도 갱신한다.
  코드 변경 뒤에는 영향 페이지를 다시 캡처하며, CI 녹색·자동 픽셀 점수만으로 대체하지 않는다.
  `scripts/visual_sweep.py`의 2px 이웃 관용 내용 실루엣 일치율이 대표 review PNG 중 하나라도
  90% 미만이거나 측정 불가이면 새 PR을 만들지 않고 이미 열린 PR은 승인·통합하지 않는다. 기여자가 자기 branch에서
  원인을 재검토·수정하고 새 head에서 gate를 통과한 뒤에만 PR을 생성·갱신한다. reviewer는 보류를 기록하며
  메인터너 보정으로 대신하지 않는다.
  같은 원본·출력 환경의 한컴 PDF와 rhwp **전체 페이지 수**가 다르면 영향 쪽의 점수가
  통과해도 PR을 재검토한다. 누락·추가된 쪽의 시작 경계와 앞뒤 내용을 확인해 원인을 고치고,
  새 head의 페이지 수와 시각 증적을 다시 검증한다. 글꼴 예외는 페이지 수 차이를 면제하지 않는다.
  차트가 OLE 컨테이너에 있어도 편집 가능한 `c:chartSpace`가 있으면 일반 OOXML 차트
  경로로 그린다. 미리보기는 값·레이블의 정본이 아니며, 레거시 `Contents`는 별도
  파서의 지원 범위로 분류한다. 차트 변경은 값축·범주·계열·백분율·데이터 레이블과
  차트 뒤 캡션 위치를 한컴 PDF에서 확인하고, 값 변경 후 낡은 미리보기 반례를 검사한다.
  상세 분류는 [차트 OLE v1 경계](mydocs/tech/chart_ole_v1_boundary.md)를 따른다.
  검증 대상 TSV의 한 페이지라도 **90% 미만** 또는 측정 불가이면 작성자가 자기 branch에서
  원인을 재검토·수정하고 새 head로 재실행한다. **정확히 90%는 통과**한다.
  해결 불가능한 실제 글꼴 문제는 [PR 제출 예외 계약](mydocs/manual/verification/visual_sweep_guide.md#해결-불가능한-글꼴의-pr-제출-예외)의
  JSON 증거를 `--font-mismatch-evidence`로 기록해 `font_mismatch_exception`으로 90% 미만이어도 제출할 수 있다.
  측정 불가·쪽수 불일치·배치 차이는 면제하지 않는다.
  PDF와 rhwp의 표 괘선·문단 시작·그림 경계를 같은 좌표계에서 대조하고 위치 차이는 먼저 고친다(#7359 p14).
  `RHWP_FONT_PATH`의 모든 경로가 존재하고 입력 face를 공급하는지 확인한다. 존재하지 않는 과거 font
  경로의 fallback은 예외가 아니라 올바른 글꼴 공급으로 재실행할 사유다.
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
