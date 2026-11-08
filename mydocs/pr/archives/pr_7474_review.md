# PR #7474 검토 기록 — standard release와 배포 WASM 동일성

## 최종 판정 — 2026-10-23

**기술적 수용 권고.** 최신 devel의 standard release 렌더 검사와 패키지 동일성 검증을 완료했고,
실측 추가 비용을 검토했다. 이 기록은 작성자 self-review이며 GitHub approve나 merge를 뜻하지 않는다.
문서 후행 head의 CI 성공을 확인한 뒤 Ready로 전환하고 메인테이너 최종 판단을 기다린다.
code head와 기록 head의 검증을 구분하며 실제 배포·병합·이슈 종료는 이번 승인 범위에 없다.

## 기준과 절차

- base route: collaborator_self_merge. modifiers: intake_and_review, local_validation,
  multi_pr_update_branch, review_only_fast_pass. 각 자식 문서와 github_operations를 읽고 적용했다.
- base devel: `605ccfd92f2f12f18e14182548d729f9929fda00`.
- 검증 code head: `df39fb5d7a566ba6a49c2f6a04965616d2c2af73`.
- 포함한 #7488 head: `3bfe08e24a55f16f7408405ead9550ffab925dad`. opt-in을 자동 선택하지 않는다.
- O3 workflow·artifact 계약 변경이다. required context·권한·production trigger·publish=false를 보존했다.
- 작업지시자는 병합 전 문서·코멘트·이슈 정리와 최종 CI 확인을 승인했고 실제 병합은 제외했다.

## 코드 검토

1. #7488 옵션을 자동 활성화하던 helper auto/PR 기본값을 standard로 변경했다. 수동 cdylib 비교와
   개발 dev 경로를 유지하며 cache key/prefix의 profile·crate type·LTO 분리도 유지한다.
2. Pages/npm은 기존 WASM 빌드 1회를 계측 wrapper로 실행한다. action은 그 pkg를 검사하며 WASM을
   다시 빌드하지 않는다. 다른 PR run의 artifact를 privileged 배포에 가져오는 경로는 추가하지 않았다.
3. release guard는 clean source SHA·standard 설정·wasm-opt 실행·JS/WASM bytes를 확인하고,
   실제 CDP 응답과 manifest의 source/profile/hash를 대조한다. matched flag만 참인 잘못된 응답도 거부한다.
4. Canvas/direct PDF 검사 실패와 준비 실패는 업로드를 막는다. Pages는 dist까지 재대조한다.
   npm/VSIX는 기존 같은-run wasm-pkg를 다운로드한다. 최종 VSIX 내부 JS/WASM도 직접 해시를 확인했다.
5. 출력 알고리즘·fixture·baseline/threshold를 수정하지 않았다. 독립 한컴 조판 정확도 개선 주장이 없으므로
   별도 전쪽 TSV·Visual Sweep은 비해당이다. 기존 대표 문서 gate와 report-only 경고는 구분했다.

## 완료한 검증

- 로컬 focused Python 46개, workflow 계약 192개, Node 계약 102개, actionlint·공백 검사 통과.
  중복되는 검사 묶음은 합산하지 않는다. 입력 12개와 code commit blob의 bytes도 일치했다.
- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/39885844384),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39885844317),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39885844291), Adapter·Proptest 성공.
  수동 실행 포함 code head check 35 success·12 skipped, 별도 CI Impact Policy 성공.
- [Pages](https://github.com/edwardkim/rhwp/actions/runs/39885887626): Build success, Deploy skipped.
- [Package](https://github.com/edwardkim/rhwp/actions/runs/39885906844): WASM·VSIX·집계 success,
  외부 게시 4채널 skipped, accepted=true / completed / verify-only.
- 세 경로 Canvas/direct PDF 각각 3/3, PR readiness 8/8. JS/WASM 해시와 실제 브라우저 응답 22건 일치.
  artifact ZIP digest와 wasm-pkg·최종 VSIX bytes를 재검산했다. Pages dist는 job 검사로 확인했다.
- npm core/editor tarball을 게시 없이 생성하고 포함 파일을 확인했다. core 내부 WASM은 검사한 것과 같았다.
- #7488은 10월 20일 head의 check 30 success·5 skipped와 별도 CI Impact Policy success를 재확인했다.
  최신 devel과의 병합 충돌이 없고 wrapper·계약 테스트는 이번 #7474에서 검사한 파일과 같다.
  최신 제품 코드에서 opt-in 성능을 다시 측정했다고 주장하지 않는다.

추가 검증 action은 222/208초였고 이번 Render Diff는 전체 CI보다 425초 먼저 끝났다.
추가 runner 비용은 남지만 이를 이유로 더 보류할 근거는 약하다. Pages cache 저장 성공과 package의
같은 키 예약 거부를 확인했다. warm cache 절감을 보장하지 않는다. report-only PDF 경고 4개와
대표 문서·쪽 범위를 유지하며 runtime 성능·전체 정확도·실제 게시 성공을 주장하지 않는다.
[최신 보고서](../../report/task_m100_7473_report.md)에 세부 근거와 역사적 비교를 구분했다.

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

## 후행 기록과 남은 처리

정확한 code head의 CI와 두 verify-only 실행이 모두 성공한 뒤 문서·증적·오늘할일만 후행 커밋으로 추가한다.
최신 base의 기록 보존, merge tree·공백·링크를 확인하고 push한 뒤 후행 head의 실제 CI 결과를 확인한다.
문서 전용이라는 이유로 fast-pass나 성공을 가정하지 않는다. 완료 결과와 최종 head는 PR 코멘트에 기록한다.
Ready 전환 뒤에도 실제 merge는 별도 지시가 필요하며, #7488 → #7474 순서를 권고한다.
[적용 기록](pr_7474_review_impl.md)을 따른다.
