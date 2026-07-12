# PR 리뷰 · 통합 워크플로우 매뉴얼

**작성일**: 2026-04-23
**대상**: rhwp 메인테이너 (외부 PR 처리 담당) + collaborator self-merge 후보 예외 경로
**교훈 기반**: v0.2.1 사이클 외부 기여자 9명 / PR 10+ 건 통합 경험

---

## 1. 개요

rhwp 는 v0.2.1 사이클부터 외부 기여자 PR 이 급증했다. 하이퍼-워터폴 방법론에 맞춰 **PR 처리도 표준화된 절차** 로 운영한다. 본 매뉴얼의 기본 경로는 외부 PR 도착 시 메인테이너가 따르는 순서를 기록한다.

운영 경로는 다음 세 가지로 구분한다.

- **maintainer 일반 경로**: 원본 저장소의 admin 또는 branch protection bypass 권한을 가진 메인테이너가
  외부 기여자 PR 을 검토, merge, 후속 보고한다. 이 문서의 기본 절차다.
- **collaborator self-merge 후보 예외 경로**: collaborator 가 본인 PR 을 merge 후보로 준비하면서 review 문서를 PR head 에 포함하는 경우다. 이 경로는 8장 조건을 만족할 때만 적용한다.
- **collaborator-mediated 외부 PR 경로**: 외부 contributor PR 을 repository collaborator 가 검토·merge 준비하면서 review 문서를 PR head 에 포함하는 경우다. 이 경로는 9장 조건을 만족할 때만 적용한다.

명령 예시는 원본 저장소 remote 가 `upstream` 인 현재 로컬 checkout 기준이다. 원본 저장소를 `origin` 으로 둔 maintainer checkout 에서는 같은 명령의 remote 이름만 `origin` 으로 치환한다.

이 문서에서 권한 용어는 GitHub repository permission 을 기준으로 구분한다.

- **maintainer**: 원본 저장소의 `admin` 또는 branch protection bypass 권한을 보유해 운영 기록을
  `devel` 에 직접 반영할 수 있는 사용자다.
- **collaborator**: 원본 저장소의 `write` 권한을 보유하지만 branch protection 을 우회하지 않고 PR 로
  변경을 반영하는 사용자다.
- **contributor**: 외부 fork 또는 contributor branch 에서 PR 을 제출하는 사용자다.

메인터너의 `devel` 직접 반영은 merge 된 PR 의 review 문서, 오늘할일, `mydocs/pr/assets/` 아래 검증 자산처럼
코드 동작을 바꾸지 않는 운영 기록에 한정한다. 소스, 테스트, CI workflow, golden/baseline, 기존 샘플 변경은
메인터너 권한이 있어도 PR 과 GitHub Actions 검증을 기본으로 한다. admin/bypass 권한이 없는 사용자는 직책명과
무관하게 collaborator 경로를 따른다.

## 2. PR 도착 시 확인 체크리스트

새 PR 이 열리면 다음을 순서대로 확인한다.

### 2.1 reviewer assign 선행

PR review 처리를 시작하기 전에 먼저 reviewer 를 assign 한다. reviewer assign 이 빠지면 PR 검토 상태가
GitHub 상에서 추적되지 않으므로, 메타 확인·로컬 fetch·문서 작성보다 앞서 수행한다.

```bash
gh pr edit N --repo edwardkim/rhwp --add-reviewer <reviewer>
```

여러 PR 을 체리픽 누적 검토하는 경우에도 각 원 PR 번호별로 reviewer 를 먼저 assign 한 뒤 개별 review
문서를 작성한다.

### 2.2 기본 메타

