# PR #7460 사전 판정 보고서

## 최종 판정

**승인.** 원 code head의 serializer 형식 수정과 관련 증거가 수용 조건을 충족합니다.
원격 Approve와 merge는 완료되지 않았으며 각각 별도 승인·최신 head 확인이 필요합니다.

## 해결 범위와 근거

빈 HF 문단의 저장 문자 수 0→1과 출력 버전 5.0.3.2 이상에서의 문단 헤더 22→24바이트 보완입니다.
원 PR source/test/baseline 5개 파일을 검토했고, 저장 회귀 5개와 IR field sweep 4개가 통과했습니다.
같은 저장 회귀의 수정 전 대조는 4개 실패·1개 통과입니다.
한컴뷰어·Chrome 웹한글기안기에서 세 생성 사례 모두 수정 전 실패·수정 후 성공과 본문 보존을 확인했습니다.

[검토 문서](pr_7460_review.md)에 호출 경로, 조판 원칙 판정, 정확한 SHA·CI job·명령,
baseline 변경 근거와 실제 비교 이미지 7개를 연결했습니다.
[증적 README](../assets/pr_7460/README.md)에 실행 입력 8개와 SHA-256을 보존했습니다.
commit `f21bac8051852f7417d917e9acb5e6de1c2a95d9`의 입력 blob을 실제 실행 파일과 대조했습니다.
새 입력과 기존 샘플의 커밋 포함 게이트는 충족입니다.

## Route A와 기여자 credit 보존

대상은 [원 PR #7460](https://github.com/edwardkim/rhwp/pull/7460),
`jeong-sik/rhwp:fix/empty-header-hwp5-pr`입니다. 원격은 clean이고 `maintainerCanModify=true`이므로
별도 integration PR 없이 collaborator Route A를 사용합니다.
원 기여자의 아래 네 커밋·작성자·기존 trailer와 PR credit을 그대로 보존합니다.

| 원 기여자 commit | 내용 |
| --- | --- |
| `0b3e3ee29f2e8a088266978376bbe1a21179c051` | valid empty header/footer paragraphs |
| `ceeae1c01cd075e86831ec7dbe0af3355b56f947` | normalize empty counts only when saving |
| `aa70b6eaf7b9237b3f54ef3ff382c2aee3a05e4d` | versioned paragraph header test |
| `bf50bc7df46ff9df566e407291fddad0155a9bd6` | integration test formatting |

로컬 검증 head `7e47b085e5f744fa6bf632c84f68b47391342aed`의 체리픽 이력은 실행 증거로만
사용합니다. 문서 branch는 원격 원 code head `bf50bc7d…`에서 직접 시작했고,
추가 커밋은 `mydocs/pr/archives`의 review/report와 `mydocs/pr/assets/pr_7460`에만 한정합니다.
원격 push는 contributor의 원 branch를 대상으로 하며 force-push를 사용하지 않습니다.

## 게시·병합 전 조건

현재 최신 base는 `9954daf7ee04adb8f7dbd5a66df371f5fd8d4040`입니다.
문서 작성 전 source head와 이 base의 `git merge-tree --write-tree`는 충돌 없이 통과했습니다.
정책 문서는 당시 검증 base와 동일하고, 새 검색/IME 변경은 HF serializer 계약에 영향이 없음을 대조했습니다.

1. 문서와 게시할 Approve 본문을 사용자에게 제시하고 push·Approve 게시 승인을 각각 받습니다.
2. push 직전 최신 원격 base/head, 파일 범위, single-parent 이력, 입력 blob 동일성,
   LFS 대상, dry-run, 실제 merge tree의 공백·링크·기존 오늘할일 보존을 확인합니다.
3. 문서 커밋만 원 PR head에 정상 push하고 원격 exact 새 head와 PR diff를 재조회합니다.
4. exact 새 head의 CI/preflight/CodeQL 등 관련 check와 mergeability를 확인합니다.
   review-only fast-pass는 자동 판정 결과가 나올 때만 통과로 기록합니다.
5. 확인받은 본문을 그 exact head에 Approve로 게시하고 API로 내용·상태·commit_id를 확인합니다.
6. merge는 별도 사용자 승인 뒤 최신 상태를 다시 확인해 진행합니다.

오늘할일 파일은 이 원 source head에 없고 최신 devel에 별도로 존재하므로 문서 tail에 복사해
add/add 충돌을 만들지 않습니다. 이번 검토 사실은 review/report에 기록합니다.
정확한 문서 tail head와 최종 preflight 결과는 push 승인 시 제시하며,
이 보고서에 미래 CI 성공·원격 게시·merge 사실을 미리 기록하지 않습니다.

## 미검증 범위와 후속 확인

확인한 것은 합성 HWP5의 열기 성공과 본문 보존입니다. 여러 쪽의 짝수·홀수 표시,
한컴 PDF 기준 조판 일치, 무편집 손상 raw stream의 자동 복구, Studio UI E2E는 미검증입니다.
Visual Sweep/fresh Studio WASM은 serializer-only 변경 범위에서 비해당입니다.

실제 merge 후에는 검토 문서의 contributor comment 계획에 따라 merge SHA 고정 대표 이미지를
댓글에 표시하고 CI·PR·관련 [#7459](https://github.com/edwardkim/rhwp/issues/7459)의 최종 상태를
재조회합니다. PR 본문의 `Closes #7459`가 실제 자동 종료로 연결됐는지 확인하며,
수동 종료가 필요하면 별도 승인을 받습니다.
