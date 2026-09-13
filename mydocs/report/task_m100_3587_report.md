# #3587 최종 결과보고서 — 템플릿 복제·채우기와 문서 간 가져오기

- Issue: #3587 — https://github.com/edwardkim/rhwp/issues/3587
- 작성: 2026-09-13, 브랜치 `task_m100_3587`.
- 통합 검증 기준 HEAD: `6a8aeb9ff` (상세 명령·결과는 Stage 25).
- 이전 D3 제품·테스트 검증 기준: `9acd8d54fd04477ee819238ee63abd4eb7676380`.
- 상태: **승인된 A/B/C/D 기능, D3 자동 검증 및 선택적 Gym 시나리오 완료.
  증적 보존과 최신 devel 통합 완료, [Stage 25](../working/task_m100_3587_stage25.md)에서
  통합 후보 최종 검증 완료. 로컬 제출 준비 완료, 원격 push·Open PR 승인 대기.**
- 기준 계획: [수행계획](../plans/task_m100_3587.md),
  [구현계획](../plans/task_m100_3587_impl.md),
  [공개 사용법](../manual/template_automation.md).

## 1. 무엇을 사용할 수 있는가

템플릿의 표·문단 묶음을 필요한 건수만큼 복사하고, 복사본마다 다른 값을 채우거나,
다른 문서의 블록을 가져오는 공통 문서 코어 기능을 구현했다.
기존 영역을 지우거나 빈 문단을 압축해서 화면을 맞추는 후처리가 아니다.

| 구간 | 구현 결과 | 완료 근거 |
| --- | --- | --- |
| A: 식별자·참조 보존 | 소유 구조를 순회하여 복사본 식별자를 새로 할당하고 참조를 연결 | [Stage 2](../working/task_m100_3587_stage2.md), [Stage 3](../working/task_m100_3587_stage3.md) |
| B: 같은 문서 복제 | 완결 문단 블록·표 행 묶음 반복, 삽입 전후 영역 보존 | [Stage 7](../working/task_m100_3587_stage7.md), [B 종료 검증](../working/task_m100_3587_stage8.md) |
| C: 내용 채우기 | 고정 양식 채우기, 문단/행 복제와 복사본별 채우기, 공통 공개 호출 | [Stage 11](../working/task_m100_3587_stage11.md), [C 종료 검증](../working/task_m100_3587_stage14.md) |
| D: 다른 문서 가져오기 | 서식·그림 자원과 참조 이식, 대상 경계 삽입, 별도 원본 핸들/파일 호출 | [Stage 18](../working/task_m100_3587_stage18.md), [Stage 20](../working/task_m100_3587_stage20.md), [D3](../working/task_m100_3587_stage23.md) |
| 선택적 Gym 소비자 | 연구노트 3건의 채우기→복사→개별 채우기→저장·재열기 및 음성 대조 | [Stage 24](../working/task_m100_3587_stage24.md), 메인테이너 양 형식 한컴 정상 판정 |

외부 실행 action은 `fill_template`, `repeat_and_fill_paragraph_block`,
`repeat_and_fill_table_rows`, `import_paragraph_block`이다.
native DocumentCore, WASM, CLI `run`, MCP `hwp_run_plan`에서 같은 코어를 사용한다.
Gym은 이 API를 이용하는 선택적 평가 도구이며 제품·CI·배포 의존성으로 추가하지 않았다.

## 2. 원 제안과 최종 범위의 관계

원 제안자 yuyu04의 v0.7.13 기반 포크 `f40c12f0`를 먼저 조사했다
([포크 검토](../working/task_m100_3587_fork_review.md)).
이슈의 네 가지 문단/구역 정리 연산을 그대로 이식한 것은 아니다.
메인테이너 결정에 따라 **자동화 문서를 안전하게 구성하는 기반**으로 범위를 확장·수정했다.

- 의도적인 엔터·앞뒤 문단을 자동 삭제하지 않는다. 2pt 압축이나 자동 구역 병합을 도입하지 않는다.
- 같은 문서 복제와 다른 문서 가져오기를 구분하고, 컨트롤 ID·서식·그림 참조를 함께 처리한다.
- 연구노트 원본과 메인테이너가 한컴에서 복사한 자료를 사용했다. 제안자의 원래 템플릿/전체
  시나리오를 그대로 재현했다고 주장하지 않는다. 별도 요청한 원 제안 사례는 수용 범위와 구분한다.
