# Issue #6389 단계 3 — PR 제출 준비와 최종 검증

Issue: [#6389](https://github.com/edwardkim/rhwp/issues/6389)

## 분석

- 사용자 지시: PR 준비 후 PR 생성. 추가 승인 질문 없이 준비·push·생성까지 진행한다.
- 후보 code head: `4406687d8c38124fd73e32245f896152c0992526`.
- 최신 upstream/devel: `5720d3f1646d6c25e51c8d7cbb0b4cc6bc5feb4f`, 후보는 2 commits ahead / 0 behind.
- 두 commit은 1단계 실제 PDF 줄 경계 검증, 2단계 명시적 폰트 환경 구현과 증적이다.
- 이번 회차는 생산 코드 수정 없이 최종 전체 테스트 결과와 PR 제출 기록을 정리한다.
  2단계 전체 검사에서 분류 누락을 수정한 뒤 가드만 재실행했으므로, PR 제출에는 수정 포함
  전체 실행 결과를 다시 남긴다. 이미 동일 소스로 통과한 lint·WASM·Skia·Visual Sweep은
  소스 일치를 확인해 재사용한다.
- API의 한정된 범위와 PDF 잔여 차이를 구분한다. #6389 전체 해결을 주장하지 않으며 Refs를 쓴다.

## 검증 계획

1. review worktree의 변경 source/test/script/fixture를 후보 Git blob과 대조한다.
2. 전체 release-test, 고정 base 정책 검사, diff/문서 링크·metadata를 확인한다.
3. 실제 결과를 기록하고 사용자에게 보고한 뒤 이 회차 문서·증적을 커밋한다.
4. 원본 upstream의 작업 branch로 push하고 devel 대상 Open PR을 생성한다.
5. 실제 PR 번호로 self-review·오늘할일을 작성하고 검증·보고 후 같은 PR에 문서 commit을 추가한다.

## 실행 결과

- 후보 commit의 변경 source/test/script/fixture 24개를 검토 worktree와 대조했고 모두 바이트가 같았다.
- 검증에 사용한 HWP/PDF 5개도 후보 Git blob과 바이트가 같았다. 신규 복제 입력은 없다.
  [입력별 SHA-256/SHA-1와 PDF 메타데이터](assets/issue6389-stage3-20260916/inputs.json).
- KoPub 비교 PDF는 Producer `cairo 1.18.0`, PDF 1.7, 383쪽이다. 기존 #6389에서 사용한
  비교 자료로 재사용했지만, 현재 파일 메타데이터만으로 한컴 직접 출력본 또는 원 출력에서의
  변환 계보를 확정하지 않는다. 이전 단계의 “한컴 PDF” 표현은 이 출처 제한과 함께 읽는다.
  no-ttf는 Creator/Producer `Hancom PDF 1.3.0.404`, PDF 1.4, 389쪽이고, 86712는
  Creator `Hwp 2024 13.0.0.3622` / Producer `Hancom PDF 1.3.0.550`, PDF 1.6, 65쪽이다.
- 최종 전체 release-test: **9,905 passed, 0 failed, 51 skipped**, 274.309초, exit 0.
  [실제 최종 실행 로그](assets/issue6389-stage3-20260916/full-regression-final.log).
- 고정 base suite 정책 검사 통과: 1,338 sources / 5,775 static attrs / 48 integration targets.
  [정책 로그](assets/issue6389-stage3-20260916/suite-policy-final.log).
- 동일 생산 소스의 기존 3종 Clippy·fmt·workspace·fresh WASM·Skia 결과를 재사용했다.
  source-side unit test, Studio/npm, workflow 변경은 없다.
- 새 문서 링크와 전체 PR diff 공백 검사를 통과했다. 원본 저장소 push 권한 true 및 동일 branch의
  기존 Open PR 없음, 최신 base와 merge-tree exit 0을 확인했다.
- 결과를 사용자에게 먼저 보고했다. 이 회차 문서·입력 원장·최종 검증 로그를 함께 커밋한 뒤
  upstream 작업 branch로 push하고 PR을 생성한다. PR 번호는 생성 후 기록한다.

## 원격 제출과 self-review 기록

- 준비 결과 commit `60a3ad32e`를 upstream 작업 branch에 push하고
  [PR #7179](https://github.com/edwardkim/rhwp/pull/7179)를 devel 대상 Open PR로 생성했다.
- PR API 본문이 준비한 UTF-8 Markdown과 같음을 확인했다. 작성자 self PR이므로 reviewer를 지정하지 않았다.
- 실제 번호로 [self-review](../pr/archives/pr_7179_review.md)와
  [오늘할일](../orders/20260916.md)을 작성했다. 기존 WASM 대표 PNG 2개를 PR assets로 이동하고
  2단계 링크를 갱신한다. 원본·기준 HWP/PDF는 이동·복제하지 않았다.
- 이 후속 commit은 review·오늘할일·asset 이동·보고서 링크만 포함한다. GitHub CI는 최신 head에서
  별도 확인해야 하며 merge·issue close는 수행하지 않는다.
