---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7245 검토

## 최종 판정

**승인** — 통합 로컬 필수 검증 완료. 원 PR 직접 merge가 아닌 보정 통합 head를 수용한다.

검토 범위에서 차단 결함을 발견하지 않았다. #7248 충돌은 공용 ID 할당과 열별 문단 스타일 인자를 모두 보존해 해소했다.

이 PR의 구조적 검토는 승인한다. 통합 묶음의 보류는 해소됐으며 최종 head CI가 통과하기 전 원격 merge는 하지 않는다.

## 최종 통합 검증

보정 제품 코드 `88f2f00da8412c769f34ef6bc3b72bc13402557b`. contributor 원 head는 아래 provenance에 별도 기록했다.
[공동 최종 검증](pr_7244_review_impl.md#최종-통합-검증)의 전체 nextest, Native Skia 3종,
fmt·Clippy 3종·workspace build·정책 검사, Studio 타입·단위·production build를 통과했다.
원 PR head CI를 재사용한 통과 주장과 구분한다. 해당 입력은 최종 Native/fresh WASM으로
다시 Visual Sweep했고 compare·standalone overlay·review와 남은 차이를 확인했다.
렌더 변경이 없는 진단·scaffold ID·래칫 자체에는 별도 시각 통과를 주장하지 않는다.

작업지시자의 PR 생성·CI 모니터링·merge·후속 처리 승인을 받았다. 통합 원격 최종 head의
CI·mergeability 확인은 아직 남아 있으며 완료 전 merge하지 않는다.

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7245](https://github.com/edwardkim/rhwp/pull/7245) — 수정: 표 생성 경로의 개체 id 를 공용 할당기로 모은다 (#7231) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `ec9da112a380b9ae5cf2443a03521781974ab01b` |
| 규모 | 3 files, +200 / -26 |
| 조회 당시 mergeability | `MERGEABLE` / `CLEAN` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 최초 검토 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `ec9da112a380b9ae5cf2443a03521781974ab01b` | `e9ea3ca15d1c2e1fecdb507932eba5cda7000d2e` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35268675330) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35268675350) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35268674229) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35268675303) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35268675483)

## 범위·조판 계약 검토

관련 이슈: [#7231](https://github.com/edwardkim/rhwp/issues/7231). scaffold와 HTML 표 import가 공용 Allocator를 사용하고 IR instance_id와 raw 제어 데이터에 같은 값을 기록한다.

used_instance_ids → Allocator → common.instance_id와 raw INSTANCE_ID → HWP/HWPX 저장을 대조했다. 배치·paint 변경이 없어 이 PR 자체의 Visual Sweep은 비해당이다.

주요 소비 경로: [src/document_core/html_table_import.rs](../../../src/document_core/html_table_import.rs), [src/scaffold/builder.rs](../../../src/scaffold/builder.rs).

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 최초 검토 검증과 한계

- 통합 제품의 `issue_7231_scaffold_unique_table_ids`: **4 tests run: 4 passed, 215 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 최초 검토 당시에는 전체 게이트가 미실행이었다. 아래 과거 기록을 최종 상태로 해석하지 않으며, 최신 결과는 최종 통합 검증 절과 공동 원장을 따른다.

파일 대신 코드에서 문서를 생성하는 구조·왕복 계약 검사다. 다중 표의 ID 중복과 IR/raw 일치, 형식 왕복 4건이 통과했다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 이슈·다음 단계

#7231의 scaffold/HTML 표 ID 범위에 한해 종료 후보. 다른 개체 생성 경로까지 해결했다고 쓰지 않는다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
