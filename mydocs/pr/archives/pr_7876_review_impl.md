# PR #7876 체리픽 통합·메인터너 보정 실행 기록

## 승인·처리 경로

사용자가 #7876의 프로젝트 적합성을 먼저 검증하고 체리픽 검토·필요한 메인터너 보정을 요청했다.
첫 기여자 절차와 원 크레딧을 적용했고, PR push·생성·CI 모니터링·trailing 문서·정상 merge·후속 처리까지 승인을 받았다.
기본 작업공간의 최신 devel 기반 통합 경로를 사용하며 원 contributor fork/source는 변경하지 않았다.

- 통합 PR: [#7981](https://github.com/edwardkim/rhwp/pull/7981)
- branch: `review/9ood2s-7876-20261017`
- 기준: `515bd72dcde0eecdeece333666b675354b7fa91d`
- 원 source: `b273e3fc44dd713f33d22846faa6fb1d98f01d3b`
- 판정·독립 근거·검증: [개별 review](pr_7876_review.md)

## 적용 순서와 역할

| SHA | 역할·내용 |
| --- | --- |
| `38d6de1b41808d602970e766e4d7c0ea5f9d65b5` | 프로젝트 적합성과 pinned wasm-bindgen·최신 rhwp 수정 전 재현을 먼저 기록 |
| `1869234ed8c2e98138677bee2c76baeb64ed5c23` | 원 b273e3fc 체리픽, author/co-author와 -x 보존. 현재 pdf-import를 유지해 충돌 해결 |
| `366654f29959c62088bee7206d77b8889e3ebcd4` | 일반 main/명시 start·Rust API opt-in·additive feature 설명 보정. 최종 로컬 기능 producer |
| `69f1b8e7ce2427fd642bec1b7a07a074f752264e` | PDF 제외 CLI/Studio 안내·검증 결과·완료 stage archive. 기능 파일 diff 0인 Full CI candidate |

## 완료한 단계

1. 원 PR reviewer를 지정하고 첫 rhwp 기여임을 확인했다. #5446의 배포/API 경계와 현재 rlib 경로를 대조했다.
2. 같은 버전의 최소 start 4조건과 실제 rhwp 소비 앱의 Node/Chrome에서 원 문제·정상 경계를 재현했다.
3. Cargo 충돌을 보정해 원 기능을 통합하고 적용 전후 main/export/파싱 계약을 직접 대조했다.
4. 기본 JS/type/API 표면·선택 소스·브라우저 전12쪽과 feature-only 실제 바이너리 대조로 기본 동작을 확인했다.
5. PDF on/off 브라우저와 비기본 CLI를 검증해 현재 프로젝트 기능과 사용법을 보존했다.
6. 필수 lint·workspace·전체 회귀·Skia·fresh WASM 검증을 완료했다. 실제 명령·수치·한계는 [완료 기록](../../working/archives/task_m100_7875_stage2.md)에 있다.
7. upstream/devel이 동일함을 다시 확인하고 LFS 대상 없음·dry-run 뒤 후보를 push해 #7981을 생성했다. PR 본문의 한글/LF/바이트 왕복을 확인했다.
8. 정확한 code candidate의 6 workflow·36 check 성공을 확인한 뒤에만 이 리뷰·오늘할일 trailing 문서를 작성했다.

## 남은 순차 게이트

1. 최신 devel과 source의 오늘할일 변경 구간을 비교하고 기존 기록을 유지한다.
2. 로컬 문서 commit 후 고정 base/head의 merge-tree 종료 0·공백·실제 merge tree 링크·오늘할일 보존을 확인한다.
3. 원격 base/head가 검사 기준과 같은지 확인하고 문서 commit만 push한다. 새 head의 실제 preflight fast-pass·heavy worker skip·최종 required aggregate와 mergeability를 확인한다.
4. 원 PR에 첫 기여 감사·환영과 정확한 통합 검토 의견을 남긴다. 원 head가 바뀌지 않았는지 확인하며 잘못된 원 PR 직접 승인은 하지 않는다.
5. exact trailing head를 --match-head-commit으로 고정해 정상 merge한다. 원 author 이력을 보존할 수 있는 merge commit 방식을 사용하고 --admin으로 우회하지 않는다.
6. merge SHA·archive/오늘할일·원 기여 commit의 devel 포함과 자동 duration 실행 결과를 확인한다. 검증 workflow를 병합 후 다시 시작하지 않는다.
7. devel fast-forward 후 #7875 자동 close를 확인하고 필요하면 수동 close한다. 검증 결과 comment를 남기고 원 PR에 merge SHA 고정 결과와 첫 기여 환영을 게시한 뒤 통합 완료로 close한다.
8. 이번 작업에서 만든 upstream 임시 head의 SHA·merged 상태·다른 OPEN PR 부재를 확인해 lease로 삭제한다. clean한 local review branch/소유 PR fetch ref도 정리한다. 별도 worktree는 만들지 않았다.
9. 최종 증거는 archive/stage·기존 입력·GitHub 실행에 연결한다. 활성 작업·열린 경로가 없음을 확인한 뒤 root output 전체를 제거한다. 기본 작업공간·원 fork·공유 target/pr-review·tests/golden_svg는 보존한다.

source/test/fixture/workflow/기준값은 trailing 문서에 섞지 않는다. 실제 remote/CI 결과를 완료 전 기록하지 않으며 별도 후속 문서 PR은 만들지 않는다.
