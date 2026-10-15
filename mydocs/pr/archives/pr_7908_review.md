---
kind: report
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-15
---

# PR #7908 self-review — HWP 편집 저장 보존과 저장 재열기 배치

## 최종 판정

**승인 — 저장 계약 해결과 메인터너가 명시한 시각 예외 범위의 self-review.** 정확한 제출 head의 GitHub CI가 성공했다. 자동 시각 gate 통과와 모든 backend 일치를 주장하지 않는다. 문서-only trailing head 검사와 별도 merge 승인은 남아 있다.

- PR: https://github.com/edwardkim/rhwp/pull/7908
- 제출 head: `6689eedaf36ba766b440015b14dd2fe88f8f116b`
- 제품 검증 source: `a5d56ea64d1b6ccb98ebc19e70a98b9709d1d982`; 제출 head까지 src/crates/tests 변경 없음.
- 기준 devel: `c0d330737fabae77ca2bf1706cba8514d0e8f1df`
- 원격 CI: https://github.com/edwardkim/rhwp/actions/runs/38590875511

기본 경로는 [collaborator self-merge](../../manual/pr_review/collaborator_self_merge.md), 보조 경로는 intake·local validation·visual fixture evidence·review-only fast-pass다. 실제 코드와 앞선 단계 기록을 대조했으며 다른 계정에 reviewer를 지정하지 않았다.

## 코드 검토

구역 끝 목록은 `Section.trailing_lists`에 소유시킨다. 본문 변경·raw cache 무효화 뒤에도 tag/level/payload 순서를 그대로 한 번 저장하며, master/memo 모델 자체를 바꾸면 projection seal이 그 차이를 검출한다. 직접 HWP parser와 Core load fixup 이후 경로 모두 같은 tail 봉인 helper를 소비한다. 미편집 direct parser의 정상 저장 거부는 첫 전체 검사에서 검출·보정했고 원래 검사를 유지했다.

인라인 수식 경계는 visible scalar 뒤 문자가 아니라 실제 원시 UTF-16 제어 슬롯에서 생산한다. Frame/reflow의 LineSeg → serializer → composer/paragraph 배치가 이를 소비하며 실제 큰 수식이 TextLine 소속을 유지함을 확인했다. Source equation baseline은 줄 정렬 속성이고 AST baseline은 내부 glyph 원점으로 구분한다. 문단 BorderText bit29는 resolved style → 실제 border Rect로 전달한다. sample ID·고정 pixel 보정은 추가하지 않았다.

## 공통 검토 범위

| 항목 | 판정과 증거 |
| --- | --- |
| 구현 근거·일반성 | 충족: 원본 raw 목록, memo marker, 제어 슬롯 경계 및 독립 Print. sample별 예외 분기 없음. |
| 측정·배치 일관성 | 해당 수식 경로 충족: 공통 raw 줄 경계·source baseline 생산/소비를 단계 3에 추적. 모든 수식 종류의 정밀 정렬은 미검증. |
| 최종 backend | Native·Studio·SVG 진단은 관측 및 명시적 시각 예외 수용. Canvas2D glyph 누락은 기준에도 존재하는 미충족 경로이며 정상 통과로 선언하지 않음. |
| 분할·이어받기 | 비해당: pagination·rowspan·continuation을 변경하지 않음. |
| 줄 소속·점유 높이 | 실제 API의 수식 5개 TextLine 소속과 저장 재열기 보존 확인. 다중 행 약 2px 위치 차이는 명시적 수용 예외. |
| 독립 증거 | 저장 계약 10건·원래 실패 검사 FAIL→PASS, 한컴 2020/2024 실제 열기·재저장, 같은 편집본의 독립 Print. 다른 문서 전쪽 시각은 미검증. |
| baseline 변경 | 비해당: 기존 검사·baseline 유지, 새 조판 golden 없음. |
| 시각 선행·확대 판독 | 자동 90% 미달 유지. 메인터너가 `{` 크기와 다중 행 vpos 차이를 예외 수용. 새 렌더링 assertion을 추가하지 않음. |
| 주장·검증 범위 | storage 해결과 수식 overlap/줄 소속 보정, 남은 shape/vpos/Canvas2D 차이를 분리. |

## 검증

전체 nextest 10,295 PASS / 0 FAIL / 45 skipped. Native Skia lib 4,041 PASS / 12 ignored, placeholder 2·direct PDF 4 PASS. fmt·세 Clippy·workspace build·고정 base manifest/unit tiers PASS. 단일 문자 lint 보정은 test-only이며 제품 code·완료된 전체 회귀를 반복하지 않았다.

최종 Docker WASM hash `d4c9802ea2a502630952f2a3c1b4a24a3c92c1c6c18ce33fb95174e0101d4b43`. 실제 CDP HTTP 응답과 root/public 동기화를 확인했다. Node 저장 hash `e00edfc63c35d5753e1369bad72de5187a485e9a6d0e9406f5746cb7c7cebf52`, 기존 수식·저장 배치와 같은 결과다. Browser 전체 render tree도 같다. 파일 바이트는 DocInfo 캐럿 위치만 달라 모든 파일 hash 동일로 주장하지 않았다.

같은 편집본·PDF·Print·compat 2022·96dpi·글꼴의 전쪽 1쪽 비교: Native 64.91873→77.01188, Canvas2D 65.33564→75.78392(두 캡처 모두 glyph 실패), SVG 진단 67.33301→77.20043. Decimal 하락≥0.01%p 0쪽. 자동 90% gate는 통과하지 않았으며 수용 예외로 구분했다. 관련 입력·대표 이미지·원본 hash는 저장소에 포함했다. `.env.local` 및 private font/glue/log/generated suite는 제외했다.

자세한 자료는 [단계 3의 호출 경로·전후 실행](../../working/task_m100_7899_stage3.md), [최종 검증](../../working/task_m100_7899_stage4.md)과 [보존한 입력·독립 Print](../../../samples/issue7899/README.md)에 연결한다.

## 정확한 제출 head의 GitHub CI

2026-10-15에 `6689eedaf36ba766b440015b14dd2fe88f8f116b`의 검사를 조회해 실패·대기 없이 성공 또는 정책상 skipped임을 확인했다.

- [CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/38590875511): archive a–d 빌드·검사, Native Skia, lint, frontend package gates와 집계 성공.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38590875953): Actions·JavaScript/TypeScript·Python·Rust Analyze와 GHAS CodeQL 성공.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38590875634), [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38590875854), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38590875391), [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/38594480905): 성공.
- `WASM Build`·Frontend unit 등 정책상 skipped는 정상으로 구분하며 로컬 fresh Docker WASM 검증을 생략했다는 의미가 아니다.

## 남은 병합 조건

사용자가 승인한 PR 등록·self-review에 따라 이 기록과 [오늘할일](../../orders/20261015.md)만 single-parent trailing commit으로 반영한다. 검증된 source·test·fixture·workflow를 바꾸지 않으므로 완료한 Rust/WASM/시각 검증을 반복하지 않는다. 최신 base의 실제 merge tree에서 공백·링크·기존 오늘 기록 보존을 확인한 뒤 push한다.

trailing head의 실제 fast-pass·required checks를 다시 확인하고 별도 merge 승인을 받아야 한다. 현재 merge·#7899 close는 수행 전이다. 이슈 종료 범위는 원래의 구역 끝 바탕쪽·메모 목록 저장 손실 계약이며, 남은 수식 shape/vpos와 Canvas2D 문제를 해결 완료로 확대하지 않는다.
