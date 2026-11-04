# PR #8588 리뷰 — duration API 게시·최종 fast-pass 검증과 편람 글꼴 환경 전달

## 최종 판정

**승인 — 정확한 code head의 CI와 병합 후 GITHUB_TOKEN 실제 게시를 확인했다.**
[원 PR #8588](https://github.com/edwardkim/rhwp/pull/8588)의 code head는
`b02f6a4c0500b08ebf3c91290ae9ef2a32fa5bbe`, squash merge는
`056652594ba2cd1a515c9a5def1f80c8e1275064`다. 최신 head의 required check 전체 성공과
`MERGEABLE / CLEAN`을 재확인하고 일반 merge했다. admin·보호 규칙 우회는 하지 않았다.

CI 실행 정의·수집기를 수정한 PR이므로 옵션 2의 별도 기록 PR로 이 review와 오늘할일을 반영한다.
원 PR에 문서 trailing commit을 추가하거나 병합 후 검증 CI를 재실행하지 않았다.
기록 PR의 실제 fast-pass·required check·최신 mergeability는 별도로 확인한다.

## 접수 정보와 범위

- 작성자 `jangster77`의 collaborator self-review이며 reviewer를 자동 지정하지 않았다.
- 제출 base `96827b3accd98ad0b80f76260b42fba24eded504`, 실제 CI·병합 base
  `140cb02b7fd6184e58209b29383ddaed18e49457`, tested merge `a30ccb63fdc1c4da2ca941c0ac82151eaf608191`.
- [#8587](https://github.com/edwardkim/rhwp/issues/8587)의 게시 경로·최종 승인과 측정 출처 구분을 보완한다.
  [#8533](https://github.com/edwardkim/rhwp/issues/8533) 전체를 재종료하거나
  별도 [#8578](https://github.com/edwardkim/rhwp/issues/8578)의 클릭·Esc를 해결한 것으로 표시하지 않는다.
- 사용자 지정 ubuntu-ted `a4fb27f651e17dc1b238b93005086fc165fe9e1b`를 출처와 함께 `4c892a9886`으로
  체리픽했다. 그 원격 작업장은 읽기만 했으며 다른 미완료 커밋을 적용하지 않았다.
- base route: `collaborator_self_merge.md`.
- modifiers: `intake_and_review.md`, `review_template.md`, `local_validation.md`,
  `visual_fixture_evidence.md`, `review_only_fast_pass.md`, `post_merge.md`.
- loaded documents: 위 문서와 `AGENTS.md`, `CLAUDE.md`, `pr_review_workflow.md`,
  문서·Git·GitHub 운영 및 Visual Sweep 정본. 기록 시각 2026-11-04 KST.

## 구현·조판 원칙 준수 검토

Git Data API publisher는 정책 파싱·크기·blob 식별자를 확인하고 같은 데이터는 새 commit 없이 끝낸다.
기존 tree 보존·metrics HEAD 단일 부모·`force:false`로 게시하며 게시 직전 devel·metrics 전진을 확인한다.
ref 경쟁은 최신 자료를 보존하고 응답 유실은 원격 후보 ref로 확인한다. 서버·권한 오류를 정상 보류나
성공으로 숨기지 않고 무조건 retry도 하지 않는다. 두 ref 확인은 원자적 트랜잭션이 아니며 해당 한계를 유지한다.

최종 head 최신 CI의 성공·신원·attempt를 확인한 뒤 허용된 review-only tail을 증명해 실제 worker를 연결한다.
최종 CI 실패·진행 중·재실행·다른 fork·중간 source 변경은 과거 녹색으로 대신하지 않는다.
최종 승인 run/head/attempt와 측정 code run/head/실제 worker attempt/artifact를 결과에 따로 기록한다.
문서-only 생략 worker는 측정 없음으로 설명하고 다음 후보를 확인한다.

편람 변경은 Native 전용 gate의 `--font-environment` 전달 누락이다. 단독·일반 대상 뒤·PNG pair 뒤의
모든 호출이 요청 환경을 공통 출력기에 전달한다. 옵션 생략·Native 전383쪽·Print96 DPI·90%·#8389의
페이지별 하한은 그대로다. Rust·WASM renderer·본문/글꼴 폭·분할 계산·일반 대상 환경 전달은 바꾸지 않았다.
이 PR에 새 px 좌표·문서 배치 핀은 없으며 Python 인자 계약과 실제 Native gate를 검사했다.

## 로컬 검증과 입력·생산 신원

[작업 기록](../../working/task_m100_8587_stage1.md), [검증 JSON](../assets/issue8587/validation.json),
[파일 hash](../assets/issue8587/sha256.tsv)에 실제 명령·source hash·프로그램 공급과 한계를 연결했다.

- Node 수집·게시·artifact·정책: 157 PASS / 0 FAIL.
- workflow 계약 4모듈: 45 PASS / 0 FAIL, actionlint PASS.
- Visual Sweep Python 전체: 247 PASS / 0 FAIL, 변경 문서 상대 링크·`git diff --check` PASS.
- 실제 준비 shell: CI origin의 임시 Git 문맥에서 exit0·46-target JSON SHA 일치.
- 실제 #8583 API로 최종 trailing `41808163341` → code CI `41802090733` → B–E 실제 attempt1을 확인했다.
- 실제 동일 자료 publisher 재조회는 `unchanged`이며 추가 쓰기가 없었다.

Rust·제품 source 변경은 없어 GitHub 운영 O3의 command·workflow 검증을 로컬에서 실행했다.
이 PR의 새 로컬 Cargo·WASM build로 표시하지 않는다. 제품 전체 검증을 생략한 CI라는 뜻도 아니다.
아래 원격 CI에서는 최신 head로 실제 Native/WASM Clippy·workspace build·Native Skia·A–E가 실행됐다.

편람 입력은 기존 `samples/2025 행정업무운영 편람(최종).hwpx`, 독립 기준은
`pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`다. 각각 SHA-256
`c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6`,
`299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e`로 검토 tree의 기존 blob을 사용했다.

## 정확한 code head GitHub CI

모든 검증의 head는 위 `b02f6a4c05`이며 실제 attempt1이다. 기록-only 재사용이 아닌 Full 실행이다.

| workflow | run | 결과 |
| --- | --- | --- |
| CI | [41813428104](https://github.com/edwardkim/rhwp/actions/runs/41813428104) | success |
| CodeQL | [41813427812](https://github.com/edwardkim/rhwp/actions/runs/41813427812) | success |
| Adapter inter-diff | [41813427665](https://github.com/edwardkim/rhwp/actions/runs/41813427665) | success |
| Proptest | [41813427952](https://github.com/edwardkim/rhwp/actions/runs/41813427952) | success |
| CI Impact Policy | [41816286538](https://github.com/edwardkim/rhwp/actions/runs/41816286538) | success |

실제 root nextest A/B/C/D/E는 3,801/2,122/2,087/2,122/2,110 PASS,
합계 **12,242 PASS·0 FAIL·45 skip**이다. 이전 #8583의 12,239개 또는 로컬 workspace 12,537개와
같은 집합으로 표시하지 않는다. 최신 base의 수식 접두어 검사는 실제 `equation_glued_prefix` PASS 로그도 확인했다.
이 PR에는 Render Diff run이 없어 성공으로 추가하지 않았다. 제품 렌더러 경로 변경이 없는 tool/CI diff이며
시각 근거는 아래에 적은 Native 전용 옵션 실측이다. WASM Build job은 영향 분류상 skipped이며
Native/WASM check가 들어 있는 Lint와 혼동하지 않는다.

## Native 편람 옵션과 직접 판독

[전383쪽 TSV](../assets/issue8587/handbook-native.tsv): Print2024·96 DPI·canonical PDFium RGB exact-DPI v2,
문턱232·2px·색상 경계 보정 전 raw 조건이다. 정확한 #8389 기록 정밀도 0.01%p 하한을 유지했다.
383쪽·누락/중복0·최저99.96463%·130쪽·기준 하락0쪽, 201.964초·exit0이며 145쪽 raw100%다.
원본·PDF·baseline·허용치는 바꾸지 않았다.

실제 명령에 지정 환경의 경로·hash가 전달된 것을 provenance로 확인했다. Native 실행 파일은
#8583 검증 바이너리 SHA `8ae681680593809015ebb5c0a19ddaa5269ba11aa56d467882d6be81ab3d8728`를
재사용했고 Python gate source producer는 `4c892a9886`이다. 이후 기록 commit에서 해당 코드가 변하지 않았다.
새 Native build 또는 새 fresh WASM 전쪽 실행이라고 표시하지 않는다. Native 전용 gate의 인자 전달만
바뀌었고 이전 #8583의 WASM 결과와 이번 Native 결과를 합치지 않는다.

145쪽의 실제 [Native raw](../assets/issue8587/native-p145-raw.png)·[PDF raw](../assets/issue8587/pdf-p145-raw.png),
[review](../assets/issue8587/native-p145-review.png)·[overlay](../assets/issue8587/native-p145-overlay.png)를 열어
기관명·도형·표·앞뒤 본문 소속과 줄바꿈을 확인했다. 원래 작은 raster 차이와 130쪽 미세 차이는 남는다.
표시100%·raw100%·RGB 완전 동일은 구분한다.

## 병합 후 실제 API 게시

[duration run41816717408](https://github.com/edwardkim/rhwp/actions/runs/41816717408)은 성공했다.
GITHUB_TOKEN `contents: write`로 실제 실행했고 수집·생성·API 게시가 모두 통과했다.

- 최종 승인/측정 run `41813428104`, head `b02f6a4c05`, attempt1, `reviewOnlyTail:false`.
- tested merge `a30ccb63fdc1c4da2ca941c0ac82151eaf608191`, B/C/D/E 실제 attempt 모두1.
- artifact IDs `13315350049 / 13314388582 / 13314516799 / 13314408595`.
- 46 targets, `published:true / reason:published / previousSha:null`.
- 게시 commit `b9d279745964aa2e4eff89c68eaaf43f30649519`, blob `594b94ca9f4c7a59d7d68a3d558fd89427eb8b6f`.

사용자가 앞서 metrics branch를 삭제했으므로 부재 상태의 생성 분기도 실제 검증됐다.
이 성공은 이전 Git push 500의 내부 원인을 특정하는 증거가 아니며 과거 실패 run도 그대로 보존한다.
사용자가 요청한 metrics branch 제거는 게시 증거를 확인한 뒤 최종 정리에서 다시 수행한다.
자동 생성 기능 자체를 폐지한 것은 아니므로 이후 병합 시 재생성될 수 있다.

## Merge 후 contributor PR comment 계획

옵션2 기록 PR의 실제 fast-pass·required check·mergeability를 확인하고 병합한 뒤 devel을 동기화한다.
원 PR에는 원 merge SHA로 고정한 편람145쪽 Native/PDF raw·review·overlay를 처음부터 표시하고,
[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment), 실제 CI·duration 결과와
한계를 한국어 존댓말로 적는다. 게시 전 이미지 응답·해시를 대조하고 body-file/API 본문·렌더링을 확인한다.
#8587의 수용 조건과 실제 API 게시를 다시 확인한 뒤 이슈 상태·완료 comment를 기록한다.
#8533이나 #8578은 재종료하지 않는다.

그 뒤 소유한 원 PR/기록 PR의 local·remote head와 이전 #8583의 남은 소유 head, 저장소 루트 output 전체·
미사용 target/pr-review/debug를 정리한다. 기본 작업공간·원본·PDF·글꼴·다른 worktree·다른 target profile과
사용자 Vite 서버는 보존한다. 정리 결과는 output 밖의 metadata 로그에 남긴다.
