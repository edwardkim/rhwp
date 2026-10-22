---
kind: canonical
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-10-23
---

# Documentation And Git Workflow

> 이 문서는 문서·Git 작업의 공통 절차를 다룬다. PR 검토·merge·후속 처리의 역할별 규칙은
> [PR 리뷰·통합 워크플로우](../pr_review_workflow.md)와 그
> [조건별 자식 가이드 선택표](../pr_review/README.md)를 우선한다. 현재 세션이나 종료된 작업의 상태는
> 이 문서에 기록하지 않는다.

## Document Language

모든 프로젝트 문서는 한국어로 작성한다.

- 한국어 문장은 맞춤법과 띄어쓰기를 지킨다. 작업 계획·진행 기록·검증 결과도 같은 기준을
  적용하며, 작성 속도를 이유로 여러 단어를 붙여 쓰지 않는다.
- 한국어와 영문 용어·수치가 이어지는 문장은 단어 경계와 조사를 구분해 읽기 쉽게 쓴다.
  예: `같은 글꼴의 Native 전후 대조`, `기존 39개 검사`, `37 PASS / 2 FAIL`.
- 띄어쓰기를 정리할 때 코드 블록·인라인 코드·식별자·경로·URL·SHA·명령·수치는 원문을
  보존한다. 공백 교정에 기술 내용이나 검증 판정 변경을 섞지 않고, 변경 후 원문과 대조한다.

## 임시 산출물과 기준 PDF의 저장 위치

- 작업용 임시 PDF·PNG·TSV·JSON·로그 등 중간 산출물은 처음부터 Git에서 제외된
  `output/` 아래에 생성한다. PR 검토 자료는 `output/pr-review/<id>/`를 사용한다.
  저장소 루트나 정식 자료 경로에 임시 파일을 만들어 Git의 untracked 목록에 쌓지 않는다.
- 기존 기준 PDF가 없어 검증 자료로 **커밋해야 하는 PDF**는 임시 산출물로 취급하지 않는다.
  처음부터 `pdf/<issue-or-topic>/` 등 해당 작업의 정식 자료 경로에 생성하고,
  대응 원본·출처·SHA와 함께 제출한다. 이 PDF를 `output/`에만 두고 검증을 완료 처리하지 않는다.