- 원 제안의 네 API 이름 또는 모든 문서 정리 요구가 구현되었다는 의미로 이슈를 닫아서는 안 된다.
  종료 판단은 위 수정 범위의 승인·병합 근거로 한다.

## 3. 보호 계약과 지원 제한

1. 원형·삽입 경계 밖 문단과 대상 용지 설정을 보존한다. source 용지 설정을 대상에 덮어쓰지 않는다.
2. 같은 문서 자원은 재사용하고, 다른 문서는 참조 검증과 자원 대응표를 통해 이식한다.
   raw 참조의 의미를 모르면 버리거나 추측하지 않고 지원 오류로 거부한다.
3. dry-run과 실행은 같은 준비·검증 결과를 사용한다. 복구 가능한 요청 오류는 변경 전에 거부한다.
   프로세스 OOM이나 외부 파일 교체 경합까지 원자적으로 복구한다는 보장은 아니다.
4. 복제는 반복 실행할 때 다시 추가되는 연산이다. SHA precondition은 입력 버전 보호이지
   영구 요청 중복 제거가 아니다. 주소는 0 기준 구조 경로이며 페이지 번호가 아니다.
5. 기본 글상자·일반 하이퍼링크 보존과 값 채우기 대상은 구분한다. 필드 채우기는 ClickHere 범위이며,
   알 수 없는 글상자 확장·필드 매개변수·구역 경계와 행 경계를 가로지르는 병합은 명시적으로 거부한다.
6. WASM 다른 문서 가져오기는 별도 핸들이 필요하다. 같은 핸들 중복 borrow는 JS 호출 전에 막아야 한다.
7. 크기·깊이·개수 예산을 둔다. CLI source는 64 MiB 이하 일반 파일이며 URL을 내려받지 않는다.
   세부 한도·오류·JSON 계약은 [공개 사용법](../manual/template_automation.md)을 따른다.

### 조판 변경의 근거

자동화만 추가한 PR은 아니다. `src/renderer/typeset.rs`의 복제된 표 흐름 처리도 포함한다.
이전 문단에서 지연된 표를 다음 표 문단/구역 경계와 올바른 순서로 처리하고,
저장된 호스트 위치와 표 continuation 상태를 구분한다. 특정 문서 ID나 엔터 횟수로 분기하지 않는다.
원형과 한컴 복사본, 복제 결과의 관측 및 반례는 [Stage 7](../working/task_m100_3587_stage7.md)에 있다.
`issue_3587_repeated_table_flow` 계약과 기존 전체 회귀로 보호한다.

이 구현이 Studio의 모든 재편집 페이지네이션을 해결한 것은 아니다.
한컴에서는 생성 파일이 정상 편집되고 Studio에서만 남는 증상은 #7065로 분리했다.
자동 테스트·페이지 수만으로 한컴 시각 일치를 판정하지 않는다.

## 4. 검증 결과 — 실행 시점과 제출 후보를 구분

### 최신 devel 통합 후보

[Stage 25](../working/task_m100_3587_stage25.md)의 `6a8aeb9ff` 기준:

| 검사 | 실제 결과 |
| --- | --- |
| fmt·native/WASM32/workspace all-target Clippy·workspace build·manifest·unit tier | 재검증 전부 PASS |
| 집중 검사 | 192 PASS |
| 신규 입력 보안 | 문서 41개 명시 입력, 6 PASS |
| 전체 nextest 1차 | 9,727 PASS / 2 FAIL / 51 skipped, 신규 fixture의 기존 하단 넘침 미등록 |
| 독립 devel 대조 | 원본·복제 HWP·복제 HWPX의 전체 이상 진단 JSON/로그 동일; 신규 두 경로만 등록 |
| 전체 nextest 재실행 | **9,729 PASS / 0 FAIL / 51 skipped** |
| Gym 재실행 | 양 형식 각 13개 검사·음성 대조 통과, 최종 두 파일 SHA가 메인테이너 판정 파일과 동일 |
| Native Skia | root 3,930 PASS/13 ignored, 내부 crate 182 PASS, 그림 2 PASS, 직접 PDF 4 PASS |
| Docker WASM·실제 WASM | 표준 최적화 빌드 PASS, 두 입력 형식의 가져오기·오류·저장/재열기 계약 PASS |

