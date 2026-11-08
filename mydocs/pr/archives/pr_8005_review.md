# PR #8005 리뷰 — 수정(renderer): 빈 host 다중 자리차지 표의 틈 채우기와 둘째 표 아래 host 글줄 (#7687, #7680)

## 최종 판정

**메인터너 보정 후 수용 가능 — 아래에 명시한 원 PR의 기능 범위의 로컬 검증 완료.**

통합 PR [#8148](https://github.com/edwardkim/rhwp/pull/8148)의 정확한 code head `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7` CI가 성공했습니다. 이 기록은 같은 PR의 문서 trailing에 포함합니다. 아직 생성 전인 trailing head의 CI와 실제 merge는 별도 완료 조건입니다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR | #8005 / planet6897 / devel |
| 원 head | `839af8c92427d834d7042ec540c5e0fbab01473c` |
| 원 기능 tip | `839af8c92427d834d7042ec540c5e0fbab01473c` |
| 현재 통합된 원 기능 | `7bf6a6613992e42cd8bfb41a76edd7894e46d585` |
| 실제 runtime | `3cb5a4c290663343123074be1e2fc0dda18d3db4` |
| 전체 회귀 검증 runtime | `3cb5a4c290663343123074be1e2fc0dda18d3db4` |
| 제출 code head | `228db2f278b52bc73fe2c5b5e0bb9a09b2ef4ff7` |
| 새 회귀 등록·lint head | `e25dbd0f5d47bc32686016c08054d2bf718be6a7` |
| 검증 기준 devel | `a6e9d18fc1061318055b4eb348daec201427ea03` |
| 최신 원격 조회 | 2026-10-22T01:51:22.667055+00:00 / OPEN / draft=False / reviewer jangster77 |

## 변경과 검토 범위

빈 호스트에 붙은 여러 float의 실제 제외 영역과 후속 본문 위치를 공통 결과로 검토한다. 이미 미달 원본의 기존 검사를 보관·이관한 범위와 활성 정상 대조군을 구분한다.

정상 empty-host fixture와 기존 HWPSPEC 원본의 시각 미달·보관 검사 범위를 구분합니다. #7687 전체 해결을 주장하지 않습니다.

## 현재 원본·Print·TSV 범위

아래는 이 원 PR의 주장에 직접 연결한 현재 전쪽 결과입니다. 공통 숫자를 원 PR의 다른 원본 해결로 대체하지 않습니다.

| 입력 | PDF/출력 쪽수 | Native 최저 raw | WASM 최저 raw |
| --- | ---: | ---: | ---: |
| added-issue7687-empty-host-hwpx | 6/6 | 100.00000% | 100.00000% |

입력·독립 Print·폰트 환경·TSV SHA/바이트·기준 대비 결과와 대표 PNG: [전쪽 TSV·입력/Print·producer 원장](../../../mydocs/pr/assets/planet6897_20261018/README.md) / [변경 영역의 직접 판독](../../../mydocs/pr/assets/planet6897_20261018/documents/current_visual_reading.md) / [원 PR별 추가 대표 판독](../../../mydocs/pr/assets/planet6897_source27_representatives_20261022/README.md).

## 조판 원칙과 검증의 구분

원본 저장 속성·줄/개체 소유·공통 측정/예약/배치의 실제 경로를 검토하고, 독립 Print·관계 검사와 완료 페이지의 ±2px 근거를 구분합니다. 90% 이상 수치가 수식·미세 간격의 직접 판독을 대신하지 않습니다. 생산·측정·배치의 실제 경로와 반례·메인터너 보정은 [통합 계획·생산/소비 경로·보정 기록](../../../mydocs/plans/pr_review_planet6897_20261018.md)을 따릅니다. 실제 입력·Print SHA와 제출 commit의 Git blob 일치, Native/fresh WASM 전쪽 쪽수·raw 90% 이상·내용 대응·0.1%p 이상 하락 없음은 [전쪽 TSV·입력/Print·producer 원장](../../../mydocs/pr/assets/planet6897_20261018/README.md)에 연결합니다. 저장 재조판의 원래 비적용·이관 범위는 위의 구분을 유지합니다.

## 실제 회귀 검사와 CI

실제 runtime `3cb5a4c290663343123074be1e2fc0dda18d3db4`에서 전체 nextest 11,353 PASS·45 skipped를 확인했습니다. 새 수식 검사 등록 `e25dbd0f5d47bc32686016c08054d2bf718be6a7`는 별도로 5 PASS였고 fmt·Native/WASM/workspace Clippy·workspace build·manifest 검사를 통과했습니다. 한 번에 11,358건을 로컬 실행한 것으로 합산하지 않습니다. Native Skia lib 4,041 PASS, placeholder 2 PASS, 직접 PDF 14 PASS, Studio 2,356 PASS·타입 검사·production build, 최적화 fresh WASM·실제 브라우저 3 PASS, Native 소비자 38 PASS·실제 raster/영역/cache 측정 12건, FFI/Qt offscreen 25 PASS와 리베이스 반영 기능의 실제 저장·재열기 대조를 완료했습니다. 전체 명령·개별 scope와 SHA는 [실제 검증 명령과 결과](../../../mydocs/pr/assets/planet6897_20261018/documents/validation_commands.md)에 기록했습니다.

이 원 PR에 대응하는 기존 정식 검사 **1 PASS**를 전체 nextest의 실제 PASS 행과 대조했습니다. 아래 수량은 원 PR 사이에서 겹칠 수 있으며 전체 수에 중복 합산하지 않습니다.

| 실제 현재 검사 경로 | 실행 수 | 상태 |
| --- | ---: | --- |
| [tests/cases/issue_7687_empty_host_multi_float_gap.rs](../../../tests/cases/issue_7687_empty_host_multi_float_gap.rs) | 1 PASS | 실제 실행 |

정확한 제출 code head의 실제 성공: [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39705756548) · [CI](https://github.com/edwardkim/rhwp/actions/runs/39705756552) · [CI Impact Policy Controller](https://github.com/edwardkim/rhwp/actions/runs/39705756369) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39705756537) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39705756582) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39705756438). 기여자 fork의 CI를 통합 head의 결과로 대신하지 않았습니다.

## Merge 전 조건과 후속 처리

이 검토는 원 기여 source를 보존하고 통합 PR #8148의 공통 보정을 판정합니다. [기여 SHA·현재 통합 SHA와 처리 순서](../../../mydocs/pr/archives/pr_8005_review_impl.md)를 연결합니다. 문서 trailing의 최신 required CI·병합 tree·링크·원격 SHA를 확인한 뒤 사용자가 승인한 merge와 후속 처리를 수행합니다. merge 전 기록에서 merge SHA나 issue close를 완료로 표시하지 않습니다.

merge 뒤에는 실제 merge SHA에 고정한 TSV·review/overlay PNG와 CI URL, 원 기여·추가 공통 보정·남은 범위를 구분한 한국어 존댓말 comment를 원 PR에 게시합니다. 원 PR의 open/closed 상태를 다시 읽고, 이관·부분 해결 이슈는 유지합니다. 통합 원격 branch·검증 worktree·종료된 output은 후속 절차에 따라 정리하고 공유 target·사용자 자료·fork branch는 보존합니다.

## Merge 후 contributor PR comment 계획

실제 merge SHA·통합 PR·최신 CI URL·아래 검토 범위와 남은 차이를 적고, [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 direct link로 포함합니다. 실제 줄바꿈의 `--body-file`로 게시한 뒤 API로 UTF-8 본문과 이미지를 재확인합니다.

대상 `added-issue7687-empty-host-hwpx` 1쪽의 실제 raw Native 100.00000% / WASM 100.00000%와 위 직접 판독 결론을 사용합니다. 아래 네 PNG의 실제 merge SHA raw URL을 Markdown 이미지로 표시합니다.

- [native review](../../../mydocs/pr/assets/planet6897_source27_representatives_20261022/images/added-issue7687-empty-host-hwpx/p001-native-review.png)
- [native overlay](../../../mydocs/pr/assets/planet6897_source27_representatives_20261022/images/added-issue7687-empty-host-hwpx/p001-native-overlay.png)
- [wasm review](../../../mydocs/pr/assets/planet6897_source27_representatives_20261022/images/added-issue7687-empty-host-hwpx/p001-wasm-review.png)
- [wasm overlay](../../../mydocs/pr/assets/planet6897_source27_representatives_20261022/images/added-issue7687-empty-host-hwpx/p001-wasm-overlay.png)
