# PR #6843 검토 기록

## 판정: 메인터너 보정 후 수용 가능

새 fixture의 layout 래칫 안내에 보안 코퍼스 입력 전달 절차를 보완했다. 보정은 로컬 문서에 반영 완료했으며 소스 코드나 baseline을 완화하지 않았다. 통합 PR의 원격 CI·병합은 별도 절차다.

## 출처와 기준

- 검토일: 2026-09-07.
- 원 PR: [#6843](https://github.com/edwardkim/rhwp/pull/6843), 작성자 `planet6897`.
- 원 커밋: `f4b56e0329b59f2ec6e1ef5f24bc029da068ecec`, `ac5743ae342ec96a4e0508974d6747ad9dc847a0`.
- 로컬 체리픽: `4154319b2`, `5aaccdb72`.
- 원 최신 head는 두 번째 커밋이며 이번 API 확인에서도 동일하다.
- 브랜치: `review/ci-green-batch-20260907-2`.
- 기준 devel: `a3a30d99d4aefb15ed0dbed96645eb1ca3595099`.
- 실행 검증 코드 HEAD: `2c864284557caba4b97d07f104ce957bb756311f`.

## 보정 내용

[local_validation.md](../../manual/pr_review/local_validation.md) 4.3.1절에 다음을 명시했다.

- 여섯 layout/시각 래칫과 새 문서 보안 코퍼스 검사는 별도 게이트다.
- `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 검토 대상 sample의 실제 경로 목록을 전달한다.
- 환경변수 미설정이나 검사 대상 없음은 새 문서 검사 통과가 아니다.
- 추가·수정·이동 및 아직 커밋하지 않은 작업 소유 fixture를 누락하지 않는다.
- 탐지 발생 시 원인을 조사하며 목록 제외나 무차별 allowlist로 숨기지 않는다.
- 새 래칫 실행 예시에 `--locked`, `--target-dir target/pr-review`, `--tests`를 명시했다.

클리핑 입력 92건이 모두 누락되었을 때 exit 0을 검사 성공으로 오인하지 않는 원 PR의 취지는 유지했다. 그 92건 자료를 이번에 전부 확보하거나 검증한 것으로 기록하지 않는다.

## 실제 입력 전달 검증

이번 batch에는 신규 sample이 없으므로 다음 기존 문서 3개를 사용해 보안 검사 입력 전달 경로를 점검했다. 신규 fixture 3개를 추가한 것이 아니다.

- `samples/issue6697/80550-agricultural-machinery-act-amendment.hwpx`
- `samples/issue-6271-rowbreak-float-tail-line.hwp`
- `samples/issue6551/113424_evaluation_guideline.hwpx`

위 경로를 JSON 배열로 환경변수에 넣고 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`에 집중 필터를 적용했다.

결과: #6808·#6809·보안 코퍼스 합계 23개 통과, exit 0. 그중 `new_sample_documents_are_clean_across_all_three_detectors`가 실제 목록을 받아 통과했고, 악성 양성 통제군·allowlist 탐지 확인도 통과했다. 필터 제외 9,217개는 미실행이다.

기존 통합 코드의 전체 회귀는 9,194개 통과·46개 건너뜀, exit 0이며 fmt, native/WASM clippy, workspace build, workspace all-targets clippy, suite manifest check도 통과했다. 문서 보정 후 전체 회귀를 다시 실행한 것으로 표시하지 않는다.

## 원 CI 및 증적 처리

원 head의 [Build/Test](https://github.com/edwardkim/rhwp/actions/runs/34120214744/job/101736394773)는 intake에서 성공했다. docs 분류에 따른 worker skip은 전체 렌더러 검증 실행과 구분한다.

문서 절차 개선이므로 별도의 시각 자료는 만들지 않았다. 실행 로그·JSON을 커밋하는 대신 입력 범위와 실제 결과를 이 문서에 남긴다. 링크 대상 안내와 실행 경로는 기존 문서를 유지하며 본문 게이트를 보완했다.

## 병합 후 코멘트 계획

통합 PR이 승인·병합되고 실제 devel CI가 성공한 뒤 source SHA, 통합 merge SHA, CI 링크, 메인터너의 보안 검사 절차 보완 내용을 원 PR에 기록한다. 원 PR 본문의 closing references와 관련 이슈의 실제 상태를 확인해 `post_merge.md`를 따른다. 참조된 이슈 번호만으로 임의 close하지 않는다.

기존 후속 코멘트가 있으면 새로 등록하지 않고 수정한다. UTF-8 body file로 작성하고 API로 본문을 재조회한다. 이번 검증은 시각 자료가 없으므로 이미지가 있다고 주장하지 않는다. 현재 원격 변경은 하지 않았다.

## PR 제출 시점 확정 기록 (2026-09-07)

- 통합 브랜치: `review/ci-green-batch-20260907-2`.
- 원본 체리픽 누적 head: `2c864284557caba4b97d07f104ce957bb756311f`.
- 메인터너 코드 보정 및 검증 대상: `fe6d157c0044dd4b402edd3b6435ad0f6edb7c6b`.
  앞선 준비 단계의 미커밋 표기는 당시 상태이며, 보정 코드는 이 커밋으로 확정했다.
- 전체 Rust 9,194개 통과/46개 skip은 보정 전 결과다. 보정 후 Rust 집중 12개,
  Studio 1,493개 통과/2개 skip, TypeScript, fmt, native/WASM lib/workspace all-targets clippy,
  workspace build와 manifest 검사를 완료했다. 보정 후 전체 Rust 회귀는 재실행하지 않았다.
- 새 WASM 제품 빌드와 브라우저 실동작 검증은 미실행이다. WASM lib clippy 및 Studio 테스트를
  실제 새 WASM 제품 실행 결과로 대체하지 않는다. 최신 통합 PR CI는 제출 뒤 확인해야 한다.
- 시각 증적 게시 절차는 [Visual Sweep의 GitHub merge comment 규약](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다.
  위 증적의 실제 페이지·측정 범위만 인용하고, 미측정 flagged/시각 정확도 수치를 임의로 만들지 않는다.
  메인터너 보정 후 실제 Undo 화면을 새로 촬영한 증적은 없으며, 기존 동일 값 setter 대조 PNG를 그 증적으로 주장하지 않는다.
- merge 이후에는 이 문서에 기록한 대표 PNG를
  `https://raw.githubusercontent.com/edwardkim/rhwp/<실제-merge-SHA>/<해당-PNG-저장소-경로>`로 Markdown 이미지에 직접 삽입한다.
  자산이 devel에 포함되고 실제 devel CI가 성공한 뒤 UTF-8 body-file로 게시하고 API로 본문을 재조회한다.
  동일 목적 댓글이 있으면 이전 댓글을 수정하며 중복 등록하지 않는다. 원 PR의 직접 merge나 전체 관련 이슈 해결로 표현하지 않는다.
