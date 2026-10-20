# PR #7474 검토 기록 — standard release와 배포 WASM 동일성

## 최종 판정

**승인 — 검증한 코드 후보가 standard release 기본값과 검사한 WASM의 배포 패키지 재사용 요구를 충족한다.**
이 판정은 작성자 self-review이며 GitHub approve·Ready 전환·merge를 뜻하지 않는다.
#7474는 Draft를 유지한다. 남은 merge 조건은 문서 후행 head의 최신 CI, 배포 전 검사 비용에 대한
메인테이너 검토와 실제 병합 승인이다. 이슈는 종료하지 않았다.

## 기준과 절차

- PR #7474, 작성자 postmelee, base devel. 기준 SHA `5e2b9bfbf08573a31fa1e11a2445ff63fc0755c0`.
- opt-in #7488 head `3bfe08e24a55f16f7408405ead9550ffab925dad`를 이력 보존 통합했다.
- 검증 code head `1ae9b510a5af0164de45191670e5e895e97270b3`.
- 기본 경로 collaborator_self_merge, 보조 intake_and_review·local_validation·multi_pr_update_branch·
  review_only_fast_pass. github_operations와 publish_guide를 함께 적용했다. self PR reviewer는 지정하지 않는다.
- O3 workflow 실행·artifact 계약 변경이다. 기존 required context·권한·production trigger와 publish=false를 보존했다.

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

- focused Python 46개, workflow 계약 192개, 정책/provenance Node 102개, actionlint·Python/Node/쉘 관련 구문·공백 검사.
  테스트 묶음은 중복을 포함하므로 합산하지 않는다. 파일 교체·dirty/stale/dev/cdylib·소비 누락/불일치 반례를 검사했다.
- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/39380204001),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39380203913),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39380203897)와 관련 PR checks: 33 success·5 skipped.
- [Pages](https://github.com/edwardkim/rhwp/actions/runs/39380248603): Build success, Deploy skipped.
- [Package](https://github.com/edwardkim/rhwp/actions/runs/39380255159): source guard·WASM·VSIX·aggregate success,
  외부 게시 4채널 skipped, verdict completed/verify-only.
- 세 경로의 JS/WASM bytes 동일, 실제 WASM 소비 22건 일치, 대표 Canvas/direct PDF 각각 3/3,
  PR readiness 8/8. artifact ZIP digest를 확인한 뒤 manifest와 package/VSIX bytes를 별도로 재검산했다.
- #7488 head도 CI 31 success·5 skipped이며 opt-in 제품 코드는 그대로다. Ready·메인테이너 검토 대기다.

[보고서](../../report/task_m100_7473_report.md)에 시간·CPU·실제 checkout SHA·hash·원자료 run을 기록했다.
서로 다른 VM의 n=1 시간을 개선율로 사용하지 않는다. 배포 검증 action은 225/239초의 추가 작업이며,
release CI 비용은 남는다. report-only PDF 경고 4개는 유지했다. cache 저장은 예산 초과로 거부됐고
warm-cache 비용 절감을 보장하지 않는다. runtime 성능·전쪽 정확도·실제 registry 게시를 검증했다고 주장하지 않는다.

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

정확한 code head의 CI와 두 verify-only 실행이 모두 성공한 뒤 review·보고서·오늘할일만 후행 커밋으로 추가한다.
최신 base와의 merge tree, 공백, 변경 링크, 오늘할일의 다른 기록 보존을 확인한 뒤 push한다.
후행 head의 required check는 별도로 확인하며 기존 code 결과를 자동 통과로 간주하지 않는다.
[적용 기록](pr_7474_review_impl.md)을 따른다. 실제 배포·approve·merge·이슈 종료는 수행하지 않았다.
