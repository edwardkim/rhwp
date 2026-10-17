# PR #7474 검토 기록 — PDF 추가 후 release/cdylib 재측정

## 최종 판정

**머지 보류 — 추가 CI 비용과 #7488의 명령 단위 LTO 정책에 대한 메인테이너 검토를 기다린다.**
코드 후보의 CI와 9회 비교 검증은 완료했고 기능상 차단 결함은 발견하지 않았다.
Draft를 유지하며 GitHub approve·merge·이슈 종료는 하지 않았다.
해제 조건은 위 정책·비용 수용과 최신 head CI·mergeability 확인이다.

## 접수와 범위

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR·작성자·base | #7474 / postmelee / devel |
| 원 #7474 head | `9d05998e09e4698391fe44f543abdd66ddc86c99` |
| 고정 devel | `6ed1682c3267ea786734eb2e75252916a861d1fd` |
| 통합 #7488 head | `e8e57b36d4703556128dae31e37eaa9acdceee0d` |
| 검증 코드 후보 | `0eb4cc2ffde257a15080d0add104247c941fcbf9` |
| 규모·상태 | 코드 후보 기준 19 files, +2060/-6; Draft, MERGEABLE/CLEAN |
| 관련 이슈 | #7473 참조, 종료하지 않음 |

사용자 요청에 따라 #7488을 선반영하고 PDF import 기본 feature가 포함된 devel에서 다시 측정했다.
기본 경로는 `collaborator_self_merge.md`, 보조는 `intake_and_review.md`, `local_validation.md`,
`review_only_fast_pass.md`, `rework_and_exceptions.md`다. `review_template.md`, `github_operations.md`,
`visual_fixture_evidence.md`의 관련 절도 적용했다. self PR이므로 별도 reviewer를 지정하지 않는다.

## 코드와 계약 검토

- PR은 release/cdylib, 수동 실행은 dev/standard·release/standard·release/cdylib를 선택한다.
  dev+cdylib는 거부하며 inherited opt-in 환경변수를 명시적으로 덮어써 조건을 고정한다.
- cache key와 restore prefix에 profile·crate type·LTO를 넣고 Cargo manifest·lock·wrapper도 해시한다.
  서로 다른 구성의 cache를 섞지 않으며 PR restore-only와 trusted main/devel 저장 제한은 유지한다.
- #7488 wrapper를 원본 그대로 통합했다. cdylib의 `profile.release.lto=false`는 명령 단위 override다.
  Cargo.toml 전역 설정·Rust 제품 코드 변경은 이 PR의 base 대비 diff에 없다.
- release에서는 실제 wasm-opt 실행을 관측해야 성공 manifest를 만든다. E2E는 탐색 전에 CDP를 연결해
  실제 앱 WASM 응답을 확인하고 누락·실패·해시 불일치를 거부한다. 종료 시 CDP를 정리한다.
- metadata에 CPU/image와 빌드 구성을 기록해 서로 다른 VM의 결과를 동일 환경으로 오인하지 않게 했다.
- 주요 한계: cold 개선은 CPU별 1~2회 표본, cache hit 성능 미측정, 배포와 LTO 플래그 동일성 미보장.

## 검증과 남은 차이

[재측정 보고서](../../report/task_m100_7473_report.md)에 9회 run URL·시간·CPU·산출물 hash가 있다.
전 실행 Canvas 3/3·직접 PDF 3/3·readiness 8/8, PNG 43개 동일, 실제 응답 162건 일치다.
report-only PDF 경고 4개와 KTX 머리글 겹침은 기존 그대로이며 threshold를 변경하지 않았다.
대표 KTX·table-border-style PNG를 직접 판독했다. 조판 알고리즘·회귀 기준 변경은 비해당이며
한컴 정합·전쪽 Native/fresh WASM silhouette 개선을 주장하지 않는다. 별도 TSV는 생성하지 않았다.

- `python3 -m unittest` focused wrapper/render/workflow/Docker 묶음: 28 PASS.
- `python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'`: 192 PASS.
- `node --test scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs rhwp-studio/tests/wasm-artifact-check.test.ts`: 실제 102 PASS.
- actionlint, shell/Python/Node 구문, 변경 Markdown 링크, `git diff --check`: PASS.
- dev/standard/cdylib 모드와 반대 inherited 환경변수에 대한 mock subprocess 경계 확인: PASS.
- 로컬 Rust 미설치로 로컬 전체 Rust 회귀는 미실행. 제품 Rust를 추가 변경하지 않았으며 실제 빌드와 브라우저는 CI에서 검증했다.

정확한 코드 후보의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/39072973013),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39072973012),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39072973025),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/39072973002),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/39072973042), CI Impact가 완료됐다.
34 success·5 skipped, 실패·미완료 0건 확인 후 이 문서 후행 기록을 작성했다.

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

## 문서 후행 커밋과 처리 범위

녹색 코드 후보 위에 보고서·review·통합 기록·오늘할일·대표 PNG만 single-parent 문서 커밋으로 추가한다.
push 전 최신 devel/head merge tree·공백·변경 링크를 확인하고 후행 head의 실제 CI·fast-pass를 다시 확인한다.
코드 후보 성공을 후행 head의 자동 통과로 간주하지 않는다.
[통합 순서·되돌리기](pr_7474_review_impl.md)를 따른다.

## Merge 후 contributor PR comment 계획

이번 작업은 self PR 재측정·리뷰이며 merge나 별도 댓글 게시는 하지 않는다.
추후 merge가 승인되면 실제 merge SHA·최신 CI·본 보고서·merge SHA 고정 대표 PNG와 남은 차이를
한국어로 준비하고 해당 시점의 게시 지시와 절차에 따라 처리한다.