### 이전 D3 검증 이력

아래는 [D3 실행 기록](../working/task_m100_3587_stage23.md)의 제품·테스트 내용 기준이다.
기존 review worktree의 overlay를 바이트 대조했으며 worktree의 과거 HEAD를 검증 SHA로 쓰지 않았다.
후속 문서·Gym 조사 스크립트 변경을 제품 코드 변경으로 계산하지 않는다.

| 검사 | 실제 결과 |
| --- | --- |
| fmt·native/WASM32/workspace all-target Clippy·workspace build | 모두 PASS |
| 전체 nextest | 9,704 PASS / 1 FAIL / 51 skipped. 필드 사전 `steps[].source` 누락 |
| 누락 문서 보완 후 영향 검사 | 23 PASS / 0 FAIL. 제품·테스트 단언 변경 없음 |
| 종합 자동 판정 | 전체 실행과 영향 재검증 합산으로 미해결 실패 0. 단일 9,705 PASS 실행은 아님 |
| Native Skia | root 3,930 PASS / 13 ignored, 내부 crate 182 PASS, 그림 placeholder 2 PASS, PDF 4 PASS |
| Docker WASM | 빌드 PASS, 실제 WASM에서 HWP/파생 HWPX 가져오기·오류·저장·재열기 PASS |
| manifest·unit tier | PASS. 파생 suite·manifest는 source 제출에 포함하지 않음 |
| Gym 시나리오 | 양 형식 각각 13/13 PASS, 무편집/복사 생략/마지막 채우기 생략 모두 거부 |
| Gym 기존 계약·구조 감사 | runner 177 PASS, 감사 `ok=true`, `issueCount=0` |
| 메인테이너 | Stage 24의 HWP/HWPX 모두 한컴 정상 판정 |

전체 nextest 명령은 `cargo nextest run --locked --cargo-profile release-test --target-dir
/home/edward/mygithub/rhwp/target/pr-review --tests --no-fail-fast`다.
run ID는 `1509d858-865e-48db-932c-f7979cbe64e6`, 문서 보완 후 영향 실행은
`e021c466-bcfc-4ae2-992e-079ed47c3cbf`다. 명령별 로그·환경·바이너리 지문은 D3 기록을 따른다.

Gym은 공개 pack 전수 평가나 에이전트의 미지 과제 수행 능력 평가가 아니다.
이 원본의 마지막 문단을 복사했으므로 **삽입 뒤 실제 후행 문단은 없다**.
Gym에서 새로 확인한 앞쪽 보존은 12개 실물 문단이며, 후행 보존 계약은 이전 B/D 테스트에 근거한다.
파생 HWPX·rhwp 재열기는 독립 한컴 정답지로 취급하지 않는다.

## 5. 한컴 수용 증거와 제출 보존 상태

연구노트 원본/한컴 복사본 등 `samples/rnote/` 12개는 현재 commit에 포함되어 있다.
기존 `samples/table-in-tbox.hwp`와 `pdf/table-in-tbox-hwp-2020.pdf`도 재사용한다.
Stage 25에서 최종 검증 자료 73개 출처를 고유 파일 37개로 대응시키고, 기존 3개 재사용·
새 문서 29개/PDF 5개 보존을 완료했다. [MANIFEST](../../samples/issue3587/MANIFEST.json)의
`path`가 정식 경로이고 각 `source`는 기존 로컬 위치다. 실제 파일·commit bytes·SHA를 대조했다.

