# PR #8659 리뷰 — semanticist21 원 PR 18건 통합

## 최종 판정

**승인 — 원 PR 18건의 주장 범위와 메인터너 보정의 로컬 검증·정확한 code head CI 완료.**

작성: 2026-11-06 11:43:58 KST. 이 문서-only trailing head의 required checks·실제 재사용 판정·mergeability는 push 뒤 확인할 조건이다. 원 PR의 수정 전 head를 승인한 것으로 해석하지 않는다.

## 접수와 변경 범위

- [통합 PR #8659](https://github.com/edwardkim/rhwp/pull/8659), 작성자 `jangster77`, base `devel`. collaborator self PR이므로 reviewer를 지정하지 않았다.
- code head `33cf0803336b2b0cb2bd7ebb82b458c767ef8104`, 검증 base `170b886f167b705b6e630c5b34bb638d92fae0a9`.
- Rust 생산 source `86709ac16711820e61baa3b87e2c4ddaac23573e`, Studio 최종 source `5feccb130e0f695de0046d994c060130fa3e928e`.
- 스타일·번호·각주·구역·글머리표·표·쪽 번호·저장·덧말·원고지의 18개 PR, 원 커밋 44개를 각각 한 번 `-x` 적용했다. [원/적용 SHA 대응](../assets/semanticist21_20261105/historical-validation.md).
- [Issue #8019](https://github.com/edwardkim/rhwp/issues/8019)·[Issue #8628](https://github.com/edwardkim/rhwp/issues/8628)·[Issue #7635](https://github.com/edwardkim/rhwp/issues/7635)의 요구와 실제 API·독립 출력의 해결 범위를 대조했다. 최종 원격 close는 merge·devel 포함 뒤 처리한다.

## code head GitHub Actions

아래는 같은 PR·source branch·위 code SHA에서 실제 완료된 실행이다. 로컬 성공이나 다른 PR 실행으로 대체하지 않았다.

| 실행 | 실제 head | 결과 |
| --- | --- | --- |
| [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/42149217187) | `33cf0803336b2b0cb2bd7ebb82b458c767ef8104` | success |
| [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/42149217286) | `33cf0803336b2b0cb2bd7ebb82b458c767ef8104` | success |
| [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/42149217508) | `33cf0803336b2b0cb2bd7ebb82b458c767ef8104` | success |
| [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42149217251) | `33cf0803336b2b0cb2bd7ebb82b458c767ef8104` | success |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42149217269) | `33cf0803336b2b0cb2bd7ebb82b458c767ef8104` | success |
| [CI](https://github.com/edwardkim/rhwp/actions/runs/42149217190) | `33cf0803336b2b0cb2bd7ebb82b458c767ef8104` | success |

heavy worker와 aggregate를 포함한 완료 check를 조회했고 실패·취소·진행 중인 필수 실행이 없다. source/test 변경 없이 문서-only trailing을 잇고, 그 head의 preflight와 required check를 다시 확인한다.

## 로컬 검증·조판 원칙

[실제 명령·실행 결과·재사용 근거](../assets/semanticist21_20261105/validation-r6/final-validation.md)를 정본으로 사용한다.
전체 Rust는 최소 공개 글꼴 5개에서 eligible 12,740건 PASS·기존 ignored 45건, 목록 누락·중복 0이다.
성능 1건과 정확한 보집합 12,739건을 분리 실행했다. 공유 `target/pr-review`, 빌드 jobs 8·test threads 12로 순차 실행했다.
fmt·Native/WASM/workspace all-target Clippy·workspace/release build·release lib·doctest·Native Skia lib 2건/누락 그림 2건/직접 PDF 22건·정책 검사도 통과했다.

Studio Node 2,744건 PASS, production build, 실제 API 12그룹·저장 bytes 30개·독립 CFB·생성 타입 631개×2와 소비 10건을 확인했다.
실제 Canvas2D·CanvasKit compatibility/default의 screen/print 12캡처, 웹폰트 브라우저 7시나리오, 최소 글꼴 PageRenderer 관계 검사도 통과했다.
회귀 환경과 시각 측정의 전체 시스템 글꼴 환경을 구분했다. 글꼴 프로그램·신규 다운로드 URL은 추가하지 않았다.

공통 측정/배치·줄 소속·후속 본문·저장과 편집 재조판을 [통합 분석](../assets/semanticist21_20261105/integration-plan.md)에서 추적했다.
빈 번호 머리의 50% 본문 간격과 Studio 기호 후보 순서는 공통 경로에서 보정했다. 특정 문서 좌표·golden 완화로 맞추지 않았다.
회귀는 관계 검사를 우선하고 [독립 Print 및 양쪽 raw 98% 이상을 확인한 17개 완료 페이지](../assets/semanticist21_20261105/validation-r6/qualification.md)의 적격한 기존 좌표 검사만 유지했다.

## 입력·전쪽 TSV·확대 판독

- [같은 원본·독립 Print PDF SHA·전쪽 Native/fresh WASM TSV](../assets/semanticist21_20261105/validation-r6/measurements.md): 각 128입력·301쪽, 쪽수 일치·누락 0·전쪽 90% 이상. 최저 Native 91.81401%, fresh WASM 91.17272%.
- [새 base와 전쪽 대조](../assets/semanticist21_20261105/validation-r6/base-comparison.md): 각 301쪽의 raw 하락 자체가 0쪽.
- [PR #8389·#8397의 보호 TSV 34개](../assets/semanticist21_20261105/validation-r6/protected/README.md): 각 17개 고유 문서·1770쪽, 공개 중복 보호 행 포함 2153행/backend에서 반올림 표시값 하락·누락·오류 0.
- 공식 Native 편람 HWPX 383쪽 gate min 99.96%, 공개 기준 하락 0·exit 0. 실행 뒤 Rust 생산 코드·입력·PDF·binary·측정 도구가 같아 재사용했다.
- [같은 주장 페이지의 review·overlay와 직접 판독](../assets/semanticist21_20261105/validation-r6/panels.md). 원본·버전·페이지를 맞췄고 원 PR마다 개별 리뷰에 연결했다.
- [실제 Studio 두 원본 1쪽 review·overlay](../assets/semanticist21_20261105/studio-symbol/README.md)는 Canvas2D raw 각각 100.00%다. 컬러 이모지와 반각 기호의 크기 차이를 독립 Print로 발견해 전각 후보 소비를 보정했다.
- PR #8564의 파일명 필드는 Print 임시 파일명 재평가와 저장 문자열이 다르다. 원 결함인 필드 앞뒤 입력 보존은 실제 API base FAIL→후보 PASS로 확인했다. raw 100%를 내용/RGB 완전 동일로 표현하지 않았다.

## 개별 PR 판정과 후속 처리

[PR #8526](pr_8526_review.md) · [PR #8527](pr_8527_review.md) · [PR #8528](pr_8528_review.md) · [PR #8536](pr_8536_review.md) · [PR #8537](pr_8537_review.md) · [PR #8538](pr_8538_review.md) · [PR #8541](pr_8541_review.md) · [PR #8549](pr_8549_review.md) · [PR #8558](pr_8558_review.md) · [PR #8564](pr_8564_review.md) · [PR #8570](pr_8570_review.md) · [PR #8571](pr_8571_review.md) · [PR #8572](pr_8572_review.md) · [PR #8577](pr_8577_review.md) · [PR #8586](pr_8586_review.md) · [PR #8602](pr_8602_review.md) · [PR #8616](pr_8616_review.md) · [PR #8629](pr_8629_review.md)

각 개별 리뷰의 실제 입력·페이지·TSV·코멘트 계획을 기준으로 merge 뒤 결과를 게시한다. 원 PR head와 통합 보정 head를 구분하고, 기여자 fork branch는 보존한다.

원 PR 18건·관련 이슈의 close, merge SHA와 devel 포함 확인, nextest duration 갱신 상태 확인, 검토 전용 branch/worktree 및 output·종료된 debug 산출물 정리를 순서대로 수행한다. 확정 merge SHA·원격 close는 아직 실행하지 않은 조건이며 PR 타임라인·코멘트에서 실제 결과를 확인한다. 이 기록이 포함된 뒤 추가 source 변경이 없으면 별도 운영 기록 PR은 필요하지 않다.
