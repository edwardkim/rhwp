# #5715 손상된 축약 입력의 활성 검증 제외

## 제외 근거와 범위

2026-10-25 작업지시자가 Windows 한컴으로 확인한 축약 입력에 대해
**“손상된 파일임. 무시해야함.”**으로 판정했다. 이 지시에 따라 해당 입력의 수동 Print
요청을 철회하고 정상 배치를 주장하는 활성 회귀·공유 corpus 참조에서 제외한다.
90% 미달을 구현 보정 없이 통과 처리하거나 기대값을 완화한 것이 아니다.
**손상된 축약본의 제외는 원 이슈 #5715나 전체 원본의 해결 판정이 아니다.**

정확한 파일은 `samples/issue5715/float_chart_ghost_ladder_gap.hwp`였으며,
Windows 사본은 `C:\temp\rhwp-review-5715-522a9bc4-20261025\float_chart_ghost_ladder_gap.hwp`다.
삭제한 HWP는 21504바이트이며 SHA-256은
`fe6570871a907d8cff07f9a672286c26ff15a42c665c315ee634a44e39ac8282`다.
처음에는 `samples/` 밖의 이 경로에 보존했으나, 2026-10-26 작업지시자의
“손상된 파일 삭제” 지시에 따라 보관본도 삭제했다. 손상 바이트를 별도로 복사하거나
정상 입력으로 바꾸지 않는다. 아래 해시와 Git 이력은 삭제 대상을 식별하는 기록이며,
이 폴더를 정상 렌더링 corpus에 새로 등록하지 않는다.

## 원 해결 이력과 유지한 구현

원 보정은 `c627bb2f9414ea857a80ea312e270601dec6e4c2`이며 [PR #5849](https://github.com/edwardkim/rhwp/pull/5849)의
검토를 거쳐 [통합 PR #5853](https://github.com/edwardkim/rhwp/pull/5853),
merge `0f9ceeb19a7f36efebe77f03a4c7d39fb3f8c83f`로 반영됐다.
저장 vpos gap이 본문 위로 차트를 끌어올리는 조건의 renderer 보정은 유지한다.
원 이슈의 전체 124쪽 원본과 이 21KB 축약 입력을 같은 문서로 취급하지 않는다.

## 활성 유지·제거 목록

| 대상 | 처리 |
| --- | --- |
| `issue_5715_chart_stays_below_body_top_after_ghost_gap_reject` | 전용 모듈 전체를 [`.rs.txt`](issue_5715_float_chart_ghost_ladder_gap.rs.txt)로 보존하고 활성 수집에서 제거 |
| `default_column_rebuild_preserves_content_and_object_geometry` | 손상 HWP 한 개의 배열 항목만 제거; 나머지 입력·모든 assertion 유지 |
| 같은 공유 모듈의 다른 함수 | 그대로 유지 |
| IR field sweep | 해당 입력의 6행만 [제거 행 TSV](removed-ir-field-baseline.tsv)에 보존하고 활성 baseline에서 제거; 다른 행·허용치 유지 |
| `samples/issue5715/float_chart_ghost_ladder_gap.hwp` | 활성 corpus에서 제외한 뒤 증적 폴더에 남아 있던 손상 HWP 보관본도 삭제 |
| 원 renderer·정상 원본·다른 fixture·기준 PDF | 변경하지 않음 |

제외 전 공유 모듈 전체도 [`.rs.txt`](issue_7490_edited_paragraph_indent.rs.txt)로 보존했다.
원 전용 모듈에 별도 helper는 없다. baseline의 완전한 이전 파일은 아래 Git blob으로
복원하며 이번에 제거한 정확한 6행도 함께 남겼다.

## 복원 조건과 실제 검증

정상 한컴에서 열리는 원본 또는 독립적으로 재생성한 유효 입력과 변경 내역·같은 입력의
독립 Print PDF를 확보한다. 삭제한 손상 입력과 새 입력을 구분하고 관련 전체 페이지의
Native/fresh WASM raw 90% 이상 및 확대 판독을 완료한 뒤 새 이름의 관계형 회귀를
등록한다. 이전 690~710px 핀을 새 입력의 정답으로 재사용하지 않는다. 98% 완료 페이지의
px 검사는 현재 정본의 별도 조건을 충족할 때만 추가한다.

사용자 판정에 따른 제외이며 base 90% 미달 이관으로 대신 설명하지 않는다. 원 이슈를
close하거나 GitHub 댓글·새 이슈를 게시하지 않았다. 이 단계의 수집·관련 검사 결과는
아래에 실행 후 기록하며, 다른 실패가 남으면 그대로 보고한다.

## 제거 전 source와 보존 해시

- `tests/cases/issue_5715_float_chart_ghost_ladder_gap.rs`: source `e4c3714bbb227e5c25eb6b4ef5be52479dc1d751`,
  Git blob `16b621bfe5066ffdfa1efc6b2ef087205dabcf84`, SHA-256 `303eced86773f9a3bb34b879a08c2751ec65e312d8ca643f0234bf3b8bf603ca`.
- `tests/cases/issue_7490_edited_paragraph_indent.rs`: source `e4c3714bbb227e5c25eb6b4ef5be52479dc1d751`,
  Git blob `734ecbdf022f4ff0f9b2df5dce40e0ae3672858f`, SHA-256 `013779ba85e5769ac4881a77176ff31725f34e5d2ad1caa65a6a9b7685b08cef`.
- `samples/issue5715/float_chart_ghost_ladder_gap.hwp`: source `e4c3714bbb227e5c25eb6b4ef5be52479dc1d751`,
  Git blob `5831befb53dbbe0c4097fc924189d13c8f8e3b0d`, SHA-256 `fe6570871a907d8cff07f9a672286c26ff15a42c665c315ee634a44e39ac8282`.
- `tests/fixtures/ir_field_sweep_baseline.tsv`: source `e4c3714bbb227e5c25eb6b4ef5be52479dc1d751`,
  Git blob `579b8861bd3435ad1c384c430b84b769ce85e920`, SHA-256 `74ec0901ea3bc1830a15de7451853c75a861c6a30e9e525874bcd8d7b8dc2b01`.

### 이번 단계 실행 결과

자동 수집 `--prepare`가 성공해 1809 source / 7872 static test attribute /
48 integration target을 확인했다. 제외 전보다 전용 source 1개·검사 1개가 줄었으며,
보존 `.rs.txt`가 수집되지 않고 활성 경로에 손상 입력 참조가 남지 않은 것을 확인했다.
전체 Rust 실행 성공을 뜻하지 않는다.

공유 모듈의 기존 `default_column_rebuild_preserves_content_and_object_geometry`는
동결 Native224로 실제 1회 실행했으며 **1 FAIL**이다. 첫 대조군인
`issue5701/1530000-200800002_slice_p139_tac_reset_tail.hwp`의 12쪽 대 10쪽 차이가
기존처럼 남아 있어 이번 손상 파일 제외를 해당 검사 통과로 쓰지 않는다.
이 입력의 별도 분석·보정은 계속하며 기대값을 변경하지 않았다.

실제 실행 기록은 ignored `output/pr-review/semanticist21-20261019/`의
`logs/corrupt5715-suite-prepare-3663.log`, `pr8054-focused-after-3664-result.json`,
`logs/corrupt5715-retirement-integrity-3665.log`에 있다. 제품 runtime은
`8dc75860cec88ddf7c3f8aeee230b2ca7e2e5537`로 그대로이며 renderer를 다시 빌드하지 않았다.