| 증거 | 현재 위치와 판정 | 제출 상태 |
| --- | --- | --- |
| 제목 표/그림 가져오기 | `samples/issue3587/d-pi2-import.{hwp,hwpx}`, 메인테이너 성공 | 보존 완료, 대응 PDF `pdf/issue3587/d-pi2-import-{hwp,hwpx}-2020.pdf` |
| 큰 글상자 가져오기 | `samples/issue3587/d-pi4-import.{hwp,hwpx}`, 한컴 정상 열림 | 보존 완료, 대응 PDF `pdf/issue3587/d-pi4-import-{hwp,hwpx}-2020.pdf` |
| Gym 최종 연구노트 | `samples/issue3587/gym-{hwp,hwpx}-labnote-filled.*`, 모두 한컴 정상 | 단계별/파생/음성 입력도 MANIFEST에 보존·재사용 대응 |

제목 표·큰 글상자의 파일/PDF SHA, 한컴 runtime·job·원본 페이지 비교는 Stage 18/20에 있다.
큰 글상자의 HWPX PDF에서 바깥 분홍 점선 dash 차이를 관측했으므로 완전한 픽셀 동등성을 주장하지 않는다.
Gym 최종 HWP SHA는 `ed63c8da2595d7f205be32eaf534e200f58969b490dda601ff831925ddc883f4`,
HWPX는 `9cd132770cf3b9ffda967e0e7510ed7dd04c78e0404627f6fdb16ed43375abf8`다.

[fixture 보존 규칙](../manual/pr_review/visual_fixture_evidence.md)에 따라 실제 사용한 입력과
채택한 한컴 PDF를 정식 경로에 보존했다. 신규 검증 입력 게이트는 Stage 25에서 실행한다.
위 표는 대표 목록이며 최종 수용/자동 계약에 사용한 보존 목록은 MANIFEST가 정본이다.
실패한 조사 출력·중간 PNG/SVG/JSON·빌드 로그·output 전체는 제출하지 않는다.

## 6. 성능 영향과 별도 문제

D3의 두 템플릿×두 형식×1/10/100개×3회, 총 36회 계측에서 100개 가져오기는
native 실행 중앙값 약 15~20 ms, CLI 시작·파싱·실행·저장 전체 약 263~379 ms였다.
native 실행은 preview 뒤의 따뜻한 캐시 조건이다. 절대 성능 보장이나 기존 대비 개선율이 아니다.
기준 환경·메모리 최고값·결과 JSON 크기·바이너리 재사용 차이는 D3 표와 제한을 따른다.
**이번 계측을 전체 렌더러의 성능 영향 측정으로 대신하지 않는다.**

메인테이너 결정으로 다음 Studio 증상은 분리했으며 이번 변경의 해결 목록에 포함하지 않는다.