- 이미 정식 자료 경로에 있는 파일은 임시 산출물인지 필수 제출 자료인지 확인한 뒤 처리한다.
  필수 기준 PDF를 untracked 목록에서 숨기기 위해 `output/`으로 옮기거나 ignore하지 않는다.
  50 MiB 이상 PDF는 기존 [대용량 PDF 로컬 보관](../pr_review/intake_and_review.md#28-검증-입력-커밋-확인)
  예외 계약을 따른다.

## Working Document Naming

단계별 작업 문서:

```text
mydocs/working/task_m100_{issue}_stage{N}.md
```

예:

```text
mydocs/working/task_m100_854_stage1.md
mydocs/working/task_m100_854_rebuild_stage4.md
```

최종 보고서:

```text
mydocs/report/task_m100_{issue}_report.md
```

오늘할일:

```text
mydocs/orders/YYYYMMDD.md
```

회차형 측정 기록:

```text
mydocs/report/{주제}_{회차}_{YYYYMMDD}.md
```

서베이·벤치마크처럼 동일 축을 반복 측정해 시계열로 비교하는 문서는 이슈 1:1 대응이
아니므로 `task_m100_{issue}_report.md` 를 적용하지 않는다. 회차와 날짜가 식별자다.
예: `survey_10k_r18_20260721.md`. 한 이슈로 몰면 회차끼리 이름이 충돌하고 시계열
비교라는 목적이 사라진다.

파일명의 숫자는 **이슈 번호**다. PR 번호를 쓰지 않는다 — GitHub 는 이슈와 PR 이 번호
공간을 공유해 번호만으로는 종류를 판별할 수 없으므로(`gh issue view <PR번호>` 가 PR 을
반환한다), 보고서 본문에 `Issue: #N` 을 명시해 근거를 남긴다. (#2753)

## Folder Roles

- `mydocs/orders/`: 당월 오늘할일
- `mydocs/orders/archives/`: 이전 달 오늘할일 보관
- `mydocs/plans/`: 당월 수행 계획서, 구현 계획서
- `mydocs/plans/archives/`: 이전 달 또는 역할별 절차에서 먼저 정리한 완료 계획서 보관
- `mydocs/pr/`: 당월 PR 검토 증적
- `mydocs/pr/archives/`: 이전 달 또는 역할별 PR 후속 절차에서 먼저 정리한 검토 증적 보관
- `mydocs/working/`: 당월 단계별 완료 보고서
- `mydocs/working/archives/`: 이전 달 또는 역할별 후속 절차에서 먼저 정리한 단계별 보고서 보관
- `mydocs/report/`: 당월 최종 결과보고서
- `mydocs/report/archives/`: 이전 달 또는 역할별 후속 절차에서 먼저 정리한 최종 보고서 보관
- `mydocs/feedback/`: 작업지시자 피드백, 코드 리뷰 의견
- `mydocs/troubleshootings/`: 재발 방지용 문제 해결 기록
- `mydocs/tech/`: 기술 조사와 스펙 정리
- `mydocs/manual/`: 매뉴얼과 장기 지침
- `mydocs/manual/memory/`: 과거 사용자 피드백과 프로젝트 memory의 historical 출처
- `mydocs/manual/codex/`: Codex 부트스트랩과 현행 문서·Git 절차. 종료 세션 자료는 `archive/`에 보존
- 저장소 루트 `docs/`는 사용하지 않는다. 장기 기술 문서는 `mydocs/tech/`, 운영 절차는
  `mydocs/manual/`, 타스크 증적은 위 생명주기별 경로에 둔다.

## Monthly Archive Governance

### 목적과 의미

`orders`, `plans`, `pr`, `report`, `working`의 직접 하위는 최근 문서를 빠르게 찾기 위한 **당월
작업 집합**이다. 각 `archives/`는 완료·폐기 상태가 아니라 작업 집합에서 빠진 기록의 **보관
위치**다. 따라서 archive 이동만으로 GitHub 이슈·PR을 닫거나 문서의 의미를 `historical`로
바꾸지 않는다.

역할별 절차가 merge·review 후 같은 달 문서를 먼저 archive로 옮기는 기존 규칙은 유지한다.
월별 정리는 그 절차에서 옮기지 않은 문서도 다음 달까지 root에 무한 누적되지 않게 하는 상한이다.

### 시점과 대상 판정

1. 매월 첫 유지보수 구간에 전월까지 생성된 문서를 정리한다. 경계는 `Asia/Seoul` 기준 당월
   1일 00:00 미만이다.
2. 다섯 대상 디렉터리의 **직접 하위 `*.md`만** 검사한다. assets·evidence·주제별 중첩
   디렉터리는 월별 일괄 이동 대상이 아니다.
3. 생성일은 filesystem mtime이 아니라 Git에서 그 경로가 최초 도입된 commit의 author
   timestamp로 판정한다. clone·checkout 시각은 근거로 쓰지 않는다.
4. `git log --diff-filter=A`로 추가 commit이 나오지 않는 merge 유입 경로는 전체 이력에서 그
   경로가 처음 존재한 commit을 근거로 삼는다. Git 근거가 없는 파일은 자동 추정하지 않고 예외로
   보고한다.
5. 이전 달에 생성된 열린 이슈·PR의 문서도 이동한다. 후속 작업은 archive 경로에서 계속하며,
   이 이동을 근거로 이슈·PR 상태를 바꾸지 않는다.

### 충돌과 링크 보존

- 목적지 파일이 없으면 basename을 유지해 같은 역할의 `archives/`로 이동한다.
- 목적지가 있고 SHA-256이 같으면 기존 archive를 보존하고 root의 byte-identical 중복만 제거한다.
- 목적지가 있지만 내용이 다르면 **절대 덮어쓰지 않는다**. root 문서는
  `<stem>_archived_<YYYYMMDD>_<blob7>.md`로 옮기고 두 경로·hash를 충돌 원장에 기록한다.
- 저장소 내부 Markdown 링크는 이전 source 위치에서 해석한 실제 target을 기준으로 새 상대 경로를
  계산하여 이동과 같은 commit에서 갱신한다. 단순 문자열 치환으로 같은 basename의 다른 문서를
  연결하지 않는다.
- 이동 전 경로를 파일 계약으로 여는 test·tool·generator·fixture도 저장소 전체에서 찾는다. 완성된
  `mydocs/working/<파일>.md` 문자열뿐 아니라 `.join("mydocs").join("working")`이나
  `Path / "mydocs" / "working"`처럼 경로 조각을 나눠 조립하는 코드도 별도로 검색한다. 실제 파일을
  읽거나 생성하는 경로는 새 archive 위치로 같은 batch에서 갱신하고 해당 focused test를 실행한다.
  과거 실행 증적의 인용 문자열과 이동 문서 자체의 historical `canonical` 값은 실행 경로와
  구분하며 일괄 치환하지 않는다.
- 대량 월별 이동에는 root를 다시 채우는 redirect stub을 만들지 않는다. 현재 branch 경로를 쓰는
  중요한 외부 링크가 확인되면 해당 링크나 별도 canonical index를 정정한다.

### 탐색 규칙

- 당월 작업을 훑을 때는 각 root의 직접 하위 문서를 우선한다.
- 특정 이슈·PR, 과거 결정, 원인 계보를 조사할 때는 root와 `archives/`를 함께 검색한다.
- 파일이 root에 없다는 사실을 작업 미수행·완료·삭제의 근거로 사용하지 않는다. GitHub 상태와
  root/archive 양쪽의 증적을 확인한다.

### 월별 감사와 변경 제출

월별 정리는 독립 task와 commit으로 수행하고 다음 증적을 계획·단계 보고서에 남긴다.

1. 기준 branch SHA, 시간 경계, 폴더별 root·후보·유지 수
2. old/new path, 최초 도입 commit/time, 이동 전 SHA-256을 재현할 수 있는 판정 방법
3. 동일 중복과 divergent 충돌 원장
4. 이동 전후 유효·무효 내부 링크 집합과 신규 오류 수
5. cutoff 이전 direct Markdown 잔여 수
6. rename-aware 변경 파일 수와 add/delete로 보수적으로 센 변경 경로 수

GitHub REST의 pull request file 목록은 최대 3,000개이므로, CI가 이 목록으로 변경 범위를 분류하는
동안에는 대량 이동 PR을 안전한 크기의 순차 batch로 나눈다. 후속 batch는 앞 batch가 반영된 최신
`upstream/devel`에서 시작한다. batch 때문에 자식 이슈를 만들 필요는 없다.

문서 전용 batch의 기본 검증은 다음과 같다.

```bash
git diff --check
python3 scripts/check_markdown_links.py
python3 scripts/check_document_metadata.py
```

기존 historical 문서 오류가 있으면 단순 exit code만으로 판정하지 않고 이동 전 기준선과 결과 집합을
비교해 **신규 오류 0건**을 증명한다. Rust source·test, Cargo, WASM 또는 workflow를 바꾸지 않은
문서 이동에는 해당 빌드 게이트를 추가하지 않는다. 반대로 경로 소비자인 source·test·generator를
고쳤다면 더 이상 문서 전용 batch로 간주하지 않고, 변경 종류에 해당하는 focused test와
[`local_validation.md`](../pr_review/local_validation.md#43-변경-범위별-기본-검증)의 게이트를 추가한다.

## Issue Workflow

이슈 기반 작업의 기본 순서:

1. GitHub Issue 확인 또는 생성 (**신규 등록 전 동일 증상 선행 검색** — 아래). 기존 이슈는
   [해결 PR의 선행 분석](#기존-이슈와-해결-pr의-선행-분석)으로 원인·수정·검증 근거도 확인한다.
2. 열린 PR 확인
3. 이슈 assignee 지정 — **이 단계가 곧 세션 간 잠금이다.** GitHub 에 별도 잠금
   기제가 없어, assignee 가 비어 있으면 병렬로 도는 다른 세션이 같은 이슈를
   합리적으로 "열린 작업"으로 보고 동시에 집는다(실제 재발 사고 기록:
   [병렬 세션 규약](../../tech/autonomous_maintenance/parallel_session_protocol.md)
   §4-2). 조사를 시작하기 **전에** 확인·할당한다:
   ```bash
   gh issue view <n> --repo edwardkim/rhwp --json assignees -q '.assignees[].login'
   gh issue edit <n> --repo edwardkim/rhwp --add-assignee @me   # 권한 있는 계정만 성공
   ```
   `gh issue edit --add-assignee` 가 403 등으로 실패하면(외부 기여자는 보통
   실패한다) 코멘트가 대체 잠금이다:
   ```bash
   gh issue comment <n> --repo edwardkim/rhwp --body "착수합니다 — <범위>"
   ```
   canonical 은 [병렬 세션 규약](../../tech/autonomous_maintenance/parallel_session_protocol.md)
   §5-1 — 이 단계는 그 규약의 요약이지 대체가 아니다.
4. 작업 브랜치 생성 또는 전환
5. 역할별 절차에 따라 오늘할일 또는 PR review 문서 갱신
6. 계획서 작성
7. 작업지시자 승인
8. 구현과 테스트
9. 단계별 보고서 작성
10. 커밋
11. 작업지시자 승인 후 이슈 close

### 사양 우선과 실제 결과 불일치 처리

HWP/HWPML/HWPX의 파싱·변환·조판·렌더링은 **휴리스틱으로 판단하거나 화면 점수에 맞춘
수치로 보완하지 않고 해당 버전 사양을 따른다.** 기여자·reviewer·메인터너·콜라보레이터와
그 대행 자동화에 같은 원칙을 적용한다.

1. 변경 전에 공식 사양의 버전·절·표·페이지와 적용되는 단위·속성·계산 규칙을 확인한다.
   원본에 저장된 값을 추적하고 같은 단위·좌표계로 환산한다. 임의 오프셋·배율·장치 해상도나
   문서별 상수를 독립 근거 없이 추가하지 않는다.
2. 실제 결과가 사양과 다르면 입력 생성 방식, 한컴 버전·출력 경로, 글꼴·DPI·좌표계와
   파서·계산·배치의 실제 소비 경로를 확인한다. 같은 원본의 원시 바이트/속성, 관련 참조 구현,
   독립 한컴 출력과 반례를 교차 대조해 구현 오류·사양 오류·미기술 동작을 구분한다.
   rhwp의 현재 출력이나 단일 화면 점수만으로 사양 오류를 확정하지 않는다.
3. 구현 오류는 공통 로직에서 고친다. **사양 오류가 확인되면 문서 정정도 같은 작업에 포함한다.**
   공식 원문 PDF와 원본 자료는 보존하고, 프로젝트의 사양 설명본·정오표 및 해당 설명을 소비하는
   매뉴얼·코드 주석을 함께 수정한다. 잘못된 원문 위치, 정정 내용, 적용 버전·범위,
   입력/source SHA, 원시 값과 독립 출력·참조 근거를 연결한다.
4. 확인한 정정에 따라 구현하고 수정 전후와 정상 대조군을 실제 재검증한다. 구현 결과에 맞추려고
   사양·기대값을 역으로 수정하지 않는다. 사양에 설명이 없는 동작이나 근거가 부족한 추정은
   확인된 사양 오류와 구분해 기록하고 공식 규칙으로 채택하지 않는다.

HWP5의 확인된 오류는 [사양 정오표](../../tech/hwp_spec_errata.md)에 기록하고
[사양 설명본](../../tech/한글문서파일형식_5.0_revision1.3.md)을 함께 정정한다.
HWP↔HWPX 대응은 [정합 관찰 노트](../../tech/owpml_conformance_observations.md)와
[한컴 공식 모델 참조 가이드](../../tech/hwpx_hancom_reference.md)를 대조한다.
관찰 기록·제3자 참조 구현을 한컴 공식 사양으로 표기하지 않는다.

### 기존 이슈와 해결 PR의 선행 분석

기여자·메인터너·콜라보레이터와 작업 에이전트는 기존 이슈를 조사하거나 관련 회귀 실패를
보정할 때, **코드·테스트 기대값·baseline·golden을 바꾸기 전에 해당 이슈를 해결한 PR과
실제 반영 커밋을 찾아 읽는다.** 리베이스로 반영한 기능의 보존 검증에도 적용한다. 실패가
여러 건이면 조사 대상 모두를 이슈·입력·동작별로 연결하며 대표 한 건의 조사로 대신하지 않는다.

1. **이슈 본문과 전체 논의**에서 원본 입력, 실제 증상, 재현 명령, 기대 결과, 분리한 범위와
   최종 정정을 확인한다. 초기에 기각된 가설과 나중에 확정한 원인을 구분한다.
2. **이슈의 연결 PR·타임라인·종료 코멘트**에서 원 PR을 찾는다. 체리픽·통합·supersede로
   반영됐다면 원 PR뿐 아니라 통합 PR, 실제 merge/cherry-pick SHA, 메인터너 보정과 후속 수정까지
   따라간다. 원 PR이 `CLOSED`라는 사실만으로 미반영이라고 판단하지 않는다.
3. **PR 본문·리뷰·코멘트·diff와 회귀 도입 커밋**을 읽어 당시의 원인 계층, 적용·비적용 조건,
   독립 기대값, 수정 전 FAIL/수정 후 PASS, 한컴 출력·시각 증거 및 미검증 범위를 확인한다.
   테스트 이름이나 번호만 보고 보호하려던 동작을 추정하지 않는다. 이슈가 명시되지 않은 검사는
   `git log --follow -- <파일>`과 도입 커밋의 본문·체리픽 이력으로 원 PR을 추적한다.
4. **현재 코드·입력과 대조**한다. 원 수정의 조건과 소비 경로가 유지되는지, 후속 수정으로 계약이
   달라졌는지, 같은 이름의 fixture가 같은 바이트인지 확인한다. 과거의 변환본 문제와 현재의 원본
   렌더 문제, 합성 계약과 실물 배치, 골든 형식 변화와 실제 출력 변화를 구분한다.
5. **고정한 base와 현재 head에서 재현·판정**한다. 과거 PR의 통과·이슈 종료는 현재 정상의 증거가
   아니다. 조판 영향이 있으면 같은 원본·독립 한컴 Print PDF의 Native/fresh WASM Visual Sweep과
   직접 판독으로 쪽수·내용 소유·누락·중복도 확인한다. 정확한 기존 기대값은 유지하고 구현을 고친다.
   기대값·골든 변경은 독립 근거와 [회귀 기대값 재검토](../pr_review/visual_fixture_evidence.md#기존-회귀-테스트의-기대값-재검토)
   절차를 충족한 경우에만 진행한다.
6. **기존 작업·검토 기록에 근거를 연결**한다. 이슈 → 원 PR → 실제 반영·후속 보정 SHA → 원래
   계약·검증 → 현재 관측·판정의 연결을 짧게 남긴다. 이 조사만을 위한 별도 양식이나 보고서를
   의무로 늘리지 않는다. 찾지 못했거나 읽지 못한 자료는 작업 중 `미검증`으로 구분하고,
   **필수 근거 확보·원인 보정·실제 재검증을 완료한 뒤에만 PR을 제출한다.** 미검증이라고
   기록하는 것만으로 제출 조건을 충족하지 않는다. 승인된 작업 범위의 항목을 미검증·범위 밖·
   별도 이슈로 돌려놓고 해결된 것으로 보고하지 않는다. 필수 항목이 남으면 PR 생성·승인·통합을
   보류하고 해결 작업을 계속한다.
   아직 해결 PR이 없는 이슈는 선행 구현·관련 이슈를 확인하고 새 계약의 독립 근거와 실행 증거를
   확보한다. 존재하지 않는 과거 PR 대신 검증 가능한 근거로 해결을 입증한다.

조사는 `gh issue view --comments`, `gh pr view --comments`, `gh pr diff`, GitHub issue timeline,
저장소의 review·보고서·Git 이력을 함께 사용한다. 이슈 본문이나 검색 결과 한 개만으로 끝내지 않는다.
원 PR의 해결 방식을 참고하되 현재 실패가 같은 원인이라고 단정하거나 과거 임시 보정을 그대로
복원하지 않는다. 선행 분석 후 승인된 구현·검증을 계속하며 같은 작업의 진행 허가를 다시 묻지 않는다.

### 신규 이슈 등록 전 동일 증상 선행 검색

내부에서 결함을 발견해 이슈를 새로 열기 전에, **같은 증상이 이미 외부 리포트로 열려
있는지 먼저 검색한다.** 있으면 새 이슈를 만들지 말고 그 이슈에 원인 분석을 붙이거나,
분리가 필요하면 원 이슈를 명시적으로 참조·연결한다.

```bash
# 증상 문자열·패닉 메시지·오류 코드로 열린 이슈 검색 (닫힌 것도 함께 보려면 state 제거)
gh search issues --repo edwardkim/rhwp --state open "<증상 키워드>"
gh search issues --repo edwardkim/rhwp "panicked at <파일명>"
```

**Why:** 다운스트림이 늘면 같은 결함이 **외부는 증상으로, 내부는 원인으로** 각각 등록된다.
연결하지 않으면 내부 이슈만 처리되고 외부 리포터는 방치된다 — 수정이 배포됐는데도 그
사실을 모른 채 우회 조치를 유지하게 된다.

실제 사례: [#2519](https://github.com/edwardkim/rhwp/issues/2519)(외부 사용자, 각주 삽입
패닉, 2026-07-20)와 [#3214](https://github.com/edwardkim/rhwp/issues/3214)(내부 발견, 같은
원인, 2026-07-23)가 연결되지 않아, `597dabf07` 로 수정된 뒤에도 리포터는 11일간 응답을
받지 못했고 배포에서 메뉴 세 개를 감춘 채 운영했다.

이 사례는 검색으로 **잡혔을 것이다** — `gh search issues --repo edwardkim/rhwp "panicked at
note.rs"` 한 번이면 두 이슈가 나란히 나온다(2026-07-31 실측). 비용은 명령 한 줄이다.

**How to apply:**

- 내부 이슈를 새로 열 때 증상 키워드로 최소 1회 검색한다. 패닉 메시지·오류 문자열은
  외부 리포트에 원문 그대로 실리는 경우가 많아 검색어로 효과적이다.
- 이미 외부 이슈가 있으면 **그 이슈를 주 트랙으로 삼는다.** 원인 분석은 코멘트로 붙이고,
  범위가 달라 분리가 필요할 때만 새 이슈를 열되 양쪽을 상호 참조한다.
- 수정이 merge되면 외부 리포트에 **해결 사실·적용 버전·확인 방법**을 회신한다.
  auto-close 로 닫히더라도 외부 리포터에게는 별도 설명이 필요하다.

## GitHub CLI Usage

GitHub Actions, branch protection, repository permission, cache, runner, workflow 실행·복구를 다루는 작업은
먼저 [GitHub 저장소 운영 매뉴얼](../github_operations.md)에서 변경 등급과 비례 검증 범위를 선택한다.
이 절은 일반 GitHub CLI 변경 경로만 설명한다.

GitHub connector가 읽기는 가능하지만 mutation 권한 부족으로 403을 반환할 수 있다.
이슈 assignee 지정, 이슈/PR metadata 수정, 코멘트 작성 등 GitHub 변경 작업은
로컬 인증된 `gh` CLI를 사용한다.

예:

```bash
gh issue edit 1063 --add-assignee edwardkim -R edwardkim/rhwp
```

운영 규칙:

- connector mutation이 403으로 실패하면 `gh` CLI로 재시도한다.
- sandbox 네트워크 제한으로 `api.github.com` 연결 실패가 나면 동일 `gh` 명령을 escalation으로 재시도한다.
- `gh`로 수행한 GitHub 변경은 오늘할일, 계획서, 보고서 중 관련 문서에 기록한다.
- `gh` 사용도 하이퍼-워터폴 절차를 대체하지 않는다. 이슈 확인, 브랜치, 문서, 승인 게이트는 그대로 유지한다.
- Windows PowerShell에서 한글 다단락 PR 본문을 게시·수정할 때는 pipe가 아닌 UTF-8 without BOM
  `--body-file` 경로와 게시 후 API 검증을 사용한다. 중복 명령은 두지 않으며, 정본 절차는
  [PR 리뷰·통합 워크플로의 Windows 본문 전송](../pr_review_workflow.md#341-windows-powershell-한글-본문)을 따른다.

## PR Workflow

외부 기여자 PR은 내부 task와 다르게 처리한다.

문서 위치:

```text
mydocs/pr/
```

파일명:

```text
pr_{number}_review.md
pr_{number}_review_impl.md
pr_{number}_report.md
```

PR 댓글 톤은 과장하지 않는다. "정말 감사합니다", "정성스러운 PR" 같은 반복적이고 과한 표현보다 사실 중심으로 쓴다.

## 새 PR 생성 전에 확인할 항목

**새 PR 작업보다 본인이 이미 제출한 미병합 PR의 실패·충돌 해소를 우선한다. 특히 lint 오류가
남은 기존 PR을 두고 신규 PR을 생성하지 않는다. 단, 이미 리뷰어가 지정된 PR은 이 우선 처리
규칙의 예외다.** 먼저 리뷰어 지정 여부를 구분한 뒤 나머지 PR의 실패·충돌을 처리한다.

### 1. 본인의 열린 PR 조회와 예외 분류

1. 새 PR 준비를 시작할 때와 실제 생성 직전에 현재 GitHub 인증 계정의 열린 PR을 조회한다.
   모든 base와 draft를 포함한다. 조회 실패·상태 미확인을 "기존 PR 없음"으로 처리하지 않는다.
2. 각 PR의 리뷰어 지정 여부, 최신 head/base SHA, `mergeable`·`mergeStateStatus`, required
   check와 CI 결과를 확인한다. 실패가 있으면 실제 job·step·로그에서 원인을 확인한다.
3. **이미 개인 또는 팀 리뷰어가 지정된 PR은 자동 우선 처리 대상에서 제외한다.** 해당 PR에
   lint 실패·충돌이 있더라도 임의로 rebase·보정 커밋·push하지 않으며, 그 PR만을 이유로
   신규 PR 생성을 막지 않는다. 리뷰 제출 후 `requested_reviewers`가 비는 경우도 있으므로
   기존 리뷰 기록과 지정 이력을 함께 확인하고, 예외를 만들려고 리뷰어를 추가·제거하지 않는다.
   작성자의 self-review는 리뷰어 지정으로 세지 않는다.
4. 메인터너가 머지를 보류하고 보정을 요청했다면
   [리뷰어 지정 후 커밋 규칙](../pr_review/intake_and_review.md#211-리뷰어-지정-후-추가-커밋-제한)의
   요청 범위 예외를 적용해 기존 PR을 보정한다. 리뷰어 지정 자체와 보정 요청은 구분한다.
5. PR 번호·리뷰어 예외 여부·차단 사유·처리 순서를 기존 작업 기록에 짧게 남긴다.

```bash
gh api user --jq .login
gh pr list --repo edwardkim/rhwp --state open --author @me --limit 100 \
  --json number,url,baseRefName,headRefName,headRefOid,isDraft,reviewRequests,mergeable,mergeStateStatus,statusCheckRollup
```

조회 상한에 도달하면 페이지를 이어서 확인한다. 목록의 요약 상태만으로 리뷰어 지정 이력이나
실패·충돌 원인을 단정하지 않고 각 PR의 최신 결과를 확인한다.

### 2. 예외를 제외한 기존 PR의 처리 순서

| 기존 PR의 현재 상태 | 새 PR보다 먼저 수행할 작업 |
| --- | --- |
| lint 실패: fmt, native·WASM32·workspace Clippy 또는 해당 frontend lint/typecheck | 최우선으로 기존 PR branch에서 원인을 수정·커밋한다. 변경 범위의 필수 lint 전체와 필요한 회귀를 다시 실행하고 최신 head CI의 성공을 확인한다. 다른 새 PR에 수정만 넣어 기존 실패를 남기지 않는다. |
| 충돌 또는 오래된 base 때문에 병합 불가 | 최신 `upstream/devel`로 기존 PR branch를 rebase한다. 양쪽 변경을 보존해 충돌·호환성 문제를 해결하고 커밋·재검증한다. |
| 그 밖의 필수 CI 실패 또는 메인터너 보정 요청 | 기존 PR에서 원인을 보정·커밋하고 새 head의 필수 검증과 CI를 완료한다. |
| CI 실행 중 또는 병합 가능 상태 계산 중 | 완료 후 실패·충돌 여부를 판정한다. 대기 상태를 성공으로 간주하지 않는다. |
| CI 성공·충돌 없음, 승인 등 외부 조건만 대기 | 실패·충돌 차단 사유가 없음을 기록하고 새 PR 준비를 진행할 수 있다. 단순 devel 전진만으로 반복 rebase하지 않는다. |

lint 실패와 충돌이 함께 있으면 해당 PR 안에서 **rebase·충돌 해소 → lint 원인 보정 → 커밋 →
필수 검증 → 최신 head CI 확인** 순서로 처리한다. lint 실패가 있는 PR의 최우선 순위는 유지한다.

### 3. 기존 PR 갱신과 새 PR 재개 조건

1. 대상 branch/worktree의 소유·실행 중 작업·미커밋 변경을 확인한다. 다른 세션이 사용 중이면
   임의로 rebase·reset·force-push하지 않고 조율한다. 다른 작업의 변경과 산출물은 보존한다.
2. rebase 전 최신 원격 head SHA를 기록하고 `git fetch upstream devel`로 기준을 갱신해 SHA를
   고정한다. 실제 PR의 head repository·branch를 확인한 뒤 최신 `upstream/devel`로 rebase한다.
3. 충돌 해소는 `git rebase --continue`로 기존 커밋에 반영하고, 추가 보정은 별도 커밋으로 남긴다.
   빈 커밋으로 CI만 다시 유발하지 않는다.
   [리베이스 후 기능 보존](../pr_review/local_validation.md#리베이스로-반영한-pr의-기능-보존-검증)에 따라
   현재 수정과 새 base에서 반영한 기존 기능을 함께 검증한다.
4. 승인된 원격 갱신은 **기존 PR의 같은 head branch**에 push한다. 이력 재작성은 확인한 원격
   SHA를 지정한 `--force-with-lease`로 보호한다. 원격이 전진했으면 다른 변경부터 확인한다.
   변경 전후 head·base SHA, 보정 범위, 로컬 검증과 CI 결과를 기존 PR 설명 또는 작업 기록에 남긴다.
5. 예외를 제외한 기존 PR의 lint·필수 CI 실패와 충돌이 해소됐는지 다시 조회한 뒤 신규 PR을
   생성한다. 이전 head의 성공을 새 head의 성공으로 표시하지 않는다. 선행 처리가 막혔으면
   사유를 보고하고 신규 PR 생성을 보류한다. 작업지시자가 순서를 명시적으로 바꾼 경우에는
   그 지시와 남은 PR 상태를 기록하고 진행한다.

이 규칙은 작업 우선순위이며 push·PR 생성·merge의 승인을 새로 부여하지 않는다. 이미 승인된
범위는 반복 확인하지 않고, 승인되지 않은 원격 작업은 작업지시자 승인을 받은 뒤 진행한다.
기존 PR의 실제 merge에도 별도의 merge 승인과 해당 절차를 적용한다.

## Internal Task PR Approval

내부 타스크 브랜치에서 PR은 작업지시자 별도 승인 후에만 생성한다.

- "PR 준비"는 해당 변경 범위의 [로컬 검증 4.3](../pr_review/local_validation.md#43-변경-범위별-기본-검증)에
  지정된 **전체 로컬 회귀 게이트를 실제로 실행·판정**하고, 커밋·검증 기록·PR 본문 초안·생성 명령을
  준비하는 지시다. 본문 초안이나 review 문서만 만들고 필수 Cargo/npm/WASM 검증을 미루면 PR 준비가
  완료된 것이 아니다. 이 지시는 해당 검증 명령 실행 승인도 포함하지만 remote push와 PR 생성 승인은
  포함하지 않는다.
- 작업지시자가 검증을 명시적으로 축소·생략한 경우에만 그 범위와 사유를 PR 본문 초안에 남긴다.
- `gh pr create` 실행(Open 또는 Draft PR 생성)과 Draft의 Ready 전환은 각각 별도 승인을 받은 뒤
  진행한다.
- PR 번호는 원격 head branch를 push한 뒤 GitHub에서 PR 생성이 성공할 때 채번된다. Issue와
  PR은 같은 번호 공간을 쓰지만, 아직 생성하지 않은 PR 번호를 예측해 `pr_N_*` 파일명으로
  사용하지 않는다.
- PR 제출 전에 [로컬 사전 검증](../pr_review/local_validation.md) 4.3과 해당 보조 절차를 빠짐없이
  완료한다. 일반 제출은 검증을 마친 Open PR이며, 특별한 사유 없는 Draft 생성은 금지한다.
  Draft는 미완성 설계·WIP의 조기 논의 목적을 작업지시자가 명시적으로 승인한 경우에만 쓴다.
  일반 PR 생성 승인, 번호 확보·CI 시작·대기나 필수 검증 실패·미완료는 예외 사유가 아니다.
  예외 Draft도 목적·미완료 항목·완료 계획을 적고 Ready 전환 전에 필수 검증을 모두 완료한다.
- collaborator self PR은 PR 생성으로 번호 `N`을 받은 뒤 **정확한 code head의 GitHub Actions CI
  성공을 먼저 확인**한다. 그 뒤 `pr_N_review.md`와 필요한 review_impl·오늘할일에 code SHA,
  CI run URL과 실제 결과를 기록하고 같은 PR branch의 문서-only trailing commit으로 push한다.
  push 전 병합·링크 검증과 새 trailing head의 required check도 확인한다. 진행 중·실패·취소·미실행
  CI를 완료로 기록하거나 로컬 검증으로 대체하지 않는다. 상세 순서는
  [collaborator self PR](../pr_review/collaborator_self_merge.md#821-pr-채번과-오늘할일-생성갱신-시점)을 따른다.
- 실수로 승인 없이 PR을 열었으면 작업지시자 지시에 따라 즉시 close하고, 후속 진행은 승인 대기 상태로 되돌린다.
- PR 직전의 긴 전체 회귀·lint도 `local_validation.md` 4.3의 필수 범위이면 위 `PR 준비` 지시에
  포함된 실행 승인으로 완료한다. 이미 승인된 검증을 다시 묻거나 focused test만으로 대체하지 않는다.
  새 범위의 검증이나 원격 작업은 기존 사용자 지시의 승인 범위를 확인한다.
- PR 생성·CI 모니터링·merge·후속 처리는 세션에서 실제로 승인된 범위에 따라 수행한다. 이미
  명시적으로 승인된 작업은 반복 확인하지 않으며, 로컬 검증 지시만으로 원격 승인까지 추정하지 않는다.

## Commit Rules

- 구현 보고서는 task 브랜치에서 해당 단계의 소스 변경과 함께 커밋한다. collaborator self PR의
  PR review·오늘할일은 code head CI 성공 뒤 별도의 문서-only trailing commit으로 기록한다.
- merge 전에는 `git status`를 확인한다.
- 이슈 close 전에는 정정 commit이 `devel` 또는 대상 브랜치에 실제 포함되어 있는지 확인한다.
- 사용자가 만들었을 수 있는 변경은 임의로 되돌리지 않는다.

## Branch And PR Rule

- 이슈 작업 브랜치는 **`task/<이슈번호>-<영문 설명>` 형식을 사용한다.** 기여자·메인터너·
  콜라보레이터와 작업 에이전트에 동일하게 적용한다. 예: `task/8153-handbook-raw100`.
  이슈 번호는 `#` 없이 적고, 설명은 소문자 영문·숫자와 하이픈으로 간결하게 작성한다.
  이슈 번호를 생략하거나 날짜를 대신 붙인 이름, `fix/`·`wip/` 등 임의 접두사로 만들지 않는다.
  이슈 작업 착수 전에 번호를 확인하고, 계획서에도 실제 브랜치명을 기록한다.
  외부 PR 검토·체리픽 통합은 아래 명명 규칙과 해당 역할별 PR 검토 절차를 따른다.
- 로컬 작업과 검증의 기준은 최신 `upstream/devel`이다.
- 리베이스 후에는 현재 수정과 새 base에서 반영한 PR의 기능 유지·개선을 함께 검증한다.
  [기능 보존 절차](../pr_review/local_validation.md#리베이스로-반영한-pr의-기능-보존-검증)에 따라
  후퇴하면 보정·재검증하고, 후퇴·필수 증거 부족이 남으면 PR 진행을 보류한다.
- 일반 변경은 작업 브랜치에서 검증한 뒤 `devel` 대상 PR로 통합한다. `upstream/devel`에 직접 push하지 않는다.
- collaborator·maintainer가 원 PR에 보정하거나 merge 후 운영 기록을 반영하는 경우에도
  [PR 리뷰·통합 워크플로우](../pr_review_workflow.md)의 선택표가 지정한 역할별 경로를 따른다.
- `local/*`은 로컬 작업 이름일 뿐 원격 `devel`을 갱신하는 명령의 근거가 아니다.

### 외부 PR 검토와 체리픽 통합 브랜치 명명

외부 PR을 단독 검토하거나 여러 PR을 체리픽해 하나의 통합 PR로 제출할 때는 다음 형식을 사용한다.
로컬 검토 브랜치와 원본 저장소에 새로 push하는 통합 PR head에 같은 이름을 사용한다.

| 작업 | 브랜치 형식 | 예시 |
| --- | --- | --- |
| 외부 PR 한 건 검토·체리픽·메인터너 보정 | `review/pr-<원 PR번호>-<영문 설명>` | `review/pr-7965-table-layout` |
| 대표 작업 이슈가 있는 여러 PR 체리픽 통합 | `review/integration-<대표 이슈번호>-<영문 설명>` | `review/integration-8153-handbook-layout` |
| 대표 작업 이슈가 없는 여러 PR 체리픽 통합 | `review/integration-<YYYYMMDD>-<영문 설명>` | `review/integration-20261022-planet6897-layout` |

- 설명은 소문자 영문·숫자와 하이픈으로 작성한다. 여러 기여자를 묶으면 기여자명 대신 공통 변경 주제를 쓴다.
- 식별자는 이미 존재하는 원 PR·대표 작업 이슈를 사용한다. 통합 PR은 생성 전 번호가 없으므로
  예상 PR 번호를 브랜치명에 쓰거나, 명명만을 위한 이슈를 새로 만들지 않는다.
- 여러 PR의 경우 대표 이슈가 있으면 이슈 번호 형식을 우선한다. 대표 이슈가 없을 때만 작업 시작일을
  사용하며, 리베이스·작업일 변경·추가 체리픽 때문에 같은 작업의 브랜치를 계속 바꾸지 않는다.
- 원 PR 번호·저장소·고정한 head SHA·실제 체리픽한 commit SHA와 순서·메인터너 보정 SHA를
  계획서와 역할별 검토 기록에 남긴다. 브랜치명이나 체리픽 여부만으로 원 PR의 해결을 판정하지 않는다.
- 기존 외부 PR의 원격 head 이름은 유지한다. 직접 보정 경로에서는 로컬 검토 브랜치에서 원격 head를
  명시해 push하며, 이 규칙을 이유로 기여자 원격 브랜치를 변경하지 않는다.
- 새 head를 위한 별도 검토 브랜치가 꼭 필요한 경우에는 기존 브랜치를 보존하고 `-r2`, `-r3`을 붙인다.
  이전·새 head SHA와 분리 사유를 기록하며, 이름이 같다는 이유로 기존 브랜치를 강제로 덮어쓰지 않는다.
- 이 규칙은 신규 브랜치 생성에 적용한다. 이미 열린 PR의 head를 명명만을 위해 변경하지 않는다.
