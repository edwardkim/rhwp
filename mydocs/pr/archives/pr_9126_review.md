# PR #9126 셀프 리뷰 — johndoekim 누름틀 두 기여 수용

## 최종 판정

**승인.** code head `529ba31acbdea187a2ed5d9528be419f26686132`의 필수 GitHub CI·실제 worker·정책 감사가 모두 성공했다. 제품·검증 source는 `4deadfa9638baffc3b4dc7e9d56a3edff9f83aeb`이며 이후 변경은 검토 기록뿐이다. 작업지시자가 승인한 Native 보호 10쪽 한 행의 기존 0.01%p 차이 외 예외는 없다.

원격 push·Open PR·CI·self-review·원 기여 보존 merge·이슈 종료·감사 댓글·운영 기록·동기화·정리까지 승인받았다. code/trailing head의 모든 CI·정책·실제 fast-pass와 최신 base 병합 tree를 확인하고 정상 merge commit `fb7367574f13e65111f162bb79238cb0af45b36e`로 통합했다. [후속 원장](../assets/johndoekim-20261116/closure.md)에서 이슈·댓글·duration·정리 상태를 구분한다.

## 검토 대상과 경로

- [수용 PR #9126](https://github.com/edwardkim/rhwp/pull/9126), devel 대상·Open·non-Draft·reviewer 미지정.
- 기본 경로: maintainer 일반. 새 upstream 수용 PR의 셀프 리뷰 기록에는 collaborator self 기록 순서와 review-only fast-pass A를 적용한다.
- 보조 절차: 접수·로컬 검증·시각 fixture·다수 PR·review-only·post-merge. CI 실행 정의 변경은 없어 옵션 2 대상이 아니다.
- 원 [#9081](https://github.com/edwardkim/rhwp/pull/9081)의 `3a2fcbf7d12d4f57fa3fcff33721ff7ed9059738`, [#9084](https://github.com/edwardkim/rhwp/pull/9084)의 `81dd920099bae78d8d66148ddf9614785760deac`를 그대로 조상으로 보존한다. fork branch는 push·삭제하지 않는다.
- 기준 devel: `6c1d134d42213ce0fb5f66b5ad2bf343b7b64fe0`. 문서 기록을 위해 code head에 devel을 반복 merge하지 않는다.

## 구현 주장과 독립 검증

| 항목 | 확인 결과 |
| --- | --- |
| 숨긴 안내문·표 줄 구성 | 저장 안내문이 없는 현재 표시 내용에 공통 글상자 재조판 적용. 줄 결과·뒤 문단 흐름·조합기·실제 표 배치의 소비 경로를 대조했다. 두 실제 저장본·독립 Print의 Native/fresh WASM 각 전 1쪽 raw 100.00%, 순서·간격·내부 포함 직접 판독 완료 |
| HWP 끝 표식 저장 | Storage 예산·오류 전달을 보존하며 이미 지난 FIELD_END를 뒤 컨트롤보다 먼저 방출. CLI와 최적화 WASM 공개 API의 실제 편집·반복 HWP 저장/재열기 및 독립 CFB 토큰 검사로 세 입력의 필드 값·표 순서 보존 확인. HWPX 교차 저장의 기존 정상 계약도 확인 |
| 기존 회귀·기준값 | 렌더 회귀는 관계 검사와 전체 쪽수 계약, 원 #9081 주석만 보완. IR baseline의 해소된 저장 손실 1행 제거는 원 #9084 변경이다. 보호 시각 하한·허용치는 수정하지 않았다 |
| 새 devel #9106 보존 | 실제 병합 후 분할 셀·Print TTF em·SVG 괘선·Native SVG 프로그램·CanvasKit 예산 계약 검사. 각주 50쪽+중첩 17쪽의 Native/WASM 각 67쌍 PNG 동일, 영향 영역 직접 판독 |
| 검증 입력 commit | 실제 입력·독립 PDF·최종 TSV·대표 PNG가 검토 commit에 있으며 해시 대조 완료. 공개 원본의 중복 사본과 사설 글꼴은 새로 배포하지 않았다 |
| 범위 구분 | [#9076](https://github.com/edwardkim/rhwp/issues/9076)·[#9082](https://github.com/edwardkim/rhwp/issues/9082) 해결 범위. 기존 줄 소속의 [#9058](https://github.com/edwardkim/rhwp/issues/9058)은 OPEN 유지, 낮은 기존 시각 대조군을 새 정상 golden으로 등록하지 않음 |

## 실행 결과와 승인 예외

- 최소 글꼴 전체 회귀 **13,632 PASS·0 FAIL·45 SKIP**, 집중 30 PASS, Native 공식 4,073 PASS, format·Native/WASM32/workspace Clippy·workspace build 통과.
- 실제 최적화 fresh WASM·Native 목표 전쪽 100.00%, Native 편람 383쪽과 공개 네 문서의 양쪽 공통 검사 8건 통과.
- 보호 **3,542쪽 중 승인된 1행 외 하락·누락·쪽수 불일치·측정 실패 0건**. 문서 `78e1720edfe2277d`의 Native 10쪽만 정본 99.86%에 대해 현재 base·후보 99.85%다. WASM은 99.86%이며 두 backend의 전 17쪽 base/후보 PNG는 각각 동일하다. 기호 누락은 독립 표본으로 복원했으며, 남은 raster 차이의 원인을 확정했다고 주장하지 않는다. [정확한 승인 범위](../assets/johndoekim-20261116/native-floor-decision.md).
- code head GitHub [실제 CI 결과](../assets/johndoekim-20261116/code-ci-529ba31.md) 모두 성공. archive 실제 13,438 PASS와 로컬 13,632 PASS를 구분한다. 신규 회귀 6건의 원격 실행도 확인했다.
- 자세한 원 [#9081 검토](pr_9081_review.md), [#9084 검토](pr_9084_review.md), [전쪽·저장·빌드 증거](../assets/johndoekim-20261116/latest-devel-revalidation.md)를 연결한다.

## 남은 merge와 후속 조건

문서-only trailing commit의 최신 base 병합 tree·공백·변경 Markdown 링크·기존 오늘할일 보존을 push 전에 확인한다. push 뒤 실제 fast-pass·최신 required check·mergeability를 확인하고 exact head를 지정해 merge commit으로 통합한다. merge SHA 확인 → 원 리뷰 archive·운영 기록 devel 반영 → 동기화 → #9076/#9082 완료와 원 PR 감사 댓글 → 자동 duration 결과 확인 → 전용 branch/worktree·output/debug 종료 정리 순서다.

## 실제 병합

2026-11-16T13:46:51+09:00, 최종 head `2d80477a8c817041576157aa2d786293b877fe97`를 정상 merge했다. admin 우회를 쓰지 않았다. 실제 merge tree `e6b05eea5e57351235e1c14ea13f88a4735e56b0`는 사전 병합 tree와 같고, 원 기여 두 SHA가 devel의 조상이다.

최종 문서 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/43805795160)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43805795053)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43805795140)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/43805795339)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/43805795118)·[최종 정책](https://github.com/edwardkim/rhwp/actions/runs/43805912460)이 모두 성공했다. 각 실제 preflight 로그의 재사용 후보가 `529ba31acbdea187a2ed5d9528be419f26686132`임을 확인했고, heavy worker의 정상 skip과 최종 aggregate 성공을 구분했다.

[게시한 셀프 리뷰](https://github.com/edwardkim/rhwp/pull/9126#pullrequestreview-5915154370)는 두 입력·두 backend의 review/overlay 8개를 포함하며 게시 전 응답·해시와 게시 후 본문을 확인했다.