- [ ] **base 브랜치**: `devel` 이어야 함. `main` 이면 rebase 요청 (이전 재리젝 사례: PR #234)
- [ ] **연관 이슈**: PR description 에 `closes #N` 또는 `#N` 참조가 있어야 함. 없으면 코멘트로 요청
- [ ] **mergeable state**: `MERGEABLE` + `CLEAN` / `BEHIND` / `DIRTY` 중 확인
- [ ] **CI 상태**: `Build & Test` / `CodeQL` 실행 결과

### 2.3 규모 분석

```bash
gh pr view N --repo edwardkim/rhwp --json additions,deletions,files,commits
```

- 규모 파악 → 리뷰 시간 예측
- **대형 PR** (>1000 라인) → 별도 검토 사이클, 바로 머지 불가
- **소형 PR** (<100 라인) → admin merge 고려 가능

### 2.4 작성자 확인

- **FIRST_TIME_CONTRIBUTOR**: 환영 인사 + 세심한 피드백 톤
- **기존 기여자**: 이전 PR 컨텍스트 참고

### 2.5 update branch 이후 이전 SHA CI 강제 취소

PR 검토 중 contributor 또는 maintainer 가 `Update branch` / `devel` merge 로 PR head 를 갱신하면, 이전 head
SHA 에서 시작된 GitHub Actions run 이 그대로 남아 새 head 의 required check 와 섞여 보일 수 있다. 이 경우
다음 순서로 **이전 SHA run 만** 정리한다. 최신 PR head 의 CI 는 취소하지 않는다.

1. 최신 PR head SHA 를 확인한다.

```bash
gh pr view N --repo edwardkim/rhwp --json headRefOid
```

2. 취소 대상이 되는 이전 SHA 의 run 을 확인한다.

```bash
gh run list --repo edwardkim/rhwp --commit <old-sha> \
  --json databaseId,workflowName,status,conclusion,headSha,url --limit 20
```

3. `queued` / `pending` / `in_progress` run 은 일반 cancel 을 시도하지 않고, 처음부터 GitHub Actions
   **force-cancel API** 로 취소한다.

```bash
gh api --method POST repos/edwardkim/rhwp/actions/runs/<run-id>/force-cancel
```

4. 최종 상태가 `status: completed`, `conclusion: cancelled` 인지 확인한다.

```bash
gh run list --repo edwardkim/rhwp --commit <old-sha> \
  --json databaseId,workflowName,status,conclusion,headSha,url --limit 20
```

기록 예시: PR #1738 검토 중 이전 SHA `00bc0428173c9f413171f3b070c923089320d050` 의 CI/CodeQL run 은
처음부터 `force-cancel` API 로 최종 `cancelled` 처리했다.

### 2.6 렌더 영향 · visual sweep 필요 여부 판정

PR 검토 초기에 변경 파일과 PR 설명을 보고 visual sweep 필요 여부를 먼저 판정한다. 이 판정은 cargo 검증을
시작하기 전에 review 문서에 남긴다.

다음 중 하나라도 해당하면 visual sweep 또는 동등한 시각 검증 대상이다.

- `src/renderer/**`, `src/wasm_api.rs`, `rhwp-studio/**` Canvas/render 출력 경로가 바뀐다.
- `typeset`, `layout`, `paint`, pagination, page count, page break, table split, wrap, clipping,
  `host_line_spacing`, margin/spacing 처럼 PDF/SVG 결과가 달라질 수 있는 코드가 바뀐다.
- PR 제목/본문/보고서가 기준 PDF, 한컴 출력, 페이지 수, 라운드트립 render-diff, visual regression 을
  해결했다고 주장한다.
- HWP/HWPX 샘플, 기준 PDF, golden, visual regression fixture 가 추가되거나 갱신된다.

`cargo test --profile release-test --tests`, `cargo clippy`, `cargo test --test svg_snapshot` 통과는 이 판정을
대체하지 않는다. 위 조건에 해당하면 3.5 절과 `mydocs/manual/visual_sweep_guide.md` 에 따라 첨부 기준 PDF
또는 3.5.1 절의 HWP 2020 MCP 산출 PDF 를 사용해 대표 샘플/페이지 시각 검증을 수행한다. PR 작성자가
검증 PDF 를 첨부하지 않았다는 이유만으로 보류 사유를 적고 끝내지 않는다. 먼저 MCP 로 기준 PDF 를 산출하고,
MCP 변환이 실패하거나 원본 HWP/HWPX 가 없어서 산출할 수 없을 때만 PR 작성자 또는 reviewer 에게 한컴
2020/2024 등 실제 기준 프로그램에서 저장한 PDF 업로드를 요청한다.

페이지 수 변화나 시각 검증이 PR 판단 근거인 경우에는 기준 PDF 뿐 아니라 원본 HWP/HWPX 도 검증 자료다.
원본 HWP/HWPX 가 PR diff, 관련 issue 첨부, 또는 기존 `samples/` 어디에도 없으면 review 문서에 재현성
공백으로 기록하고, merge 후 원 PR 코멘트나 후속 검토 의견에 "다음부터 페이지 수/시각 검증이 필요한 PR 은
원본 HWP/HWPX 와 기준 PDF 를 함께 첨부해 달라"는 요청을 남긴다.

## 3. 리뷰 문서 작성

maintainer 일반 경로에서는 각 PR 마다 `pr_{N}_review.md` 를 active 경로에 작성한다. 메인터너 보정,
체리픽 통합, 복잡한 후속 단계처럼 별도 실행 계획이 필요한 경우에만 `pr_{N}_review_impl.md` 를 추가한다.

```text
mydocs/pr/pr_{N}_review.md
mydocs/pr/pr_{N}_review_impl.md   # 필요 시
```

처리 완료 후 7.6 절에서 `mydocs/pr/archives/` 로 이동한다. collaborator self-merge 후보 또는
collaborator-mediated 외부 PR 처럼 처음부터 archive 경로에 작성하는 방식은 8장·9장 예외 경로에서만
사용한다.

단, 원 코드 PR merge 후 별도 후속 기록 fast-pass PR 로 review 문서/asset/오늘할일만 반영할 것이 확정된
경우에는 active 경로에 만들었다가 archive 이동 PR 을 따로 만들지 않는다. 이 경우 review 문서는 처음부터
다음 archive 경로에 작성하거나, 후속 기록 PR 을 만들기 전 같은 커밋 안에서 archive 경로로 정리한다.

```text
mydocs/pr/archives/pr_{N}_review.md
mydocs/pr/archives/pr_{N}_review_impl.md   # 필요 시
```

### 3.1 리뷰 문서 (`pr_N_review.md`)

포함 항목:
- PR 메타 표 (번호 / 작성자 / base / 규모 / mergeable 작성 시점 참고값)
- 관련 이슈 요약
- 변경 범위 분석 (핵심 기능 / 메타 변경 / 범위 외)
- 렌더 영향 여부와 visual sweep 필요 여부 판정
- 사전 검증 결과 (로컬 빌드 / 테스트 / Clippy / doctest)
- 주요 문제점 / 리스크
- 최종 권고 (admin merge / rebase 요청 / 재작업 요청 / close)

예시: `mydocs/pr/archives/pr_234_review.md` (재작업 요청), `mydocs/pr/archives/pr_251_review.md` (admin merge 권고)

### 3.2 구현 계획서 (`pr_N_review_impl.md`)

다음 중 하나에 해당할 때 작성한다.

- contributor 원 변경 위에 maintainer 보정 코드를 추가한다.
- 여러 PR 을 체리픽해 통합하거나 충돌 해결 순서를 관리한다.
- merge, 후속 PR, 이슈 분리 등 작업지시자 선택이 필요한 단계가 둘 이상이다.
- 단순 리뷰 문서만으로 실행 순서와 롤백 범위를 명확히 설명하기 어렵다.

단순·소형 PR 은 `pr_N_review.md` 안에 처리 계획을 포함하고 별도 구현 계획서를 생략할 수 있다.

포함 항목:
- 커밋별 SHA + 제목
- Stage 구성 (승인 → merge → sync → cleanup)
- 작업지시자 확인 필요 사항 (merge 방식, 코멘트 톤, 후속 이슈)

### 3.3 volatile 상태값 기록 규칙

PR review 문서는 merge 후에도 모순되지 않아야 한다. 따라서 다음 값은 확정 사실처럼 기록하지 않는다.

- `draft`
- `mergeable`
- `head SHA`
- `CI 상태`

필요하면 다음 방식으로만 기록한다.

- "문서 작성 시점 참고값: ..."
- "merge 전 최신 상태 확인 필요"
- "최종 merge 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인"

금지 예시:

- `draft: true` 를 현재 상태처럼 단정
- `mergeable: CLEAN` 을 최종 merge 가능 판정처럼 기록
- 특정 `head SHA` 를 "현재 head" 로만 적고 merge 전 재확인 조건을 남기지 않음
- 과거 GitHub Actions 통과 상태를 최신 통과 조건 없이 최종 판정처럼 기록

### 3.4 가설 기각/재분류 PR 판단 규칙

조사 PR 이 어떤 가설을 **기각**하거나 다른 원인 계통으로 **재분류**하는 경우, 그것만으로 merge 보류나
reject 사유로 보지 않는다. PR 의 목적이 "처음 가설대로 고치는 것"이 아니라 "가설이 틀렸음을 증명하고
후속 분류를 확정하는 것"이면, 그 증명 자체가 merge 대상이다.

이 유형의 PR 은 다음 조건을 만족하면 merge 후보로 판단할 수 있다.

- 최종 보고서, stage 문서, README, 샘플 설명이 모두 같은 결론을 가리킨다.
- 초기 가설은 "초기 가설" 또는 "검증 대상"으로만 남기고, 확정 사실처럼 쓰지 않는다.
- 기각/재분류 근거가 샘플, 기준 PDF, 하니스 결과, visual sweep, 로그 중 적절한 자료로 남아 있다.
- 후속 추적 이슈가 필요하면 어떤 이슈로 이관되는지 명확히 적는다.

반대로, 최종 보고서는 기각/재분류를 말하는데 계획서나 README가 초기 가설을 사실처럼 유지하면 merge
보류 사유다. 이때는 PR 작성자에게 수정을 요청하거나, 메인터너 권한으로 문서를 보정한 뒤 "가설 기각이
맞음을 증명하는 PR"로 merge 판단한다.

### 3.5 시각 검증 및 asset 기록 규칙

일반 PR review, collaborator-mediated review, 여러 PR 체리픽 누적 검토 모두에서 PR 내용상 렌더링 결과
확인이 필요하면 [PDF/SVG visual sweep 가이드](visual_sweep_guide.md)를 사용한다. 시각 검증은 모든
샘플 PR 에 기계적으로 수행하는 절차가 아니라, PR 의 수정 목적과 검증해야 할 사용자-visible 동작에 맞춰
선택한다.

시각 검증 결과를 해석할 때도 최종 판단 기준은 PR 이 실제로 약속한 변경 범위다. renderer/layout/paint
개선 PR 이면 기준 PDF 와의 시각 차이가 blocker 가 될 수 있지만, serializer/roundtrip/parser 구조 보존처럼
렌더링 자체를 고치는 PR 이 아니면 visual sweep 차이는 참고 자료로만 기록하고 그 차이만으로 merge 보류나
reject 결론을 내리지 않는다.

시각 검증 후보 예시는 다음과 같다.

- renderer/layout/typeset/paint/wasm 출력이 바뀌는 PR
- HWP/HWPX 샘플, 기준 PDF, visual regression 자료가 추가된 PR
- 페이지 수, 쪽 경계, 표/그림 자리차지, 텍스트 wrap, clipping, 색상/밑줄처럼 눈으로 확인해야 하는 이슈
- 코드 검증은 통과하지만 PR 설명상 기준 PDF 또는 한컴 출력과의 비교가 필요한 PR

Codex 와 Claude 는 visual sweep 을 수행한 경우 `compare`, `overlay`, `review` PNG 경로와 페이지 수,
자동 후보 수, `pixel match`, `visual_accuracy_proxy_percent` 를 함께 제시하고, 검증 이미지를 확인한 뒤
작업지시자 승인 없이 시각 판정을 최종 통과로 단정하지 않는다.

시각 검증이 반드시 필요한 PR review 에 기준 PDF 가 첨부되어 있지 않으면, 먼저 3.5.1 절에 따라 HWP 2020
MCP 변환 서버로 기준 PDF 를 산출한다. MCP 산출 PDF 는 Hancom Office 2020 변환 결과이므로, 원본 파일과
변환 job 이 명확히 기록된 경우 visual sweep 의 기준 PDF 로 사용할 수 있다. MCP 변환이 실패하거나 원본
HWP/HWPX 파일이 PR diff 또는 저장소에 없어 산출할 수 없는 경우에만 maintainer 또는 collaborator 에게
한컴 2020/2024 등 실제 기준 프로그램에서 저장한 PDF 파일 업로드를 요청한다. 기준 PDF 없이 생성한 자동
비교 결과는 임시 참고 자료로만 기록하고 최종 시각 판정 근거로 사용하지 않는다.

원본 HWP/HWPX 가 없는 상태에서는 MCP 기준 PDF 를 만들 수 없고, 페이지 수 변화나 시각 검증 주장을 장기
재현할 수도 없다. 이 경우 review 문서에는 "원본 HWP/HWPX 미첨부로 독립 시각 검증 불가"를 명확히 적고,
merge 후 코멘트에는 다음 PR 부터 원본 HWP/HWPX 파일을 반드시 첨부해 달라는 요청을 포함한다.

PR 또는 관련 issue 본문/댓글에 첨부된 재현 문서는 review 시작 시 먼저 모두 내려받아 `samples/` 아래에
보존한다. GitHub `user-attachments` 파일, 본문에 삽입된 스크린샷 PNG, 외부 사이트에서 추적한 HWP/HWPX/PDF,
이슈 작성자가 올린 비교 보고서 PDF 모두 여기에 해당한다. 권장 경로는 `samples/issue{N}/` 또는 `samples/pr{N}/` 이며, 파일명은
출처와 역할을 알 수 있게 안정적으로 정한다. 이 원본 첨부 파일을 `output/` 에만 두거나 기준 PDF 라는 이유로
`pdf/` 에만 두지 않는다.

본문 첨부 PDF 가 visual sweep 기준 PDF 로도 쓰이면, 원본 첨부는 `samples/` 에 보존하고 기준 PDF 사본은
`pdf/` 아래에 둔다. review 문서나 사전 처리 판단 보고서(`pr_{N}_report.md`)에는 두 경로와 SHA-256 이
같은지, 기준 PDF 로 사용했는지, 단순 비교 보고서/참고 자료인지 구분해서 기록한다. 이렇게 해야 후속
fast-pass PR 에서 신규 `samples/**/*.hwp`, `samples/**/*.hwpx`, `samples/**/*.pdf`,
`samples/**/*.png`, `pdf/**/*.pdf` 를 모두 GitHub 에 보존할 수 있다.

#### 3.5.1 기준 PDF 미첨부 시 HWP 2020 MCP 산출 절차

PR 작성자가 검증 PDF 를 첨부하지 않았지만 PR 안에 기준으로 삼을 HWP/HWPX 원본이 있으면, review 담당자는
`hwp2020-mcp-convert` CLI 로 PDF 를 먼저 산출한다. 이 절차는 PDF 업로드 요청보다 우선한다.

MCP client tarball 위치, `.env.local` 준비, `npx --package=file:...` help 확인, CLI 인자 세부값은
`mydocs/manual/mcp_hwp2020Convert_usage.md` 를 함께 참고한다. VS Code `hwp2020Convert` 등록은 Chat 에서
tool 로 호출하고 싶을 때만 선택적으로 사용한다. 이 절은 PR review 에서의 저장 위치, 검증 증거 보존,
review 문서 기록 기준을 정의한다.

최종 저장 위치:

```text
pdf/{원본 stem}-2020.pdf
```

HWP 2020 MCP 산출 PDF 는 GitHub 에 남는 기준 PDF 여야 하므로 `output/` 아래에만 저장하지 않는다. 현행
GitHub Actions fast-pass 는 신규 `pdf/**/*.pdf` 를 review 기준 PDF 로 인정하므로, 50MB 미만 MCP 산출본은
기본적으로 `pdf/` 아래에 `{원본 stem}-2020.pdf` 이름으로 저장한다. 원본 파일이 `samples/basic/` 처럼 하위
디렉터리에 있으면 `pdf/basic/` 처럼 같은 하위 구조를 유지한다. 50MB 이상 PDF 는 `pdf-large/` 와 Git LFS
정책을 따르되, fast-pass 여부는 별도로 확인한다.

MCP 서버 IP 주소와 인증 토큰은 공개 issue, PR 본문, review 문서, 로그에 기록하지 않는다. 이 접근 정보는
인증된 collaborator 에게만 별도 비공개 채널로 공유한다. PR review 중 MCP 접근 정보가 필요한 경우에는
공개 GitHub 기록에는 "MCP 접근 정보는 인증된 collaborator 에게만 공유되며, 필요 시 @jangster77 에게 서버
URL/IP 와 토큰을 요청한다" 라고 적는다.

MCP 변환을 시작하기 전에 원본 파일 크기와 예상 페이지 수를 먼저 확인한다. 이미 PR 에 기준 PDF 가 있거나
저장소에 대응 PDF 가 있으면 `pdfinfo` 로 페이지 수를 확인하고, 기준 PDF 가 없으면 PR 설명, 샘플명,
기존 issue 기록, `rhwp dump-pages`/렌더 결과 등으로 대략적인 규모를 파악한다. 페이지 수가 많거나
거대 표/중첩 표/성능 검증 샘플처럼 변환 시간이 길어질 수 있는 경우에는 기본 `--timeout-seconds 240` 을
그대로 쓰지 않는다. CLI 의 `--timeout-seconds` 를 900~1800초처럼 충분히 크게 지정한다. VS Code MCP
tool 경로에서 `MCP error -32001: Request timed out` 이 나왔더라도 서버 job 이 성공해 PDF 를 생성했을 수
있으므로 곧바로 변환 실패로 단정하지 않는다. 이 경우 같은 입력을 `hwp2020-mcp-convert` CLI 로 다시
호출해 로컬 PDF 수신까지 검증한다.

사전 확인 예:

```bash
ls -lh samples/example.hwp
pdfinfo pdf/example-2024.pdf | rg '^Pages:'
```

CLI 변환 예:

```bash
/opt/homebrew/bin/npx -y \
  --package=file:/Users/me/rhwp/tools/hwp-convert-mcp-client-20260709-231800.tar.gz \
  -- \
  hwp2020-mcp-convert \
  --env-file /Users/me/Devel/hwp-convert/.env.local \
  --input /Users/me/rhwp/samples/example.hwp \
  --target pdf \
  --output-dir /Users/me/rhwp/pdf \
  --output-filename example-2020.pdf \
  --timeout-seconds 240
```

성공 조건:

- CLI result 가 `status: success` 이다.
- 서버 결과의 `run_status` 가 `0` 이다.
- 서버 결과의 `validation` 이 `ok` 이다.
- 로컬 출력 PDF 가 `pdf/` 아래에 존재하고 `file` 또는 `pdfinfo` 로 PDF 임을 확인한다.

review 문서에는 다음을 기록한다.

- 원본 HWP/HWPX 경로와 가능하면 SHA-256
- PR/issue 본문·댓글 첨부 파일의 원본 URL 또는 출처와 `samples/` 보존 경로
- MCP 출력 PDF 경로와 SHA-256
- MCP server job id
- `run_status`, `validation`, PDF page count
- 이 PDF 를 기준으로 실행한 visual sweep 산출물 경로와 지표

MCP 산출 PDF 는 visual sweep 입력이면서 장기 재현용 기준 PDF 이므로 review 문서/asset 과 함께 커밋 대상에
포함한다. 최종 PR/issue comment 에 보여주는 안정 증거는 3.5 절에 따라 선택한 `review_NNN.png` 를
`mydocs/pr/assets/` 아래로 복사한 파일을 기본으로 하되, 해당 visual sweep 의 기준 PDF 경로도 함께 적는다.

maintainer 또는 collaborator 가 PR review 검증을 위해 추가한 기준 PDF/HWP/HWPX 샘플이 untracked 로
존재하면, 해당 파일은 임시 디버그 산출물이 아니라 검증 근거 샘플로 간주한다. PR review 문서, visual
asset 과 함께 커밋 대상에 포함하고, review 문서에는 샘플 경로와 어떤 검증에 사용했는지 기록한다. 다만
코드 PR 과 검증 기록을 분리해야 하는 경우에는 원 PR 에 섞지 않고, 아래 옵션 2처럼 merge 후 별도
문서/자산 PR 에 포함한다.

이 문단의 커밋 대상 기준 PDF 에는 3.5.1 절의 MCP 산출 PDF 도 포함된다. MCP 산출 PDF 를 `output/` 아래에만
남기면 GitHub 에 검증 기준이 보존되지 않으므로, 최종 검증에 사용한 50MB 미만 파일은 반드시 `pdf/` 아래로
저장한다.

시각 검증 PNG 를 PR 기록 자산으로 남길 때는 먼저 PR review 문서에 visual sweep 산출물의 임시 경로
(`output/.../review/review_NNN.png` 등), 페이지 수, 자동 후보 수, `pixel match`,
`visual_accuracy_proxy_percent`, 사람 판정 메모를 기록한다.

visual sweep 을 실제 리뷰 근거로 사용했다면, **merge 가능/승인 요청 전** 선택한 대표 `review_NNN.png` 를
현재 review 작업 브랜치의 `mydocs/pr/assets/` 아래에 PR 번호가 포함된 안정 파일명으로 복사한다. 복사 직후
PR review 문서에는 임시 산출물 경로와 최종 asset 경로를 둘 다 기록한다. 이 작업이 끝나기 전에는 visual
sweep 검증을 완료로 표시하거나 merge 가능 결론을 내리지 않는다.

원시 PR comment 에서 이미지를 보여줄 예정이면 `output/...` 임시 경로만 남기고 끝내지 않는다. 이후 asset
반영은 작업자의 권한과 변경 범위에 따라 다음 세 방식 중 하나를 선택한다.

여러 페이지를 검증한 경우 모든 페이지 PNG 를 기계적으로 저장할 필요는 없지만, 리뷰 결론을 증명하는 정상
샘플 페이지와 보완 요청/후속 이슈 판단에 필요한 후보 페이지는 대표 asset 으로 남긴다. `compare`/`overlay`
개별 이미지를 따로 남길 필요가 있으면 함께 저장할 수 있고, 기본 코멘트 첨부용 대표 이미지는 좌우 비교가
포함된 `review_NNN.png` 를 사용한다.

**옵션 M. 메인터너가 운영 기록을 `devel` 에 직접 반영**

admin 또는 branch protection bypass 권한을 가진 메인터너가 원 코드 PR merge 후 review 문서,
`mydocs/pr/assets/` 검증 자산, 오늘할일처럼 코드 동작을 바꾸지 않는 운영 기록만 반영할 때 사용한다.
원 코드 PR merge commit 을 확인한 뒤 `devel` 을 최신 `upstream/devel` 로 fast-forward 하고, review 문서를
archive 경로로 이동한 상태에서 운영 기록을 한 커밋으로 반영한다.

```bash
git fetch upstream devel
git switch devel
git merge --ff-only upstream/devel
git diff --check
git status --short
git add mydocs/pr/archives/pr_N_review.md \
  mydocs/pr/assets/pr_N_visual_review.png \
  mydocs/orders/YYYYMMDD.md
git commit -m "docs: PR #N 검토 기록 보존"
git push upstream devel
```

실제 변경 파일만 `git add` 하며 존재하지 않거나 변경되지 않은 경로는 생략한다. 직접 반영 범위에 소스,
테스트, CI workflow, golden/baseline, 기존 샘플 수정이 포함되면 옵션 M 을 사용하지 않고 원 PR head 보정 또는
옵션 2 후속 PR 로 전환한다. 신규 HWP/HWPX/PDF 기준 자료가 포함되어 LFS·fast-pass·CI 확인이 필요할 때도
옵션 2를 사용한다.

**옵션 1. 현재 PR 에 review 문서, asset, 오늘할일을 함께 포함**

현재 PR 에 운영 기록을 함께 포함해도 되는 collaborator self-merge 후보 또는 collaborator-mediated 외부 PR
경로에서 사용한다. PR review 문서를 `mydocs/pr/archives/` 로 이동하고, 선택한 검증 PNG 를
`mydocs/pr/assets/` 아래로 옮기며, 오늘할일 갱신이 필요한 경우 `mydocs/orders/{yyyymmdd}.md` 도 같은
PR branch 에 포함한다. 이 세 항목을 한 묶음으로 remote push 해야 하며, review 문서/asset 만 올리고
오늘할일을 빠뜨린 상태로 옵션 1 완료로 표시하지 않는다.

오늘할일 갱신이 필요 없는 PR 이라면 review 문서에 그 사유를 짧게 남긴다. 그 PR 이 merge 된 뒤 원시 PR 또는
supersede 된 PR close/comment 시 `devel` 기준 asset 링크와 리뷰 결론을 남긴다.

**옵션 2. 원 PR merge 후 docs-only 문서/asset PR 로 분리**

코드 PR 과 review 기록/asset 을 분리해야 하거나 fast-pass 문서 PR 로 후속 처리하는 것이 더 안전한 경우
사용한다. collaborator 는 원 PR head 에 기록을 포함하지 못한 경우 이 방식을 기본으로 한다. 메인터너도
직접 반영 허용 범위를 벗어나거나 branch protection 정책상 direct push 를 사용할 수 없으면 이 방식을 쓴다.
원 코드 PR 을 먼저 merge 한 뒤, 선택한 검증 PNG 를 `mydocs/pr/assets/` 아래로 옮기고 review
문서/오늘할일과 함께 별도 후속 문서/자산 PR 로 올린다. 후속 문서/자산 PR 을 merge 하여 PNG 가 `devel` 에
존재하게 만든 다음, 원시 PR 또는 supersede 된 PR 에 review comment 를 남기며 asset 링크와 리뷰 결론을
안내한다.

옵션 2 후속 PR 은 merge 이후에도 별도 작업 브랜치와 worktree 가 남기 쉽다. 예를 들어
`/private/tmp/rhwp-pr1862-docs` 처럼 후속 문서/asset 전용 worktree 를 만들었다면, 후속 PR merge 후
7.7 절에 따라 `devel` 을 `upstream/devel` 로 fast-forward 하고 해당 worktree, 로컬 브랜치, 원격 추적
브랜치를 정리한다. squash merge 후 로컬 브랜치가 "not fully merged" 로 보일 수 있으므로, PR merge 와
작업트리 clean 상태를 확인한 뒤 `git branch -D` 로 정리한다.

작업지시자가 옵션 2 후속 처리를 중단하거나 후속 문서 PR 이 불필요하다고 결정한 경우에도, 이미 만든
docs-only 작업 브랜치와 worktree 는 같은 정리 대상이다. 이때는 PR merge 여부 대신 작업트리 clean 상태와
반영 불필요 결정만 확인한 뒤 7.7 절의 잔여 브랜치 검증까지 수행한다.

원시 PR comment 에 넣는 이미지 링크는 다음 형식을 사용한다.

```markdown
![PR #N visual review](https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/<file>.png)
```

## 4. 로컬 사전 검증

### 4.1 PR 브랜치 fetch

```bash
git fetch upstream pull/N/head:local/prN
```

### 4.2 Merge 시뮬레이션

```bash
git switch -c prN-merge-test upstream/devel
git merge local/prN --no-commit --no-ff
# 충돌 여부 확인
git status
```

실제 merge 대상인 `devel` 위에 PR head 를 합치는 방향으로 결과 tree 와 충돌 여부를 확인한다. 충돌 없으면
그대로 진행하고, 충돌 시 해결 방침은 작업지시자 결정을 요청한다.

### 4.2.1 여러 PR 체리픽 누적 검토

여러 PR 이 같은 영역을 단계적으로 수정하고 오래된 순서대로 merge 해야 하는 경우, 별도 로컬 브랜치에서
`upstream/devel` 기준 체리픽 누적 검토를 할 수 있다. 이때도 리뷰 기록은 반드시 원 PR 번호별로 분리한다.

- 체리픽 순서는 오래된 PR 번호 또는 작업지시자가 지정한 순서를 따른다.
- PR 내부의 `Merge branch 'devel' ...` 커밋은 검토 목적 체리픽에서 제외하고, 실제 기능/문서 커밋만 적용한다.
- 누적 브랜치는 충돌, 테스트, 시각 검증 확인용 임시 브랜치일 뿐이며 review 문서를 묶어서 한 파일로 만들지 않는다.
- `mydocs/pr/pr_{N}_review.md` 는 각 PR 번호별로 작성하고, `pr_{N}_review_impl.md` 는 3.2 절 조건에
  해당하는 PR 만 번호별로 추가한다.
- 각 review 문서에는 체리픽 순서, 적용한 커밋 SHA, 충돌 여부, 선행 PR 의존 여부를 해당 PR 기준으로 기록한다.
- 시각 검증이 필요한 경우 3.5 절의 공통 visual sweep 및 asset 기록 규칙을 따른다.
- 여러 PR 을 한꺼번에 로컬 검증했더라도 GitHub merge 전에는 각 PR 의 최신 head, mergeable, required checks 를 개별 확인한다.

### 4.3 빌드 · 테스트

Cargo 계열 검증은 순차 실행한다. `cargo test`, `cargo clippy`, `cargo build`, `wasm-pack build` 를
동시에 띄우면 package cache 또는 artifact directory lock 경합으로 시간이 늘고 진행 상태가 불명확해진다.
특히 focused test 여러 개를 확인할 때도 하나가 끝난 뒤 다음 명령을 실행한다.

모든 PR 에 아래 명령을 기계적으로 전부 적용하지 않는다. 변경 범위와 blast radius 에 맞춰 검증을 선택하고,
선택·생략한 명령과 이유를 review 문서에 기록한다.

| 변경 범위 | 기본 검증 |
|---|---|
| `mydocs/**` 문서만 변경 | `git diff --check`, 문서 경로·링크·변경 범위 확인. cargo 검증 생략 |
| Rust parser/model/CLI | focused test, `cargo test --profile release-test --tests`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` |
| renderer/layout/typeset/WASM | Rust 검증, `wasm-pack build --target web --out-dir pkg`, 2.6·3.5절 시각 검증 |
| `rhwp-studio/**` frontend만 변경 | `npx tsc --noEmit`, `npm test`, 실제 브라우저 동작 확인 |
| CI workflow 변경 | workflow 구문·변경 조건 확인과 최신 GitHub Actions 결과 확인 |
| 기존 golden/baseline/fixture 변경 | 관련 focused test, snapshot 결정성 재확인, 최신 PR head CI |

공용 API, 여러 crate, renderer와 frontend를 함께 바꾸는 대형 변경이나 작업지시자가 전체 검증을 승인한 경우에는
다음 전체 게이트를 순차 실행한다.

```bash
cargo build --release
cargo test --release --lib
cargo test --profile release-test --tests
cargo fmt --check
git diff --check
cargo clippy --all-targets -- -D warnings
cargo test --doc
cd rhwp-studio && npx tsc --noEmit
cd rhwp-studio && npm test
wasm-pack build --target web --out-dir pkg
```

`cargo test --profile release-test --tests` 에는 `tests/svg_snapshot.rs` integration test 도 포함된다.
따라서 전체 통합 테스트를 이미 통과했다면 `svg_snapshot`은 함께 검증된 것이다. 렌더 영향 PR 에서
golden 실패 여부만 좁혀 보거나 `UPDATE_GOLDEN=1` 재생성 후 결정성을 재확인해야 할 때는 다음 단독
명령을 추가로 사용한다.

```bash
cargo test --test svg_snapshot
```

단독 실행에서 실패하면 `UPDATE_GOLDEN=1` 으로 재생성이 필요한지 확인해야 하지만, **PR 머지 후가 아닌
머지 전에도 확인 필요**하다. 실패하면 작업지시자와 상의한다. 의도된 렌더 변경인지 버그인지 먼저 구분한다.

2.6 절에서 visual sweep 대상으로 판정한 PR 은 cargo 검증이 모두 통과해도 여기서 끝내지 않는다. 첨부 기준
PDF 또는 3.5.1 절의 MCP 산출 PDF 를 사용해 대표 샘플/페이지에 대해 visual sweep 을 실행하고,
`review_NNN.png` 확인 결과, 페이지 수, 자동 후보 수, `pixel match`, `visual_accuracy_proxy_percent` 를
review 문서에 기록한 뒤 merge 판단으로 넘어간다.

### 4.4 정리

```bash
git merge --abort   # merge 가 실제로 시작된 경우에만 수행한다. "Already up to date" 면 생략한다.
git fetch upstream devel
git switch devel
git merge --ff-only upstream/devel
git branch -D prN-merge-test
```

4.4 절은 merge 시뮬레이션 브랜치 정리만 다룬다. `local/prN`, `prN-review`, `prN-review-latest`,
docs-only/follow-up 브랜치처럼 PR review 를 위해 fetch 하거나 직접 만든 브랜치는 review 종료 시 7.7 절의
로컬/원격 PR 작업 브랜치 정리 게이트까지 수행해야 한다.

다음 중 하나가 확정되면 PR review 가 끝난 것으로 보고, 최종 상태 보고 전에 7.7 절을 실행한다.

- PR merge 완료
- PR reject/comment 후 close 완료
- supersede/통합 PR 처리로 원 PR 을 comment 후 close 완료
- review 중단 또는 보류로 해당 로컬 검토 브랜치를 더 이상 사용하지 않기로 결정
- 후속 기록 fast-pass PR merge 완료

CI 대기나 작업지시자 승인 대기처럼 review 가 아직 진행 중이면 로컬 브랜치를 유지할 수 있다. 이 경우에는
상태 보고에 "유지 중인 로컬 브랜치명" 과 "유지 사유" 를 명시한다.

## 5. 작업지시자 승인 요청

3장에서 정한 필수 review 문서와 필요한 경우의 `review_impl` 을 근거로 승인 요청한다. 예시 포맷:

```
PR #N 검토 결과 · admin merge 준비 완료.

- mergeable: MERGEABLE / BEHIND (승인 요청 시점 참고값, merge 전 재확인)
- 충돌 시뮬레이션: 0건
- cargo test --lib: XYZ passed
- Clippy: 0 warning
- 리뷰 문서: mydocs/pr/pr_N_review.md
- merge 전 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인

어떻게 진행할까요?
- A) admin merge
- B) 추가 검증
- C) 보류
```

## 6. Admin Merge 수행

```bash
gh pr merge N --repo edwardkim/rhwp --merge --admin
```

**주의**: `--admin` 플래그는 BEHIND 상태도 강제 머지한다. 프로젝트가 "devel 만 push · main 은 릴리즈 시" 정책이므로 `--admin` 이 기본.

## 7. 후속 처리 (필수 순서)

7장 절차는 순차 실행한다. 앞 단계가 성공/스킵/불필요 중 하나로 명확히 결론나기 전에는 다음 단계를
시작하지 않는다. 특히 이슈 close, 원 PR comment, 후속 문서/asset PR 생성·merge, 브랜치/worktree 정리를
동시에 진행하지 않는다.

권장 순서는 다음과 같다.

1. 원 코드 PR merge 완료와 merge SHA 확인
2. 후속 문서/asset 처리 필요 여부 확정
3. 옵션 M 이면 `devel` sync 후 운영 기록 직접 커밋·push, 옵션 2면 문서/asset PR 생성·CI 확인·merge
4. 최종 `devel` 을 `upstream/devel` 로 fast-forward sync
5. 관련 이슈 close 여부 확인 및 issue 후속 코멘트 작성
6. 원 PR 또는 supersede 된 PR 에 review comment 작성
7. 원 PR/후속 PR 의 로컬·원격 브랜치와 worktree 정리
8. 잔여 worktree, 로컬 브랜치, 원격 head 브랜치가 없는지 검증

시각 검증 asset 링크를 PR/issue comment 에 넣어야 하는 경우에는 문서/asset PR 이 `devel` 에 merge 되어
있거나 옵션 M 직접 반영 커밋이 `upstream/devel` 에 push 되어 raw asset URL 이 유효해진 뒤에 issue
close/comment 와 원 PR comment 를 남긴다.

### 7.1 후속 문서 처리 여부 확정

원 코드 PR merge 완료와 merge SHA 확인 직후에는 **반드시 후속 문서 처리 여부를 확정**한다.
PR 처리 사실이 GitHub metadata 에만 남고 `mydocs/orders/{yyyymmdd}.md`, PR review/report, task report 에
누락되는 것을 막기 위한 게이트다.

다음 중 하나로 결론을 남긴다.

- **PR head 에 이미 포함됨**: collaborator self-merge 후보 또는 collaborator-mediated 외부 PR 처럼 review 문서,
  오늘할일, report 가 merge 된 PR diff 에 함께 포함된 경우. 이때도 merge SHA, 이슈 close 여부, supersede close
  결과처럼 merge 후에야 확정되는 값이 빠졌는지 확인한다.
- **메인터너가 `devel` 에 직접 반영**: admin 또는 branch protection bypass 권한을 가진 메인터너가
  `mydocs/**` 운영 기록만 보강하는 경우. 원 PR merge 후 `devel` 을 동기화하고 archive review 문서,
  `mydocs/pr/assets/`, 오늘할일을 한 커밋으로 직접 반영한다.
- **별도 후속 기록 PR 필요**: merge 후 확정된 사실을 기존 문서에 보강해야 하거나, 오늘할일에 처리 기록이
  빠졌지만 collaborator 권한이거나 직접 반영 허용 범위를 벗어난 경우. `mydocs/**` 와 신규 기준 샘플/PDF 만
  포함하는 후속 기록 브랜치를 만들고 PR 로 반영한다.
- **추가 문서 불필요**: PR review/report/오늘할일이 모두 최신이고, merge 후 확정값 누락도 없는 경우. 이 판단은
  상태 보고에 명시한다.

메인터너 직접 반영 원칙:

- 원 코드 PR merge SHA 를 확인하고 `devel` 을 `upstream/devel` 로 fast-forward 한 뒤 기록을 작성한다.
- 변경 범위는 review/archive 문서, 오늘할일, `mydocs/pr/assets/` 아래 자산으로 제한한다.
- `git diff --check` 와 실제 staged 파일 목록을 확인하고 한 운영 기록 커밋으로 push 한다.
- 소스, 테스트, workflow, golden/baseline, 기존 샘플 수정 또는 신규 LFS 자료가 포함되면 후속 PR 로 전환한다.

후속 문서 PR 을 만드는 경우 원칙:

- 코드, 기존 샘플, workflow 파일을 섞지 않고 `mydocs/**` 문서와 신규 기준 샘플/PDF 만 변경한다.
- 후속 기록 변경이어도 `git diff --check` 를 실행한다.
- PR 본문에는 대상 PR/이슈 번호, merge SHA, 이슈 close 여부, fast-pass 조건을 기록한다.
- fast-pass 가 적용되어 heavy job 이 `skipped` 로 보이더라도 preflight 성공과 merge 가능 상태를 확인한 뒤 merge 한다.
- 후속 문서 PR merge 후 `devel` 을 다시 sync 하고, 문서 PR 브랜치도 로컬/원격 모두 정리한다.

### 7.2 devel Sync

```bash
git fetch upstream devel
git switch devel
git merge --ff-only upstream/devel
```

후속 처리 단계에서 `devel` 이 `upstream/devel` 과 diverge 한 경우에는 임의로 rebase 하지 않는다. 먼저
현재 브랜치, 미커밋 변경, 로컬 전용 커밋 여부를 확인하고 작업지시자에게 보고한다. collaborator 작업 기준
브랜치는 로컬 `devel` 이며, `local/devel` 예시는 사용하지 않는다.

### 7.3 이슈 Close 확인 및 후속 코멘트

GitHub auto-close 가 **자주 실패**한다. 후속 운영 기록이 필요한 경우에는 옵션 M 직접 반영 push 또는 옵션 2
문서/asset PR merge 와 최종 devel sync 후 수동 확인한다.

```bash
gh issue view N --repo edwardkim/rhwp --json state,closedAt
```

단, merge 직후에는 closing keyword 처리와 GitHub Actions auto-close 코멘트 반영이 몇 초에서 수십 초 늦게
보일 수 있다. 원 PR 이 `Closes #N` 또는 closing keyword 를 포함했다면 한 번의 즉시 조회 결과만 보고
`OPEN` 으로 단정하지 않는다. 다음처럼 시간을 두고 2~3회 재조회한 뒤에도 `OPEN` 일 때만 수동 close 대상으로
판단한다.

```bash
for i in 1 2 3; do
  gh issue view N --repo edwardkim/rhwp --json state,closedAt,comments
  sleep 10
done
```

`state: OPEN` 이면 수동 close + 후속 코멘트:

```bash
gh issue close N --repo edwardkim/rhwp \
  --comment "[https://github.com/edwardkim/rhwp/pull/M](https://github.com/edwardkim/rhwp/pull/M) 머지로 해결 (by @작성자). ..."
```

`state: CLOSED` 이고 GitHub auto-close 가 이미 동작했더라도 후속 코멘트는 생략하지 않는다. 자동 close 는
상태만 닫을 뿐, 어떤 검증과 시각 자료를 근거로 처리했는지 이슈 타임라인에 남기지 못한다. 따라서 관련 이슈가
PR description 의 `Closes #N` 로 자동 종료된 경우에도, 해당 이슈에 다음 내용을 실제 줄바꿈이 있는
heredoc/`--body-file` 방식으로 남긴다.

- merge 된 PR 번호와 merge commit
- GitHub Actions 및 로컬 검증 요약
- 관련 기준 PDF/시각 검증 asset 링크
- 남은 후속 과제 유무
- auto-close 로 이미 `CLOSED` 상태임을 확인했다는 기록

단, 이미 같은 merge commit 과 같은 검증 자료를 담은 maintainer 후속 코멘트가 해당 이슈에 있으면 중복 작성하지
않고 기존 코멘트 URL 을 상태 보고에 남긴다. GitHub Actions bot 의 auto-close 코멘트만 있는 상태는 후속
코멘트 완료로 보지 않는다.

### 7.4 기여자 감사 코멘트

원 PR 에 감사 + 검증 결과 요약 + 다음 PR 격려:

```
@기여자 감사합니다. 머지 완료했습니다.

[검증 결과 요약]
- 충돌 0 / cargo test ... passed / Clippy 0 warning

[재제출 피드백이 있었던 경우] 이번에 반영해주신 점:
- ... (구체 항목 1)
- ... (구체 항목 2)

[다음 작업 언급 — 있으면] 후속 이슈 [https://github.com/edwardkim/rhwp/issues/X](https://github.com/edwardkim/rhwp/issues/X)도 같은 방식으로 올려주시면 됩니다.

감사합니다.
```

페이지 수, 레이아웃, 렌더링 위치 변화처럼 시각 검증이 merge 판단 근거인 PR 은 다음 형식을 기본으로 한다.
검증 항목과 asset 링크는 실제 review 문서에 기록된 값만 사용한다.

GitHub review comment, merge 후 원 PR comment, issue 후속 comment 에서 이슈/PR 번호를 언급할 때는 대상
유형을 먼저 확인하고 **Markdown link 형식으로 직접 링크**한다. 짧은 `#2070`, `PR #2198` 같은 평문 참조나
링크가 아닌 URL 문자열만 남기지 않는다.

```markdown
[https://github.com/edwardkim/rhwp/issues/2070](https://github.com/edwardkim/rhwp/issues/2070)
[https://github.com/edwardkim/rhwp/pull/2198](https://github.com/edwardkim/rhwp/pull/2198)
```

merge 된 대상은 보통 PR URL, open 유지하거나 후속 과제로 안내할 대상은 issue URL 을 사용한다. 특정
코멘트를 증거로 인용할 때는 해당 `issuecomment-...` 또는 `discussion_r...`까지 포함된 permalink 를 같은
Markdown link 형식으로 적는다. 여러 번호를 한 문장에 쓸 때도 각각 별도 링크로 만든다.

```markdown
검토 및 머지 완료했습니다. 감사합니다.

확인한 내용은 다음과 같습니다.

- CI: Build & Test, CodeQL, Render Diff 계열 체크 통과
- 로컬 검증: `...` 테스트, 영향권 회귀 테스트, `git diff --check` 확인
- 페이지 수: 기준 PDF N페이지 / rhwp 렌더 결과 N페이지 확인
- visual sweep: pN 기준으로 PR의 핵심 주장인 “...”가 해소된 것을 확인
- 시각 검증 수치: `flagged=0/N`, pixel match `NN.NNNNN%`, 내용 픽셀 중심 자동 일치율 보조값 `NN.NNNNN%`

시각 검증 자료는 아래에 함께 첨부합니다.

pN visual sweep:

![PR N pN visual sweep](https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/<review>.png)

이번 판단은 단순히 PNG가 완전히 동일한지보다, PR 내용에서 주장한 회귀가 해결됐는지를 중심으로 봤습니다.
[남은 세부 시각 차이/후속 후보가 있으면 명시] 이번 PR의 merge blocker로 보지는 않았습니다.

[관련 이슈를 close 하지 않는 경우] 이 PR은 [https://github.com/edwardkim/rhwp/issues/ISSUE](https://github.com/edwardkim/rhwp/issues/ISSUE)의 일부 발현만 처리합니다. 남은 발현/후속 과제가 있으므로 해당 이슈는 open 상태로 유지합니다.

다음에 페이지 수나 시각 검증이 필요한 PR을 올려주실 때는, 원본 HWP/HWPX 파일과 한컴 2020/2024 등에서 저장한 기준 PDF를 함께 첨부해 주세요. 기준 PDF만 없으면 maintainer 측에서 HWP 2020 MCP 로 산출해 검증할 수 있지만, 원본 HWP/HWPX 가 없으면 페이지 수 변화와 시각 검증을 장기적으로 재현하기 어렵습니다. 기대 출력 의도도 함께 적어주시면 review와 회귀 판단을 더 빠르고 정확하게 진행할 수 있습니다.
```

외부 contributor 의 원 코드 PR 이 최종 diff 상 문서-only / review-only fast-pass 로 판정된 경우에도 후속
코멘트 예외가 아니다. heavy job 이 `skipped` 로 끝난 경우에도 PR 에 다음 내용을 남긴다.

- merge 완료 사실과 merge commit
- 문서-only / review-only PR 로 판정된 파일 범위
- preflight 성공 및 heavy job skip 결과
- 관련 이슈 close 또는 후속 추적 여부
- 기여자 감사 또는 후속 요청

특히 #1776 처럼 `Update branch` 후 최종 PR diff 가 `mydocs/**` 만 남아 heavy CI 를 skip 한 경우에도,
"문서-only fast-pass 로 검증했고 merge 했다"는 사실을 PR 코멘트에 명시한다.

단, 메인터너 또는 collaborator 가 이미 완료된 원 PR 의 review 문서/asset/오늘할일만 반영하기 위해 별도로
만든 후속 기록 fast-pass PR 은 이 코멘트 절차의 대상이 아니다. 이 유형은 fast-pass 조건과 merge 가능 상태만
확인한 뒤 merge 하고, merge 후 추가 issue close/comment, PR comment, 오늘할일 갱신, 별도 후속 문서 PR 을
생성하지 않는다. 다만 merge 후 `devel` sync 와 해당 fast-pass PR 의 로컬/원격 브랜치 및 worktree 정리,
잔여 브랜치 검증은 반드시 수행한다.

### 7.5 렌더 영향 PR 의 경우 · Golden 재생성 체크

4.3 절의 `cargo test --profile release-test --tests` 를 통과했다면 `svg_snapshot` 자체는 이미 포함되어
실행된 것이다. 이 절은 렌더 영향 PR 에서 golden 재생성이 필요할 때 단독으로 실패를 좁히고 재생성
결정성을 확인하는 절차다.

단독 확인:

```bash
cargo test --test svg_snapshot
```

실패 시:

```bash
UPDATE_GOLDEN=1 cargo test --test svg_snapshot
cargo test --test svg_snapshot   # 결정성 재확인
git add tests/golden_svg/
git commit -m "test(svg_snapshot): regenerate golden after #N (...)"
git push <PR-head-remote> HEAD:<PR-head-branch>
```

golden 재생성은 원 PR merge 전에 PR head 의 별도 커밋으로 반영하고 최신 CI 를 다시 확인한다. 원 PR 이 이미
merge 된 뒤 필요성이 확인됐다면 전용 후속 PR 을 만든다. golden/baseline 은 테스트 기준 파일이므로 메인터너
권한이 있어도 옵션 M 의 `devel` 직접 반영 대상이 아니다.

2회 연속 재현된 실수 (PR #221 / PR #251 사이클) 로 인해 **체크리스트 수준의 필수 절차**.

### 7.6 리뷰 문서 archives 이동

maintainer 일반 경로의 PR 리뷰 문서는 처리 완료 후 archive 경로로 이동한다.

```bash
mv mydocs/pr/pr_N_review.md mydocs/pr/archives/
mv mydocs/pr/pr_N_review_impl.md mydocs/pr/archives/
```

메인터너 일반 경로의 옵션 M 에서는 오늘할일·asset 과 같은 `devel` 운영 기록 커밋에 동반한다.
collaborator self-merge 후보 예외 경로에서는 처음부터 archive 경로에 두므로 이 이동 단계를 수행하지 않는다.

후속 기록 fast-pass PR 로 review 문서를 반영하는 경우에도 별도 archive 이동 PR 을 만들지 않는다. 후속 기록
PR 커밋에 포함할 review 문서는 처음부터 `mydocs/pr/archives/` 아래에 두거나, active 경로에 이미 작성한
문서를 같은 후속 기록 PR 커밋 안에서 archive 경로로 이동한다.

### 7.7 로컬/원격 PR 작업 브랜치 정리

이 절은 PR review 의 최종 종료 게이트다. 성공 merge 후에만 수행하는 선택 절차가 아니라, reject/close,
supersede 처리, review 중단, 후속 기록 fast-pass PR 완료 뒤에도 해당 작업에서 더 이상 사용하지 않는
로컬 브랜치와 worktree 를 정리한다. 정리 또는 유지 사유 확인이 끝나기 전에는 "후속 처리 완료" 로 보고하지
않는다.

PR 처리용으로 별도 worktree 를 만들었다면 브랜치 삭제 전에 worktree 를 먼저 제거한다. worktree 가 남아 있으면
해당 브랜치가 checkout 중인 상태라 `git branch -D` 가 실패한다.

정리 전에 삭제 대상 이름을 먼저 기록한다. 로컬 검토 브랜치명과 GitHub PR head 브랜치명이 다를 수 있으므로
둘을 같은 이름이라고 가정하지 않는다.

```bash
gh pr view N --repo edwardkim/rhwp --json headRefName,headRepositoryOwner,headRepository
git branch --show-current
git worktree list --porcelain
```

정리 대상은 다음 모두다.

- PR fetch/검토용 로컬 브랜치: 예 `local/prN`, `prN-merge-test`
- 현재 checkout 중인 review 브랜치가 별도 이름인 경우: 예 `prN-review`, `prN-review-latest`
- collaborator 가 원본 저장소에 직접 만든 PR head 브랜치: `headRefName`
- 옵션 2 후속 문서/asset PR 에 사용한 로컬 브랜치와 원격 head 브랜치
- 작업 중 만들었다가 중단한 docs-only/follow-up 브랜치와 worktree

옵션 2 후속 문서/asset PR 브랜치도 같은 정리 대상이다. 후속 PR 이 squash merge 된 경우 로컬 브랜치는
원격 `devel` 에 내용이 반영됐어도 commit graph 상 완전 병합으로 보이지 않을 수 있다. 이때는 다음을 확인한 뒤
강제 삭제한다.

- GitHub PR state 가 `MERGED`
- merge commit 이 `upstream/devel` 에 fetch 됨
- 해당 worktree 의 `git status --short` 가 clean
- 후속 PR 에 올린 문서/asset 이 `upstream/devel` 에 존재함

```bash
git worktree list
git worktree remove /path/to/pr-worktree
```

```bash
git branch -D <local-review-branch>
```

collaborator self-merge 후보처럼 원본 저장소에 PR head 브랜치를 직접 만든 경우에는 merge 후 원격
작업 브랜치도 삭제한다. 예를 들어 PR head 가 `upstream/task_m100_1470` 또는
`upstream/task_m100_1601_mydocs_fast_pass` 형태라면 다음을 수행한다. `<local-docs-branch>` 처럼
해당 작업에서 만들지 않은 항목은 생략한다.

```bash
git fetch upstream devel
git switch devel
git merge --ff-only upstream/devel
git worktree list
# 해당 브랜치를 checkout 한 별도 worktree 가 있으면 먼저 제거한다.
git worktree remove /path/to/pr-worktree
git push upstream --delete <headRefName>
git branch -D <local-review-branch>
# 옵션 2 후속 문서/asset 브랜치를 만들었으면 함께 삭제한다.
git branch -D <local-docs-branch>
git fetch upstream --prune
```

삭제 후에는 worktree, 로컬 브랜치, 원격 추적 브랜치가 남지 않았는지 확인한다.

```bash
git worktree list --porcelain
git branch --list '<local-review-pattern>' '<docs-pattern>' '<headRefName>'
git branch -r | rg '<local-review-pattern>|<docs-pattern>|<headRefName>' || true
git branch -vv | rg ': gone\]' || true
git ls-remote --heads upstream <headRefName>
git status --short --branch
```

### 7.8 오늘할일 갱신

maintainer 일반 경로에서는 PR merge 와 후속 처리를 끝낸 뒤 이 절을 수행한다. collaborator 경로의
오늘할일 생성·갱신 시점은 8.2.1 절 또는 9.2.1 절을 따른다.

메인터너가 옵션 M 을 사용하는 경우 오늘할일은 archive review 문서·asset 과 같은 `devel` 운영 기록
커밋에 포함한다. collaborator 또는 직접 반영 허용 범위를 벗어난 변경은 옵션 1 또는 옵션 2 PR 에 포함한다.

`mydocs/orders/yyyymmdd.md` 에 해당 PR 처리 내역 기록:
- PR 번호 + 제목 + 작성자
- merge SHA
- 관련 이슈 close 여부
- 후속 작업 (있으면)

## 8. Collaborator self-merge 후보 예외 경로

이 절은 collaborator 가 본인 PR 을 self-merge 후보로 준비하는 경우에만 적용한다. maintainer 가 외부
기여자 PR 을 검토하는 일반 경로를 대체하지 않는다.

### 8.1 적용 조건

- PR 작성자 또는 작업 준비자가 repository collaborator 이다.
- PR 번호가 이미 생성되어 review 문서명을 확정할 수 있다.
- merge 후 별도 문서 커밋을 만들지 않기 위해 review 문서를 PR diff 에 함께 포함해야 한다.
- 작업지시자 승인 전에는 ready 전환 또는 merge 판단을 하지 않는다.

### 8.2 문서 경로

collaborator self-merge 후보에서는 처음부터 archive 경로에 필수 review 문서와 필요한 경우의
`review_impl` 을 작성할 수 있다.

```text
mydocs/pr/archives/pr_{N}_review.md
mydocs/pr/archives/pr_{N}_review_impl.md   # 3.2 절 조건에 해당할 때
```

이 방식은 PR head 에 운영 문서를 포함해 merge 후 추가 문서 커밋을 방지하기 위한 예외다.
maintainer 일반 경로의 active 경로 작성 규칙까지 대체하지 않는다.

이미 active 경로에 잘못 만들었더라도 다음 PR 에 임시 동반하는 식으로 일반화하지 말고, 같은 PR 준비
단계에서 archive 경로로 바로 정리한 뒤 PR head 에 포함한다.

### 8.2.1 오늘할일 생성·갱신 시점

이 절은 내부 issue/task 구현, 문서-only 변경, 외부 PR 통합 등 collaborator 가 준비하는 모든 self PR 에
적용한다. 오늘할일 갱신이 필요하더라도 이슈 등록, 작업 브랜치 생성, 최초 조사, 계획서 작성, 구현 중간
단계에서는 `mydocs/orders/{yyyymmdd}.md` 를 생성하거나 갱신하지 않는다.

같은 날짜에 다른 contributor·collaborator PR 이 먼저 merge 되어 오늘할일 파일을 바꿀 수 있으므로, 작업
초기에 만든 항목은 쉽게 충돌하거나 오래된 상태가 된다. 따라서 변경 범위·검증 결과·merge 판단과 PR 생성
승인이 모두 확정된 **PR 생성 직전의 최종 준비 시점**에 최신 `devel`의 오늘할일을 먼저 반영한 뒤,
해당 self PR 의 최종 준비 커밋에 오늘할일을 생성·갱신한다.

오늘할일은 최초 remote push 및 PR 생성 전에 PR diff 에 포함되어 있어야 한다. PR 번호가 발급된 뒤 번호를
추가하기 위해 오늘할일을 새로 만들거나 다시 갱신하지 않는다. PR 번호가 필요한 review 문서·asset 을 PR 생성
후 추가하더라도 기존 오늘할일 항목은 그대로 유지한다. 최초 push 직전에는 최신 `upstream/devel` 기준
오늘할일과 충돌 여부를 다시 확인한다.

collaborator self-merge 후보에서는 오늘할일을 PR 생성 후 후속 커밋이나 merge 후 별도 PR 로 늦게 만들지
않는다. PR 번호가 필요한 review 문서·asset 후속 커밋은 9.3.1 절의 fast-pass 조건으로 통과시킬 수 있지만,
오늘할일은 그보다 앞선 최초 PR diff 에 이미 포함되어 있어야 한다.

여기서 최종 PR review 문서 묶음은 다음 중 해당 PR 에 실제로 필요한 문서다.

- `mydocs/pr/archives/pr_{N}_review.md`
- `mydocs/pr/archives/pr_{N}_review_impl.md` (3.2 절 조건에 해당할 때)
- `mydocs/pr/archives/pr_{N}_report.md` (필요 시)
- `mydocs/orders/{yyyymmdd}.md` (오늘할일 갱신이 필요한 경우)

### 8.3 remote push 규칙

collaborator 는 PR용 작업 브랜치를 fork 저장소(`origin`)에 우회 생성하지 않는다. 로컬 브랜치에서
원본 저장소 remote(`upstream`)의 작업 브랜치로 직접 push 하는 것을 기본 규칙으로 삼는다.

```bash
git push upstream HEAD:task_m100_1158
```

fork 브랜치를 head 로 쓰는 방식은 권한 제약 때문에 직접 push 가 불가능한 경우에만 예외로 둔다.

### 8.4 merge 전 최종 조건

collaborator self-merge 후보라도 최종 merge 판단은 다음 조건을 모두 만족해야 한다.

- PR head 최신 커밋 기준 GitHub Actions 통과
- `pr_{N}_review.md` 와 3.2 절 조건에 해당하는 경우의 `review_impl` 이 PR diff 에 포함됨
- 오늘할일 갱신이 필요한 경우 `mydocs/orders/{yyyymmdd}.md` 가 같은 PR diff 에 포함됨
- 작업지시자 승인

`draft`, `mergeable`, `head SHA`, `CI 상태`는 3.3 절에 따라 작성 시점 참고값 또는 merge 전 최신 확인
조건으로만 기록한다.

## 9. Collaborator-Mediated 외부 PR 처리 경로

이 절은 외부 contributor PR 을 repository collaborator 가 검토하고 merge 준비하는 경우에 적용한다.
maintainer 일반 경로를 대체하지 않으며, 별도 문서 PR 을 만들지 않기 위해 review 문서를 해당 PR head 에
포함하는 예외 경로다.

이 경로가 필요한 이유는 collaborator 권한 모델 때문이다. collaborator 는 원본 저장소 `devel` 에 직접
문서 커밋을 push 하지 않고 PR 을 통해서만 변경을 반영한다. 따라서 외부 PR 을 merge 한 뒤
`pr_{N}_report.md` 만 별도 문서 PR 로 올리는 방식은 PR 처리 비용을 불필요하게 늘린다. 또한 문서만 있는 PR 은
CI `paths-ignore` 조건 때문에 핵심 검증이 실행되지 않을 수 있어, "처리 후 report 작성 -> 별도 문서 PR merge" 를
기본 흐름으로 삼지 않는다.

실제 선례:

- PR #1376: `mrshinds` 외부 PR 에 maintainer 보정·review 문서·오늘할일을 PR head 에 포함한 뒤 merge
- PR #1429: `seo-rii` 외부 PR 에 review 문서·오늘할일을 PR head 에 포함한 뒤 merge
- PR #1447: `seo-rii` 외부 PR 에 review/report 문서·오늘할일을 PR head 에 포함한 뒤 merge

### 9.1 적용 조건

- PR 작성자는 외부 contributor 이다.
- repository collaborator 가 리뷰, 문서화, merge 준비를 담당한다.
- GitHub PR 의 `maintainer_can_modify` 가 `true` 이거나, contributor 가 collaborator 의 문서 커밋 push 를
  명시적으로 허용한다.
- review 문서만 별도 PR 로 만들지 않기 위해 PR head 에 운영 문서를 포함하는 편이 더 단순하다.
- 작업지시자 승인 전에는 GitHub review approval, ready 전환, merge 판단을 완료하지 않는다.

`maintainer_can_modify=false` 이면 이 경로를 쓰지 않는다. 이 경우 maintainer 일반 경로로 active review 문서를
작성하거나, 작업지시자 지시에 따라 별도 문서 커밋/PR 로 처리한다.

### 9.2 문서 경로

collaborator-mediated 외부 PR 에서는 PR head 에 다음 문서를 직접 포함할 수 있다.

```text
mydocs/pr/archives/pr_{N}_review.md
mydocs/pr/archives/pr_{N}_review_impl.md   # 필요 시
mydocs/pr/archives/pr_{N}_report.md        # 필요 시, 사전 처리 판단 보고서로 작성
```

오늘할일 갱신이 필요한 경우 `mydocs/orders/{yyyymmdd}.md` 도 같은 PR head 에 포함한다.

### 9.2.1 오늘할일 생성·갱신 시점

collaborator-mediated 외부 PR 경로에서는 오늘할일을 PR 착수나 최초 조사 단계에서 미리 생성하지 않는다.
외부 contributor PR 에 얹을 최종 PR review 문서 묶음을 작성할 때 오늘할일을 같은 커밋으로
생성·갱신한다. 이렇게 해야 작업 중간 상태가 오늘할일에 먼저 확정 기록처럼 남지 않고, PR head 에 포함되는
운영 문서와 오늘할일이 같은 판단 시점을 공유한다.

collaborator-mediated 경로에서도 오늘할일을 merge 후 별도 PR 로 늦게 만들지 않는다. 오늘할일이 필요한 경우
PR review 문서와 함께 PR head 에 포함하고, 문서/asset/오늘할일만 바뀐 후속 커밋은 9.3.1 절의 fast-pass
조건으로 통과시키는 것을 기본으로 한다.

여기서 최종 PR review 문서 묶음은 다음 중 해당 PR 에 실제로 필요한 문서다.

- `mydocs/pr/archives/pr_{N}_review.md`
- `mydocs/pr/archives/pr_{N}_review_impl.md` (3.2 절 조건에 해당할 때)
- `mydocs/pr/archives/pr_{N}_report.md`
- `mydocs/orders/{yyyymmdd}.md` (오늘할일 갱신이 필요한 경우)

단순·소형 PR 은 `pr_{N}_review.md` 안에 처리 계획을 포함하고 별도 `review_impl` 을 생략할 수 있다.
`pr_{N}_report.md` 를 함께 포함할 때는 merge 완료 후 사후 보고서가 아니라 **사전 처리 판단 보고서**로 작성한다.
따라서 아직 확정되지 않은 merge SHA, 실제 merge 시각, 이슈 close 완료 여부를 단정하지 않는다. 대신 다음을
기록한다.

- merge 수용/보류/재작업 권고와 사유
- merge 전 최종 조건
- merge 후 확인해야 할 이슈 close, 감사 코멘트, 후속 작업

merge 완료 사실과 이슈 close 결과는 GitHub PR/Issue metadata 를 원천 기록으로 삼고, 별도 문서 PR 을 만들지
않는다. 사후에 반드시 장기 보관 보고서가 필요한 예외는 작업지시자 승인 후 별도 PR 로 처리한다.

### 9.3 PR head push 규칙

외부 contributor 브랜치에 collaborator 가 커밋을 얹을 때는 다음을 지킨다.

- contributor 의 원 코드 커밋을 rewrite 하지 않는다.
- review 문서, 오늘할일, maintainer 보정 코드는 별도 커밋으로 분리한다.
- maintainer 보정 코드가 포함되면 review 문서에 contributor 원 변경과 collaborator 추가 변경을 구분한다.
- 문서 커밋 push 후 GitHub Actions 결과를 확인한다. full CI 재실행 또는 9.3.1 절의 fast-pass 결과가
  merge 가능 상태여야 한다.

예시:

```bash
git fetch upstream pull/N/head:local/prN
git switch local/prN
# review 문서 작성 및 검증
git commit -m "docs: PR #N 검토 기록"
git push https://github.com/{contributor}/rhwp.git HEAD:{head-branch}
```

### 9.3.1 후속 기록 PR fast-pass

메인터너 또는 collaborator 가 외부 PR head 또는 별도 후속 PR 에 review/운영 문서와 기준 샘플을 보강하는 경우,
다음 조건을 모두 만족하면 heavy CI 가 job-level 로 skip 될 수 있다.

별도 후속 기록 fast-pass PR 에 포함하는 review 문서는 active 경로가 아니라
`mydocs/pr/archives/pr_{N}_review*.md` 경로로 준비한다. 이미 active 경로에 작성해 둔 경우에는 후속 기록
PR 커밋 전에 archive 경로로 이동한다. archive 이동만을 위한 별도 PR 은 만들지 않는다.

- PR head 의 뒤쪽 후속 커밋들이 아래 항목만 변경한다.
  - `mydocs/**`
  - 신규 추가(`added`) 상태의 `samples/**/*.hwp`
  - 신규 추가(`added`) 상태의 `samples/**/*.hwpx`
  - 신규 추가(`added`) 상태의 `samples/**/*.pdf`
  - 신규 추가(`added`) 상태의 `samples/**/*.png`
  - 신규 추가(`added`) 상태의 `pdf/**/*.pdf`
- 기존 `samples/**` 또는 `pdf/**` 파일을 수정, 삭제, rename 한 경우 fast-pass 대상이 아니다.
- 해당 후속 커밋들은 single-parent commit 이다.
- 후속 문서 커밋을 제외한 직전 코드 검증 대상 SHA 에 기존 GitHub Actions check-run 이 존재한다.
- 직전 코드 검증 대상 SHA 의 relevant check 가 `success`, `skipped`, `neutral` 중 하나다.

fast-pass 는 merge 조건을 약화하는 예외가 아니다. 이전 코드 검증 결과를 재사용하는 좁은 최적화일 뿐이다.
다음 변경이 포함되면 반드시 최신 PR head 기준 heavy CI 를 다시 기다린다.

- 코드, 테스트, CI workflow 파일(`.github/workflows/**`) 변경
- 기존 샘플, baseline, golden, 렌더링 fixture 변경
- `mydocs/**` 밖의 파일 변경 또는 신규 기준 샘플/PDF 추가가 아닌 실행/검증 입력 파일 변경
- check-run 조회 실패, missing check, failed check, merge commit 형태의 문서 후속 커밋

fast-pass 가 적용되면 PR UI 에서 heavy job 이 `skipped` 로 보일 수 있다. 이때도 collaborator 는 GitHub Actions
결과가 merge 가능 상태인지 확인하고, branch protection 이 요구하는 check 가 pending/failing 이면 merge 하지
않는다.

메인터너 또는 collaborator 가 이미 완료된 원 PR 의 review 문서/asset/오늘할일만 반영하기 위해 별도로 만든
후속 기록 fast-pass PR 은 merge 후 7장 중 issue close/comment, PR comment, 오늘할일 갱신, 별도 후속 문서 PR
생성을 반복하지 않는다. 이 PR 자체가 후속 처리 산출물이기 때문이다. 단, merge 후 `devel` sync 와 해당
fast-pass PR 의 로컬/원격 브랜치 및 worktree 정리, 잔여 브랜치 검증은 반드시 수행한다.

반대로 외부 contributor 의 원 코드 PR 이 최종 diff 상 문서-only / review-only fast-pass 로 판정된 경우에는
원 PR 처리이므로 7장 후속 처리를 수행한다.

### 9.4 merge 전 최종 조건

- PR head 최신 커밋 기준 GitHub Actions 통과 또는 9.3.1 절의 후속 기록 PR fast-pass 결과 확인
- review 문서가 PR diff 에 포함됨
- 오늘할일 갱신이 필요한 collaborator 경로에서는 `mydocs/orders/{yyyymmdd}.md` 가 같은 PR diff 에 포함됨
- `pr_{N}_report.md` 를 작성한 경우 사전 판단 보고서 형식이며, merge 후 사실을 미리 단정하지 않음
- GitHub review 또는 PR comment 로 검토 결과를 contributor 에게 남김. 단, 메인터너 또는 collaborator 의 별도
  후속 기록 fast-pass PR 자체는 이미 후속 산출물이므로 이 항목을 요구하지 않는다.
- merge 전 최신 `mergeable` / `mergeStateStatus` 재확인
- 작업지시자 승인

이 경로로 외부 contributor 의 원 PR 을 merge 한 뒤에는 7장 후속 처리를 동일하게 수행한다. 특히 `devel` 이
default branch 가 아니어서 `closes #N` 자동 close 가 실패할 수 있으므로 관련 이슈 state 를 반드시 확인한다.
단, 메인터너 또는 collaborator 의 별도 후속 기록 fast-pass PR 자체는 merge 후 추가 issue close/comment,
PR comment, 오늘할일 갱신, 별도 후속 문서 PR 을 만들지 않는다. 브랜치/worktree 정리는 반드시 수행한다.

## 10. 재작업 요청 패턴

리뷰 결과 **재작업이 필요**한 PR (예: base 가 main, 메타 변경 혼입, 관련 이슈 없음 등):

1. **PR 에 정중한 피드백 코멘트** + 구체적 수정 요청 목록
2. **PR close** (재제출 기대 의사 표명)
3. 재제출 시 새 PR 번호로 처리

실제 성공 사례: PR #234 (close) → PR #251 (재제출, 모든 피드백 반영 후 admin merge).

피드백 톤 가이드:
- **결정 자체는 단호**: "현 상태로는 머지 불가"
- **사유는 구체적**: "base 가 main 이라 릴리즈 브랜치에 직접 커밋되는 구조"
- **재제출 경로는 명확**: "feature 브랜치에서 devel 타깃으로 재제출 부탁드립니다"
- **크레딧 약속**: "재제출 시 PR description / commit author 보존 그대로"

### 10.1 영어 요청 contributor 응답 언어

contributor 가 영어로 PR 설명, 질문, 검토 요청을 남긴 경우에도 메인테이너 기록성과 외부 contributor 배려를
함께 만족하도록 **하나의 코멘트 안에 한글 문단과 영어 문단을 구분해 병기**한다.

- 한글 문단을 먼저 작성하고, `---` 구분선 뒤에 같은 의미의 영어 문단을 작성한다.
- 문장 단위로 한영을 섞지 않는다. 읽기 쉽도록 언어별 문단을 분리한다.
- 칭찬, 감사, CI 실패 원인, 수정 요청, 다음 검토 조건을 두 언어 모두에 같은 수준으로 포함한다.
- draft PR 인 경우 "현재 draft 는 정식 PR 검토 대상에 포함하지 않으며, 검토를 원하면 Ready for review 로
  전환해 달라"는 요청도 두 언어 모두에 포함한다.

예시 구조:

```markdown
@contributor 한글 감사/긍정 피드백 문단.

한글 CI 상태와 수정 요청 문단.

---

English thanks / positive feedback paragraph.

English CI status and requested fix paragraph.
```

## 11. 예외 케이스

### 11.1 Dependabot PR

`dependabot/npm_and_yarn/...` 브랜치 PR:
- 보통 base 가 `main` (설정 이슈) → `.github/dependabot.yml` 에 `target-branch: devel` 추가로 해결
- 현재 main 타깃 PR 은 close + 수동으로 devel 에 버전 bump 커밋

### 11.2 오래된 base PR (대량 커밋 혼입)

예: PR #213 같이 수십 커밋 전의 base 에서 분기 → diff 에 이미 머지된 과거 커밋들이 포함됨

처리:
- 해당 기여자의 **신규 커밋만 cherry-pick** (저자 보존)
- PR 은 close + 설명 코멘트 ("이번 기여 2 커밋만 cherry-pick 반영했습니다")
- 중복 PR (같은 브랜치 main 타깃) 도 함께 close

### 11.3 대형 PR (>1000 라인)

- 즉시 admin merge 불가
- 코드 검토 + 사전 시뮬레이션 충분히 수행 후 결정
- 예: PR #165 (skia renderer · +100K 라인) — 장기 보류

## 12. 메모리 등록 항목 (자동 참조)

다음 상황은 `~/.claude/.../memory/` 에 등록되어 있다:

- `feedback_search_troubleshootings_first.md` — 작업 전 트러블슈팅 폴더 검색
- `feedback_external_docs_self_censor.md` — 외부 공개 문서 자기검열
- (신규 제안) `feedback_golden_regen_after_render_pr.md` — 렌더 PR 머지 후 golden 재생성

## 13. 참고 아카이브

- `mydocs/pr/archives/pr_234_review.md` — 재작업 요청 사례
- `mydocs/pr/archives/pr_235_review.md` · `pr_237_review.md` — 다양한 리뷰 패턴
- `mydocs/pr/archives/pr_251_review.md` — 재제출 후 머지 사례 (모든 피드백 반영)
