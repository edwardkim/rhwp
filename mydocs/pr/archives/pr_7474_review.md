# PR #7474 검토 기록 — PR Render Diff의 release WASM 검증

## 최종 판정

**Draft 검토 대기.** 작성자 self-review의 코드 검증과 동일 SHA CI 비용 비교를 완료했다.
코드 후보 `ddf5ce2`의 Full CI·Render Diff·CodeQL·CI Impact Policy가 모두 성공했다.
Draft 상태로 유지한다. ready 전환·GitHub approve·merge는 이번 실행 승인에 포함되지 않는다.

## 접수와 범위

- PR: https://github.com/edwardkim/rhwp/pull/7474, 작성자 `postmelee`, base `devel`.
- Issue: #7473 참조. 비용/검증 확인 전 이슈 종료를 선언하지 않는다.
- 최초 후보 `a4220e1aad7bc2a39c094130e56a5483cae7511d`의 Lint에서 trusted impact 정책의
  mirror 경로 등록 누락을 발견했다. `scripts/ci-impact-policy.cjs`에 두 경로를 추가해 수정했다.
- 비교 후보: `ddf5ce2d2c13d4c87aa4a3b165d39b7506d3a25f`.
- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서,
  `review_template.md`, `github_operations.md`.
- reviewer를 별도 지정하지 않는 collaborator 본인 PR 경로다.

## 코드와 계약 검토

`wasm-pack-locked.sh`의 release 결과를 측정 script가 해시 고정하고, `loadApp`은 탐색 전에 CDP를
연결해 실제 WASM 응답을 관측한다. 앱 준비 뒤 bytes를 대조하며 응답 누락·오류·불일치를 실패시킨다.
로컬 기본 E2E는 manifest 미설정 시 기존 동작을 유지한다. release에서 wasm-opt 실행이 관측되지
않으면 성공 manifest를 만들지 않는다. 도구 `--version` 조회만으로 최적화 실행을 인정하지 않는다.

workflow의 PR 경로는 release로 고정하며 dev 선택은 workflow_dispatch에만 적용된다.
새 script와 기존 wrapper 경로를 workflow trigger·classifier·trusted policy의 세 목록에 동기화했다.
PR cache restore-only와 trusted default branch seed 조건, 권한·기존 job 이름·시각 기준값은 유지한다.

조판 원칙은 **비해당**이다. 측정/배치·좌표·pagination·저장 LineSeg·렌더링 알고리즘을 바꾸지 않고,
특정 문서의 한컴 정합이나 시각 결함 해결을 주장하지 않는다. 빌드 프로필 전환의 출력 동등성은
별도 실제 dev/release 비교로 확인하며 CI 통과를 한컴 정합 증거로 대신하지 않는다.

## 로컬 검증과 남은 차이

실행 명령·도구·WASM hash·이미지 직접 판독은
[작업 기록](../../working/task_m100_7473_stage1.md)을 재사용한다.
Canvas/PDF PNG 27개가 같고, readiness PNG 16개도 같았다. macOS readiness는 양쪽 모두
`table-border-style` 하나 실패(7/8)이며 허용치를 완화하지 않았다.
최종 후보의 Ubuntu dev/release 각 3회가 성공했고 readiness는 각각 8/8 통과했다.
43개 PNG가 실행 간 동일하며, 108건의 실제 WASM 응답이 해당 빌드 manifest와 일치했다.
실행 URL·조건·비용·해석 한계는 [최종 보고서](../../report/task_m100_7473_report.md)에 있다.

초기 정책 누락 수정 뒤 실제 CI 정책 묶음 160개와 Python workflow 계약 183개를 통과했다.
Rust source/Cargo 변경이 없어 새 전체 로컬 Rust 회귀는 비해당이다. GitHub required checks는
최신 PR head로 확인한다.

## 검증 입력

다음 기존 파일의 작업 트리 bytes와 비교 후보 commit의 git blob bytes가 모두 일치한다.
별도 한컴 기준 PDF는 사용하지 않았으며, PDF export 결과는 생성 산출물이다.

| 저장소 입력 | SHA-256 |
| --- | --- |
| `samples/basic/KTX.hwp` | `6c1a027d67b33c03f469b56548b4c7d6bca36b1c1190c7cc5eac88e35c403cf1` |
| `samples/biz_plan.hwp` | `8b786d6824622afae2220b203beeef6e5592157e1896fea055ebc602817113c1` |
| `samples/hwp_table_test.hwp` | `dc3e57d7577447ae4b47f59bdb0f499a5ac29066b62af8a650bcdd8d70eee32a` |
| `samples/kps-ai.hwp` | `9b0fceb3d96956f27c893e15a72a1ad94f7ee005bd581381a1aadfcb1f57a7b9` |
| `samples/lseg-01-basic.hwp` | `86e607233a2218928fabe5969155116a46d4ab4400db1402fe45b0082149496c` |
| `samples/lseg-05-tab.hwp` | `0066108df04204324c0802414087563e7673e30f3d142dd014af14e318153d2c` |
| `samples/pic-crop-01.hwp` | `3b5afbe4cdb18faa49452a9cbed770d5caa27aed2f83b708e9ab8516c65ebf4c` |
| `samples/pua-test.hwp` | `592e00e3e8bc72afef829fd71a13cbc340bff3770425b9d1ee025afc1e1649fa` |
| `samples/re-font-batang-hancom.hwp` | `99ec0cd2f55ba45493b97c550333db6d0275831ae6150a041bdbe5e2bd174049` |
| `samples/render-p35-font-native-bitmap.hwpx` | `1c902d41d47e532f42d2b24a44058efd370ee6496927c32e76e955029a0b9eec` |
| `samples/table-004.hwp` | `451ab7c40d47c4200e98b57b7cb3806e64968c32fa9441fc171e7babf64be915` |
| `samples/tac-case-001.hwp` | `c003385b7b3495dd93a2688ea74b8a07848543a6c0128cb693635d8ba6fe43da` |

## 문서 후행 커밋

녹색 코드 후보 위에 보고서·원자료 요약·이 검토 기록만 single-parent 커밋으로 추가한다.
push 전 최신 base/head merge tree·공백·변경 문서 링크를 검사한다.
최신 문서 head의 required checks와 실제 trusted 재사용 판정은 push 뒤 별도로 확인한다.
코드 후보 성공을 문서 head의 자동 통과로 간주하지 않는다.
