---
kind: guide
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# 시각·fixture 증적

renderer, layout, typeset, paint, WASM 출력, HWP/HWPX/PDF fixture, 페이지 수·표 분할·wrap·clipping을
검토하는 경우에만 이 가이드를 읽는다. 모든 sample PR에 기계적으로 visual sweep을 수행하지 않는다.

## 3.5 시각 검증 원칙

최종 판단은 PR이 약속한 사용자-visible 변경 범위다. renderer/layout/paint 개선은 기준 PDF와의 시각 차이가
blocker가 될 수 있지만, parser·serializer 구조 보존 PR은 visual 차이를 참고 자료로 기록하고 그 차이만으로
merge를 보류하지 않는다.

visual sweep을 실제 검토 근거로 쓰면 review 문서에 다음을 모두 기록한다.

- 문서 비교 절차의 정본인 [PDF/SVG visual sweep 가이드](../verification/visual_sweep_guide.md#github-merge-comment)와
  적용한 command·판정 범위
- compare, overlay, review PNG의 임시 output 경로
- 검토한 페이지 수와 자동 후보 수
- pixel match, visual_accuracy_proxy_percent
- 사람이 확인한 결과와 PR 주장과의 관계

Codex 또는 Claude가 이미지를 확인했더라도 작업지시자 승인 전에는 시각 판정을 최종 통과라고 단정하지 않는다.
원본 HWP/HWPX, 기준 PDF, visual sweep 결과의 출처·역할·SHA-256을 구분해 보존한다.

## 원본 fixture와 기준 PDF 보존

PR 또는 관련 issue 본문·comment에 첨부된 HWP/HWPX/PDF/PNG와 외부에서 추적한 재현 문서는 review 시작 시
내려받아 samples/issueN 또는 samples/prN 아래에 안정적인 이름으로 보존한다. 원본 첨부를 output에만
두거나 기준 PDF라는 이유만으로 pdf에만 두지 않는다.

본문 첨부 PDF를 기준 PDF로도 쓰면 원본은 samples에, 기준 사본은 pdf 아래에 보존한다. review 문서에는
두 경로, SHA-256 동일 여부, 기준인지 참고 보고서인지를 적는다. 원본 HWP/HWPX가 없으면 독립 시각 검증과
장기 재현이 불가하다는 사실을 review 문서에 명시한다.

## 3.5.1 기준 PDF 미첨부 시 버전별 HWP MCP

PR에 기준 PDF가 없지만 원본 HWP/HWPX가 있으면, PDF 업로드 요청보다 먼저 다음 명령으로 마지막 저장
제품 메타데이터를 확인해 해당 MCP로 기준 PDF를 산출한다.

```bash
rhwp info --json <원본 HWP 또는 HWPX>
```

`lastSavedWith.product`가 `hancom-office-2010`·`hancom-office-2018`·`hancom-office-2020`·
`hancom-office-2022`이면 [HWP 2024 MCP 사용법](../mcp_hwp2024Convert_usage.md)의 통합 Windows
service에서 engine `2020`을, `hancom-office-2024`이면 같은 service의 engine `2024`를 사용한다.
이 판정은 HWP5 `HwpSummaryInformation.revisionNumber`와 HWPX `version.xml/appVersion`의 마지막 저장
메타데이터를 사용한다. 확장자와 파일 포맷 `version`만으로 서비스를 선택하지 않는다.

`lastSavedWith`가 `null`이거나 `product`가 `null`이면 서비스를 자동 선택하지 않는다. 기준 PDF, 제출자·원
저장 환경의 확인 등 별도 근거를 확보하고 그 판단을 review 문서에 기록한다. 이 메타데이터는 원 작성 제품의
증명이 아니며 재저장·삭제·변조될 수 있다.

- 최종 기준 PDF는 output에만 두지 않고 2020 계열은 `pdf/{원본 stem}-2020.pdf`, 2024 계열은
  `pdf/{원본 stem}-2024.pdf`에 저장한다.
- 50MB 미만 MCP 산출 PDF는 commit 가능한 장기 증적이다. 큰 PDF는 pdf-large와 Git LFS 정책을 따른다.
- 서버 URL, IP, 인증 token, .env.local 내용은 GitHub issue·PR·review 문서·로그에 기록하지 않는다.
- 원격 service는 rhwp maintainer, collaborator 또는 MCP 관리자가 별도로 인증한 사용자만 사용한다.
- 원본 크기와 예상 페이지 수를 먼저 확인한다. 페이지가 많거나 거대·중첩 표, 성능 sample은
  timeout_seconds를 900–1800초로 늘린다.
- VS Code MCP 호출이 timeout되어도 서버 job이 성공했을 수 있다. CLI로 재호출해 로컬 PDF 수신까지 확인한다.

통합 Windows MCP는 동기 `status: success` 또는 비동기 `succeeded → success`, 요청한 `--engine`과
비동기 `start`·`status` 응답의 `engine` 일치, client/server byte 수와 SHA-256 일치를 확인한다.
2022 이하 저장본에는 `--engine 2020`, 2024 저장본에는 `--engine 2024`를 명시한다. `server.engine`은
concrete backend 식별자일 수 있으므로 저장 버전별 engine 선택의 판정 기준으로 사용하지 않는다.
`engine_profile`과 `hancom_version`은 서버가 제공할 때만 추가 증적으로 기록하며, 부재만으로 실패로
판단하지 않는다.
공통으로 `pdf/` 아래 실제 PDF 존재와 `file` 또는 `pdfinfo` 확인이 필요하다.
review 문서에는 MCP 선택 전 `info --json`의 `format`·`lastSavedWith` 값, 사용한 서비스 버전, 원본
경로·가능하면 SHA-256·출처 URL, PDF 경로·SHA-256, MCP job id, 서비스별 status·validation metadata·페이지 수,
사용한 visual sweep asset과 지표를 적는다.

## 대표 asset과 안정 URL

visual sweep을 실제 merge 판단에 썼으면 merge 가능 또는 승인 요청 전에 대표 review_NNN.png를
현재 review branch의 mydocs/pr/assets 아래에 PR 번호를 포함한 안정 파일명으로 복사한다.

- review 문서에는 임시 output 경로와 최종 asset 경로를 둘 다 적는다.
- 여러 페이지를 검증해도 모든 PNG를 기계적으로 보존할 필요는 없다. 결론을 증명하는 정상 page와
  보완 요청·후속 issue 판단에 필요한 후보 page를 대표 asset으로 남긴다.
- GitHub merge comment에는 [Visual Sweep 정본](../verification/visual_sweep_guide.md#github-merge-comment)을
  direct link로 남긴다. output 경로 link만 남기지 않고, merge commit에 반영된 asset의 **commit SHA 고정**
  raw URL을 Markdown image로 실제 표시한다. raw URL은 PNG 표시용 증적이며 문서 비교 방법의 인용은
  Visual Sweep 정본과 review 문서가 담당한다.

~~~markdown
- 문서 비교: [PDF/SVG visual sweep 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 따름

![PR N visual review](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/<file>.png)
~~~

### asset 반영 경로

1. **옵션 M — maintainer 직접 운영 기록 반영**: 원 코드 PR merge 뒤 devel을 fast-forward하고,
   archive review·asset·필요한 오늘할일만 한 commit으로 반영한다. source, test, workflow,
   golden/baseline, 기존 sample, 새 LFS 자료는 이 경로에 섞지 않는다.
2. **옵션 1 — 현재 PR head에 함께 포함**: collaborator self-merge 또는 collaborator 매개 외부 PR에서
   archive review·asset·필요 시 오늘할일을 같은 PR branch에 넣는다.
3. **옵션 2 — merge 뒤 후속 기록 PR**: 직접 반영 범위를 넘거나 현재 PR head에 넣을 수 없을 때,
   archive review·asset·오늘할일과 신규 기준 자료만 포함한 후속 PR로 반영한다.

option 2에서도 asset이 devel에 존재하기 전에는 issue/PR comment를 게시하지 않는다. 후속 PR의
branch, worktree, review 전용 target은 merge 뒤 [merge 후속 처리](post_merge.md)에서 정리한다.
