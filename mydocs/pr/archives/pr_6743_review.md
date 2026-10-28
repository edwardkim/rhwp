---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-05
pr: 6743
author: lpaiu-cs
---

# PR #6743 검토 - 묶음 속성의 무변경 적용 시 원본 변환 보존

## 판정: 승인

무변경 원본 변환 보존 및 실제 변경 시 무효화 계약과 현재 후보 전체 회귀가 통과했다. 원 기여 범위의 차단 결함은 없으며 별도 제품 보정 없이 수용 가능하다.

이 판정은 아래에 고정한 통합 후보의 로컬 검토 결과다. 원 PR을 직접 merge했다는 뜻이나 아직 생성하지 않은 통합 PR의 GitHub CI 성공을 뜻하지 않는다. 통합 PR 생성·push는 작업지시자 승인을 받았으며, merge는 최신 head CI 확인과 별도 승인 대상이다.

## 검토 기준과 체리픽 출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/6743
- 기여자: `lpaiu-cs`; 사전 리뷰어: `jangster77`.
- 검토한 원 PR head: `ebc416c8df0e9c83012d4d46786b3504365303e0`.
- 이 PR의 마지막 체리픽 commit: `8d82cb00db2a093689f32d4e5472c159632f379c`.
- 통합 브랜치: `review/ci-green-batch-20260905`.
- 공통 base: `f7426ad95f2eb6f30732749bc50b32d60a3f343a`.
- 최종 코드 후보: `e207bb081ce3e56bad4f97c766add153ed28c28e`. [#6751 메인터너 TS 보정](../assets/ci_green_batch_20260905/pr6751-maintainer-correction.patch)과 [overflow-cell 기준선 강화](../../../tests/fixtures/overflow_cell_baseline.tsv)를 별도 보정 commit으로 포함했다. 위 작업 트리에서 통과한 검증 내용과 동일하며 리뷰·증적 문서는 뒤의 별도 commit이다.
- 현재 통합 검토 대상은 #6736, #6743, #6745, #6746, #6747, #6750, #6751, #6752, #6755의 9건이다.
- [후보 파일 해시](../assets/ci_green_batch_20260905/candidate-sha256.txt), [공통 검증·시각 증적](ci_green_batch_20260905_visual_sweep.md).

## 변경 및 코드 검토

- `object_ops/common.rs`의 변환 지문을 그림과 묶음이 공유하며, 묶음 속성 적용 전후 지문이 달라질 때만 `raw_rendering`을 비운다.
- 본문 경로와 셀 안 도형 경로에 같은 판정을 적용한다. 지문은 속성 대입 이전에 캡처한다.
- 무변경 저장 비교는 원본 파일 자체가 아니라 구역 패스스루만 무효화한 대조군과 수행하므로, 다른 직렬화 비용을 이번 결함으로 오인하지 않는다.

## 실제 검증 결과

| 검증 | 이번 실행 결과 |
| --- | --- |
| 기준선 강화 후 Rust 전체 nextest | 9,043 통과, 46 skip, 실패 0; 241.513초, exit 0 |
| Native Skia lib | rhwp 3,930 통과/13 ignored; 나머지 workspace lib 182 통과 |
| Native Skia 그림/PNG | 2 통과 |
| Native Skia 직접 PDF | 4 통과 |
| 메인터너 TS 보정 후 Studio 전체 | 1,399 통과, 1 skip, 실패 0 |
| TypeScript --noEmit | 통과 |
| host/WASM/전체 대상 clippy | 모두 통과, -D warnings |
| workspace build, fmt, suite manifest | 모두 통과 |
| WASM | 현재 Rust 후보 host build 성공; Docker 사용 안 함 |

검증 통과 사실과 실행 수치는 위 표에 기록했다. Rust 회귀 뒤 추가한 제품 보정은 TypeScript 두 파일뿐이므로 Rust 소스·fixture는 동일하며, TS 보정 뒤 Studio 전체·타입 검사·해당 브라우저 검증을 다시 수행했다. 공통 문서에 명령·환경·통과 결과를 기록했다.

원 PR의 아래 CI는 해당 원 head에서 앞서 수집한 이력이며, 오늘의 통합 후보 CI로 재사용하지 않는다. 분석 결과를 중복 조회한 최신 원격 상태라고 주장하지 않는다.

- [Analyze (rust)](https://github.com/edwardkim/rhwp/actions/runs/33928588300/job/101202403564): `SUCCESS`.
- [adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/33928588353/job/101202393369): `SUCCESS`.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33929558341): `SUCCESS`.

### 기준선 재검토 보정과 재검증

overflow-cell 기준선을 `48→32`로 강화하고 0개로 해소된 87개 허용 행을 제거한 뒤 전체 Rust 회귀를 다시 실행했다. **9,043 통과, 46 skip, 실패 0; 241.513초, exit 0**이며 overflow-cell 16개 파티션도 모두 통과했다. 새 dump의 비영 문서 12건은 강화한 기준선과 정확히 일치했다. [기준선 보정 증적](../assets/ci_green_batch_20260905/overflow-cell-ratchet-check.json). 이번 추가 실행은 Rust 전체 회귀이며, 제품 소스가 같은 Native Skia·lint·WASM 및 TS 보정 후 Studio·브라우저 검증은 기존 통과 결과를 유지한다.

## 시각 증적 및 기능 검증

화면 변경이 아닌 원본 변환 바이트 보존 계약이다. group-box.hwp의 빈 속성, 동일 값 재적용, 실제 크기 변경 3개 회귀를 현재 전체 실행에서 확인했다. 새로운 셀 내부 묶음 전용 실물 시각 대조까지 수행했다고 확대하지 않는다.

기여자의 과거 A/B 이미지와 이번 현재 후보 이미지는 공통 증적 문서에서 출처를 구분했다. 전체 픽셀 일치율만으로 승인하지 않았으며, 이번 PR의 명시된 계약을 기준으로 판정했다.

## 범위와 잔여 조건

커넥터의 변환 재계산 경로와 묶음 이외 개체의 모든 raw 보존 문제를 해결한 것으로 확대하지 않는다. 이 승인과 통합 PR의 공통 린트·WASM·CI 게이트는 별개다.

현재 로컬 검토 범위의 머지 보류 사유는 없다. 실제 원격 병합 전에는 승인받은 최종 통합 head의 required checks와 `MERGEABLE`/`CLEAN`을 다시 확인해야 한다.

## Merge 후 contributor PR comment 계획

이 절은 게시 초안이며 아직 PR·이슈 댓글이나 close를 수행하지 않았다. [후속 처리 7.3·7.4](../../manual/pr_review/post_merge.md)와 [리뷰 기록 3.1](../../manual/pr_review/intake_and_review.md)을 따른다.

- 원 PR: [#6743](https://github.com/edwardkim/rhwp/pull/6743). 직접 merge가 아니라 출처를 보존한 체리픽 통합으로 수용했음을 명시한다.
- 이슈 처리 범위: [#6740](https://github.com/edwardkim/rhwp/issues/6740), [#5890](https://github.com/edwardkim/rhwp/issues/5890)에는 변환 원본 바이트 보존과 실제 크기 변경 시 무효화 범위만 기록한다. 부분 구현·관련 표기만으로 전체 이슈를 종료하지 않는다.
- 검증 내용과 한계: 변환 원본 바이트 보존 및 크기 변경 시 무효화 회귀 3건의 통과 사실을 기록한다. 비시각 계약 검증이므로 PNG/PDF를 인위적으로 붙이지 않고 시각 일치율도 기재하지 않는다.
- 통합 PR 번호와 실제 merge SHA, 원 head·메인터너 보정 SHA를 구분하고, 최종 통합 head의 PR CI 및 해당 merge SHA의 devel CI 실제 run/job direct link와 결과를 게시 시점에 채운다. 아직 없는 번호·SHA·CI 성공을 미리 확정하지 않는다.
- 로컬 검증은 위 표의 실제 통과 수와 skip·미검증 범위만 요약한다. 원시 `.log`를 커밋·첨부하거나 로그 링크를 게시하지 않는다.
- 이 리뷰의 고정 링크: [개별 검토 기록](https://github.com/edwardkim/rhwp/blob/<merge-commit-sha>/mydocs/pr/archives/pr_6743_review.md).
- 비시각 PR이므로 별도 PNG/PDF 첨부는 하지 않는다. 위 검증 요약과 SHA에 고정한 리뷰 기록을 근거로 남긴다.


### 게시·수정 및 종료 순서

1. 승인된 통합 PR merge 및 해당 merge SHA의 devel CI 성공 뒤, 이미지·PDF·리뷰가 그 commit과 devel에 실제 존재하는지 확인한다. 모든 `<merge-commit-sha>`를 확정 SHA로 치환한 뒤에만 게시한다.
2. PR·이슈 기존 댓글을 조회한다. 같은 merge·증적의 댓글이 있으면 그 댓글 ID를 수정하며 새 댓글을 중복 등록하지 않는다. 이미 내용이 완전하면 수정도 생략하고 permalink만 남긴다.
3. 신규 댓글은 UTF-8 BOM 없는 본문 파일을 `gh pr comment` 또는 `gh issue comment --body-file`로 게시한다. 기존 댓글은 같은 본문 파일을 `gh api --method PATCH repos/edwardkim/rhwp/issues/comments/<comment-id> -F body=@<body-file>`로 반영한다.
4. 게시·수정 뒤 API로 body를 재조회하여 한글·실제 줄바꿈·SHA 고정 이미지 Markdown·대상 PR/이슈와 첨부 범위를 대조한다. 인증 값이나 로컬 임시 경로를 댓글에 노출하지 않는다.
5. closing keyword와 실제 해결 범위를 확인한다. auto-close된 이슈도 검증 댓글이 없다면 남기고, 이미 있는 동일 댓글은 재사용한다. 미해결·참조 이슈는 종료하지 않으며 원 PR은 체리픽 수용 사실을 기록한 뒤 필요한 close만 수행한다. contributor fork branch는 보존한다.

## 이전 후보 이력

이전 후보 `a6a8001b30babbb9961c8b1568abf05d72a3be80`의 9,043 통과/Studio 1,461 통과는 과거 결과다. 이번 판정의 근거는 위 **현재 9건 통합 후보 재실행** 및 TS 보정 후 검증이며, 과거 결과는 이력으로만 구분한다.
