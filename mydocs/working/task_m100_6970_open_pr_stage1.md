---
kind: snapshot
status: active
canonical: mydocs/working/task_m100_6970_open_pr_stage1.md
last_verified: 2026-09-16
---

# 열린 PR 8건 누적 체리픽과 검토 — 1회차

## 분석

- 요청: draft를 제외한 열린 PR을 체리픽하고 개별 검토한다. 충돌은 메인터너가 보정한다.
- branch: `codex/open-pr-review-20260916`; base: `8d45f242baa1a565357aaa38e9f459595b1e756c` (`upstream/devel`).
- 대상: #7118 → #7180 → #7181 → #7183 → #7184 → #7185 → #7186 → #7187, 원본 10 commits.
- 원 PR마다 reviewer `jangster77` 요청을 확인했다. 작성자는 모두 기존 기여자다.
- 기본 경로는 `collaborator_external_pr`이며 사용자가 누적 체리픽을 명시했다.
  intake/local_validation/multi_pr_update_branch/visual_fixture_evidence 절차를 적용한다.
- #7118은 최신 devel과 충돌한다. 저장 LineSeg 없는 Square 재조판과 최신 FloatLane 객체 소유권을
  동시에 보존해야 한다. 기존 tests 루트 파일은 현재 manifest에 등록되어 있음을 확인했고 유지한다.
- HWP3 PR 4건은 동일 parser를 수정하므로 후속 적용에서 앞 PR의 변환 규칙을 보존한다.
- 표 분할 #7183/#7184는 최근 #7188에서 수정한 컷·물리 높이·그림 소유 경로와 함께 대조한다.
- #7186은 명령 레지스트리와 대화상자 탭 번역 범위다. 한국어·영어 실제 소비 경로를 검토한다.
- 원 head CI 성공은 누적 head 검증을 대신하지 않는다. 전용 target에서 focused 검증을 수행하고,
  시각 변경은 동일 입력·동일 페이지의 한컴 PDF와 직접 Visual Sweep을 비교한다.
- 사진/텍스트 존재·페이지 수만으로 시각 통과를 선언하지 않는다. 필수 증거가 없거나 코드상
  규칙 위반이 남으면 개별 PR을 보류하고 실행 결함과 증거 부족을 구분한다.
- 입력과 기준 PDF는 기존 Git 경로를 먼저 확인한다. 같은 파일을 이름만 바꿔 중복 추가하지 않는다.
- 원본 replay는 작성자·메시지와 `-x` provenance를 보존한다. 새 보정은 분석·검증·결과보고 뒤 커밋한다.
- 이 회차에는 원격 push/통합 PR 생성/원 PR close를 수행하지 않는다.

## 적용·검증 결과

원본 10 commits를 누적 적용했다. code head는 `2a2089bf71e7a42286e0643a3fb517b7047c7320`이다.
정확한 source/local 매핑은 아래 개별 impl 문서가 정본이며 실제 Git SHA를 사용한다.