- [#7065](https://github.com/edwardkim/rhwp/issues/7065): 셀 Enter 증가 시 표 페이지네이션·중첩.
- [#7084](https://github.com/edwardkim/rhwp/issues/7084): 이모티콘 너비 축소.
- [#7090](https://github.com/edwardkim/rhwp/issues/7090): 4쪽 표 사이 간격 소실.

## 7. 최신 devel 통합과 남은 제출 순서

2026-09-13 `upstream/devel=1ae5ca295bddcb31b846affc62834a2a3023d24d`의 추가 18개
commit을 `ca67b5ff5`로 작업 브랜치에 통합했다. 충돌 없이 병합했으며,
양쪽에서 바뀐 `typeset.rs` 및 HWP 변환·저장 영향 때문에 전체 검증을 다시 진행한다.
현재 검증 후보는 신규 문서/PDF와 쪽수 5건 및 독립 대조한 넘침 2건 등록을 포함한 `6a8aeb9ff`다.

새 integration source는 `tests/cases/` 21개다. Cargo.toml/lock·generated suite/manifest·
CI workflow는 이번 변경에 포함하지 않는다. baseline 변경은 독립 한컴 PDF와 쪽수를 대조한
신규 `oracle_page_count` 5행과, 수정 없는 devel/후보를 독립 대조해 동일한 기존 넘침임을 확인한
신규 `body_overflow` 2행이다. 기존 문서의 허용치를 완화한 것이 아니다.

1. 최종 입력·PDF 보존과 최신 devel 통합: 완료.
2. 통합 후보 집중 192건·신규 입력 보안 6건·Rust lint·Gym·전체 회귀·Native Skia·Docker WASM:
   완료. 실행별 최종 결과는 Stage 25에 기록했다. 이후에는 문서만 변경했다.
3. 로컬 문서 commit·PR 본문으로 별도 push·Open PR 승인을 요청한다.
   생성 후 PR 번호에 맞는 review 문서·트리야지·CI·self-review·병합 절차를 따른다.

최초 보고서 작성 단계의 로컬 검사(현재 제출 검증은 Stage 25):

- `samples/rnote/` 추적 파일 12개를 `git show HEAD:<path>`와 바이트 대조해 차이 0개를 확인했다.
- Gym 최종 두 파일 SHA를 재계산하여 Stage 24의 메인테이너 판정 대상과 일치함을 확인했다.
- `python3 scripts/check_markdown_links.py mydocs/report/task_m100_3587_report.md
  mydocs/plans/task_m100_3587.md mydocs/plans/task_m100_3587_impl.md`: 3개 문서 이상 없음, exit 0.
- `git diff --check`: PASS. 이번 절차에서는 제품·테스트·기준값을 변경하지 않았다.

현재 원격 push·PR 생성·댓글·merge·이슈 close는 수행하지 않았다.

## 부록: PR 본문 초안 — 원격 제출 승인 대기

- 제목: `feat(template): support safe block copy, fill and cross-document import (#3587)`
- base/head: `devel` / `task_m100_3587`.
- 관련 이슈: `Issue: #3587`. 수정된 범위의 수용·병합 근거로 종료를 판단한다.
- 변경: A 식별자/참조 보존, B 동일 문서 반복, C 내용 채우기, D 자원 이식,
  native/WASM/CLI/MCP 연결 및 선택적 Gym 연구노트 시나리오. 표 흐름 보정도 포함한다.
- 검증: Stage 25의 `6a8aeb9ff` 후보 전체 9,729 PASS·필수 lint·Native Skia·Docker WASM.
  D3 비용 결과는 이전 측정 이력으로 구분한다. 이후 문서 commit과 제품/테스트 동일성을 확인한다.
- 시각: Stage 18/20/24의 판정 범위만 인용하며 정식 보존 문서/PDF를 사용한다.
  Stage 25 자동 비교의 남은 차이를 완전한 시각 일치로 바꾸어 보고하지 않는다.
- 제외: #7065/#7084/#7090, 원 제안 정리 API 그대로의 이식, 전체 Gym benchmark.
- 체크리스트: fixture 보존과 최신 통합 후보 검증 완료. 원본 템플릿 형식의 적용 항목별 실행 근거,
  최초 실패 2건의 독립 대조와 등록 근거, UI E2E/외부 clipping 미실행 범위를
  `output/3587/submission-stage25/pr-body.md`에 준비했다.
- 승인 후 실행 형식(현재 실행하지 않음):
  `gh pr create --repo edwardkim/rhwp --base devel --head task_m100_3587
  --title '<위 제목>' --body-file '<확정한 UTF-8 본문 파일>'`.

## 미주 — 용어

- IR (Intermediate Representation): 파싱한 문서를 조작·저장·조판이 공유하는 중간 구조.
- ID (Identifier): 문단·컨트롤 등의 식별자. 복사본은 독립 식별자를 갖되 내부 참조도 함께 연결해야 한다.
- CAS (Compare-And-Swap): 예상 입력 지문과 실제 파일이 일치할 때만 실행하는 버전 보호 방식.
- dry-run: 변경·파일 저장 없이 같은 요청의 유효성과 예상 결과를 검사하는 호출.
- fixture: 재현에 사용하는 고정 입력 자료. 원본·한컴 기준 출력과 rhwp 파생 자료의 역할을 구분한다.
- 음성 대조: 필요한 작업을 생략한 결과가 실제 검사에서 거부되는지 확인하는 실험.
- WASM (WebAssembly): 브라우저/Node에서 실행하는 바이너리 형식. 빌드 성공과 브라우저 시각 판정은 별개다.
- MCP (Model Context Protocol): 외부 에이전트가 도구를 호출하는 프로토콜. 제품 API의 소비 경로다.
