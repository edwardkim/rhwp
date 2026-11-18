# PR #9155 — Ubuntu runner 기준과 Fuzz 컴파일 메모리 검토

## 최종 판정

**승인.** 검증 code head `d9868f4ef1e506918bb3eceece5453333b047e64`의 필수 CI·보안·정책과 승인된 Fuzz 24.04/26.04 각 6개가 모두 성공했다.
사용자가 승인한 self-review·일반 merge로 devel에 `1ea5d6df80eebcdc16cf451b4e76d4a48d5edc2a`를 반영했다. admin 우회·검증 예외는 적용하지 않았다.
원 workflow 변경 PR에는 trailing 기록 commit을 넣지 않았고 이 기록은 옵션 2의 별도 PR이다.
main 적용 전이므로 [#9154](https://github.com/edwardkim/rhwp/issues/9154)와 부모 [#536](https://github.com/edwardkim/rhwp/issues/536)은 OPEN을 유지한다.

## 실제 범위와 검토

- 제품 실행 job·archive의 Ubuntu 24.04 고정과 OS cache namespace, 실제 ELF version needs의 GLIBC 2.39 상한을 검토했다.
  exported symbol·GLIBCXX를 잘못 세지 않고 알 수 없는 결과와 readelf 진단을 거절한다. 실제 loader 검증은 별도로 수행했다.
- Fuzz는 build/run을 분리해 같은 `--codegen-units 16`을 쓰며 ASan·coverage·6개 타깃·60분/60초/30초/RSS 2 GiB를 유지한다.
  로컬 같은 SDK/source/lockfile에서 compiler peak 16.64→13.18 GiB, target build 24분 14초→8분 46초였다.
  host/proc-macro cache와 CPU 차이가 있어 원격 시간 보장·엄밀한 cold-system 성능비로 일반화하지 않는다.
- 실제 compiler 인자의 ASan·coverage·trace-compares·fuzzing cfg·debug assertions를 확인했고 HWP/HWP3의 양 구성 실제 퍼징과
  같은 느린 입력 25.57초/26.26초의 재실행이 모두 통과했다. 공개 corpus 파생 입력과 SHA는 code head에 보존했다.
- 두 OS HWP의 이전 27분 2초/25분 40초 exit 143은 runner shutdown이며 60분 timeout이 아니다.
  HWP3·Chart의 60분 build timeout과 구분했다. Atomic API 사용 중단은 경고이며 제품 API를 바꾸지 않았다.
  원격 OOM의 직접 증거는 없으므로 종료 원인을 확정하지 않았다.
- 수동 OS 입력의 allow-list와 실제 선택 일치 확인은 checkout 전에 실행한다. 새 workflow·PR 자동 Fuzz·권한·보호 설정은 추가하지 않았다.
- 조판 원칙 2.7: 제품 Rust/JS/Cargo·IR·렌더·저장 source 변경이 없어 해당 없음. CI-only 검증 범위를 적용했다.
  검증 입력 2.8: 공개 tracked Fuzz seeds와 실제 생성 입력을 사용했고 비공개 corpus·재배포 불명 글꼴을 넣지 않았다.

## 실행 근거

| 검증 | 실제 결과 |
| --- | --- |
| 최종 로컬 workflow 계약·정책·lint | 263·339 PASS, actionlint 1.7.12·문서 링크·diff PASS |
| [새 head CI](https://github.com/edwardkim/rhwp/actions/runs/44117430108) | Archive A–E 13,523 PASS·45 skip, 선택 Native 245 PASS, Studio 2,805 PASS·2 skip |
| [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44117430197)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44117430218) | 성공, PR merge ref OPEN code alert 0건 확인 |
| [Adapter](https://github.com/edwardkim/rhwp/actions/runs/44117430287)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/44117430117) | 성공 |
| [최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/44123809110) | 실제 HEAD_SHA/CURRENT_HEAD_SHA가 검증 head이며 success 게시 |
| [Fuzz 24.04](https://github.com/edwardkim/rhwp/actions/runs/44117952664)·[Fuzz 26.04](https://github.com/edwardkim/rhwp/actions/runs/44118744018) | 실제 build/run 12개 성공, 최신 peak 최대 8.092 GiB |
| [Native 3플랫폼](https://github.com/edwardkim/rhwp/actions/runs/44072214045) | 실제 producer `de94e7cc23`, Linux artifact digest·ELF 확인과 같은 package의 Docker 24.04/26.04 소비 성공 |
| [Renderer 24.04](https://github.com/edwardkim/rhwp/actions/runs/44072271463)·[26.04](https://github.com/edwardkim/rhwp/actions/runs/44072292744) | 성공, browser 378 PNG·105 SVG 동일; Native 34 PNG 실제 차이는 그대로 기록 |

선택 Native 245건은 공식 Native 3종 전체 검사를 뜻하지 않는다. Native/Renderer 기존 증거의 재사용은 사용자 승인 범위이며
task branch의 해당 실행 정의·제품·도구 동일성을 확인했다. 기존 artifact를 최신 병합 결과의 새 바이너리라고 주장하지 않는다.
최신 devel `22da83586a`의 기능 보존 7건을 별도로 실행했고 실제 GitHub merge-ref tree와 로컬 병합 tree가 같았다.

[원장·개별 job 수치·직접 판독](../assets/pr9155/README.md), [receipt](../assets/pr9155/final-ci-and-fuzz-receipt.json),
[원래 로컬 보고](../../report/task_m100_9154_report.md)를 연결한다.

## 남은 범위와 후속 처리

main 승격·정기 Fuzz 적용/관찰은 별도 승인 대상이다. 기존 cache read-only 경고는 해결됐다고 주장하지 않으며 cache 삭제·예산 변경을 하지 않았다.
병합 뒤 duration refresh는 성공했고 44개 target의 검증된 시간 데이터를 게시했다. 실제 결과는 [영수증](../assets/pr9155/postmerge-duration-result.json)에 있다. post-merge Full CI를 새로 시작하지 않았다.
기록 PR의 실제 fast-pass·required aggregate 성공 후 devel 동기화, issue 상태/댓글, 이 작업 전용 branch/worktree와 output/debug 종료 정리를 완료한다.

## Merge 후 comment 계획

운영 기록이 devel에 반영된 뒤 #9154에 code merge SHA·두 OS Fuzz·실제 Native 소비 결과와 main 미적용을 존댓말로 게시한다.
같은 merge·근거의 댓글이 이미 있으면 중복 게시하지 않는다. #9155에도 self-review·merge와 기록 경로를 남긴다.
부모 #536과 #9154는 닫지 않는다. 별도 기록 PR에 또 후속 기록 PR·오늘할일 반복 갱신을 만들지 않는다.