| PR | 개별 판정 | 핵심 결과 |
| --- | --- | --- |
| [#7118](../pr/archives/pr_7118_review.md) | 머지 보류 | 실행 프레임 회귀, 2쪽 글자 겹침, 6% slack/배너 높이 덮어쓰기 |
| [#7180](../pr/archives/pr_7180_review.md) | 승인 | 실제 3,699 문단 여백/indent 한컴 일치 |
| [#7181](../pr/archives/pr_7181_review.md) | 승인 | 각주 위/아래/사이 852/568/424 저장값 일치 |
| [#7183](../pr/archives/pr_7183_review.md) | 승인 | 11개 그림 복원·소유 셀 bbox, Native/WASM 4쪽 |
| [#7184](../pr/archives/pr_7184_review.md) | 머지 보류 | 48쪽/19쪽 넘침 개선, 새 예약의 실제 end-cut/종결 경계 미검증 |
| [#7185](../pr/archives/pr_7185_review.md) | 승인 | suffix 41/start 1 및 한컴 번호 표시 확인 |
| [#7186](../pr/archives/pr_7186_review.md) | 승인 | 두 언어 팔레트 191개·문단 탭·unit/build/E2E |
| [#7187](../pr/archives/pr_7187_review.md) | 승인 | 인라인 탭 확장 112/112 byte 일치, 목차 점끌기 확인 |

**통합 head 전체는 머지 보류**다. 위 6개 승인은 각 PR이 약속한 변경 범위의 검토 의견이다.
새 통합 head의 최종 전체 게이트를 통과했다는 의미가 아니며 remote push/PR/merge를 하지 않았다.

### 실제 실행

- 모든 Cargo 실행은 순차, 전용 `target/pr7118-7187-20260916` 사용.
  shared target과 기존 다른 review 산출물은 보존했다.
- 최초 build는 Xcode 라이선스 미동의로 실패(exit 101). CLT로 `DEVELOPER_DIR`만 지정해
  release-test CLI 빌드 exit 0(3분 8초). 라이선스 동의나 전역 Xcode 설정 변경 없음.
- focused 13 case 그룹, 54 tests: **53 passed / 1 failed / 2018 filtered**.
  실패는 #7095 그림 투영 프레임 계약으로 937.80px 대 PDF 931.48px(허용 ±2px).
- 별도 소유 worktree에서 A/B: #7184 runtime patch만 제거하면 동일 실패(5/6),
  #7184를 되살리고 #7118 runtime patch만 제거하면 6/6 통과.
  어느 조합도 통합 수정본으로 commit하지 않았다. 대조 로그는 review assets에 있다.
- fmt check exit 0; base 고정 unit tier 4205 tests/298 modules 통과.
  fixture 경로 변경으로 최초 manifest check는 generated drift. prepare 재실행 후
  1345 sources/48 targets, base 비교 통과. 생성 harness/inventory는 commit 제외.
- fresh dev WASM exit 0. Native 및 WASM sweep: #7118 1–3쪽, #7183 1–4쪽,
  #7184 18–21·47–48쪽. 대표 PNG를 실제 열어 판정했다. 자동 후보 0건을 완전 fidelity로 보지 않았다.
- HWP3 원본·누적 저장본을 각각 MCP 2020으로 출력: 모두 264쪽.
  기록의 3,699 문단은 텍스트를 대응시켰고, 문단 수가 다른 두 문서를 순번 zip한 잘못된 수치는 제외했다.
  기존 Git before 저장본과의 차이는 과거 자료 참고이며 이번 base에서 만든 출력이라고 하지 않는다.
- HWP3 PDF 비교는 Visual Sweep `make_compares`로 두 한컴 PDF를 직접 합쳤다.
  4쪽 목차·9쪽 각주·254/255쪽 참고문헌을 확인했다. Native renderer fidelity 검사가 아니다.
- Studio 일반/CI unit tsc exit 0, unit 1740 pass/2 skipped, production build exit 0,
  command-palette E2E 성공. 영어/한국어 Chrome 팔레트 각 191개, 영어 한글 라벨 0개,
  내부 tab ID 유지와 표시 번역을 실제 확인했다.
- 전체 release-test, Native Skia 3종, Clippy 3종은 **미실행**. 선행 실행에서 blocker가 남았으므로
  비용이 큰 최종 제출 게이트로 넓히지 않았다. 보류 해소 후 최종 head에서 수행해야 한다.

### 증적 보존과 fixture

[공통 증적](../pr/assets/pr7118_7187_review/fixture-manifest.json)은 입력/PDF SHA-1·SHA-256,
PDF Creator/Producer/version/페이지를 기록한다. 동일 이름뿐 아니라 동일 바이트를 확인해 중복을 피했다.
새 입력은 #7183 원본(원 cherry-pick), 누적 CLI로 만든 HWP5 저장본 1개와 새 한컴 PDF 4개다.
원본 HWP3·한컴 HWP5 기준·#7118 익명화본은 기존 Git 경로를 그대로 재사용했다.
`pdf/` 새 파일은 모두 50MB 미만이다. MCP start→status→download 영수증도 보존했다.
인증 토큰·서버 URL·환경 파일 내용은 출력/증적/commit에 넣지 않았다.

### 잔여 사항

#7118의 의미적 회귀와 일반성 문제, #7184 새 컷 예약의 필수 경계 증거를 별도 해제 조건으로 기록했다.
이번 요청의 cherry-pick 및 충돌 보정은 완료했고 원격 PR 생성·close나 승인 comment는 하지 않았다.
사용자가 시각 최종 승인을 한 것으로 간주하지 않는다.


### 메인터너 보정 분석

#7118이 추가한 samples 입력은 이미 Git에 있는 tests/fixtures/issue_6970 입력과
SHA-256 31a5b76148718d92e3fd150f4d68b84b5005b02d16863c4735a6178eb02f2799로 같다.
새 복사본을 제거하고 테스트가 기존 경로를 읽도록 보정한다. 새 경로 생성은 필요 없다.

### 결과보고와 커밋 범위

검토 문서 17개의 메타데이터, fixture README를 포함한 18개 문서의 링크 검사를 통과했다.
작업 diff·staged diff·누적 source diff의 공백 오류 검사도 통과했다.
A/B 대조 실행의 종료 코드와 실행 중 Cargo/Rust 프로세스가 없음을 확인한 뒤,
이번 작업이 만든 `/private/tmp/rhwp-open-pr-ab-20260916` worktree만 제거했다.
기존 작업 worktree는 보존했다. 개별 검토·실행 증적·새 PDF/저장본과 중복 fixture 보정을
동일한 후행 커밋으로 보존하며, 통합 코드의 머지 보류 판정은 그대로 유지한다.
