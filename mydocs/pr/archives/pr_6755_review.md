---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-05
pr: 6755
author: planet6897
---

# PR #6755 검토 - 글자 없는 문단의 그림과 표를 같은 줄에 배치

## 판정: 메인터너 보정 됨, 수용 가능

필수 공개 샘플 등록 및 누락 실패 메인터너 보정이 완료됐고, 현재 후보 3쪽에서 그림·표의 동일 줄 배치와 최하단 캡션을 한컴 PDF와 직접 대조했다. 보류 사유는 해소됐다.

이 판정은 아래에 고정한 통합 후보의 로컬 검토 결과다. 원 PR을 직접 merge했다는 뜻이나 아직 생성하지 않은 통합 PR의 GitHub CI 성공을 뜻하지 않는다. 통합 PR 생성·push는 작업지시자 승인을 받았으며, merge는 최신 head CI 확인과 별도 승인 대상이다.

## 검토 기준과 체리픽 출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/6755
- 기여자: `planet6897`; 사전 리뷰어: `jangster77`.
- 검토한 원 PR head: `8e609cfacc4e3428a162a0189362dc0c14417743`.
- 이 PR의 마지막 체리픽 commit: `309aa25d0ec71833d61a81329f88c9c16b820fed`.
- 통합 브랜치: `review/ci-green-batch-20260905`.
- 공통 base: `f7426ad95f2eb6f30732749bc50b32d60a3f343a`.
- 최종 코드 후보: `e207bb081ce3e56bad4f97c766add153ed28c28e`. [#6751 메인터너 TS 보정](../assets/ci_green_batch_20260905/pr6751-maintainer-correction.patch)과 [overflow-cell 기준선 강화](../../../tests/fixtures/overflow_cell_baseline.tsv)를 별도 보정 commit으로 포함했다. 위 작업 트리에서 통과한 검증 내용과 동일하며 리뷰·증적 문서는 뒤의 별도 commit이다.
- 현재 통합 검토 대상은 #6736, #6743, #6745, #6746, #6747, #6750, #6751, #6752, #6755의 9건이다.
- [후보 파일 해시](../assets/ci_green_batch_20260905/candidate-sha256.txt), [공통 검증·시각 증적](ci_green_batch_20260905_visual_sweep.md).

## 변경 및 코드 검토

- 원 이슈의 초기 행 높이 가설과 달리 근인은 TAC 그림·표의 동일 줄 배치였다. 너비 합산, 글자 없는 줄의 표 렌더링, 단일 빈 줄의 TAC 소유 범위를 함께 수정한다.
- 메인터너 보정 `8204bdb29999fdbf96c3227fa25ee860211fd3b1`에서 `samples/issue6754/156585314-ssagirang-barley.hwp`를 등록하고 개인 경로·환경변수·샘플 누락 조기 종료를 제거했다.
- 제품 레이아웃 조건, 좌표 허용 범위, 기존 기준선을 이번 메인터너 보정에서 변경하지 않았다.

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

- [Analyze (rust)](https://github.com/edwardkim/rhwp/actions/runs/33945334443/job/101250314618): `SUCCESS`.
- [adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/33945334515/job/101250310064): `SUCCESS`.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/33946004891): `SUCCESS`.

### 기준선 재검토 보정과 재검증

overflow-cell 기준선을 `48→32`로 강화하고 0개로 해소된 87개 허용 행을 제거한 뒤 전체 Rust 회귀를 다시 실행했다. **9,043 통과, 46 skip, 실패 0; 241.513초, exit 0**이며 overflow-cell 16개 파티션도 모두 통과했다. 새 dump의 비영 문서 12건은 강화한 기준선과 정확히 일치했다. [기준선 보정 증적](../assets/ci_green_batch_20260905/overflow-cell-ratchet-check.json). 이번 추가 실행은 Rust 전체 회귀이며, 제품 소스가 같은 Native Skia·lint·WASM 및 TS 보정 후 Studio·브라우저 검증은 기존 통과 결과를 유지한다.

## 시각 증적 및 기능 검증

한컴 engine 2020 기준 PDF 3쪽 중 3쪽을 144 DPI webfont visual sweep으로 비교했다. 보리 원맥 사진 오른쪽의 4×8 표가 같은 줄에 놓였고, 하단의 <내한성>, <보리호위축병>, <보리 새싹> 캡션이 모두 표시된다. 글꼴·일부 셀 경계 및 사진 폭은 완전한 픽셀 일치가 아니며 이번 TAC 배치 해결 범위와 구분한다.

![PR #6755 현재 후보 증적](../assets/ci_green_batch_20260905/issue6754-compare-p003.png)

기여자의 과거 A/B 이미지와 이번 현재 후보 이미지는 공통 증적 문서에서 출처를 구분했다. 전체 픽셀 일치율만으로 승인하지 않았으며, 이번 PR의 명시된 계약을 기준으로 판정했다.

## 범위와 잔여 조건

최하단 텍스트가 용지 안에 있다는 좌표 테스트만으로 캡션 누락을 검출할 수 없으므로 실제 캡션 존재를 이번 시각 대조에서 별도로 확인했다.

현재 로컬 검토 범위의 머지 보류 사유는 없다. 실제 원격 병합 전에는 승인받은 최종 통합 head의 required checks와 `MERGEABLE`/`CLEAN`을 다시 확인해야 한다.

## Merge 후 contributor PR comment 계획

이 절은 게시 초안이며 아직 PR·이슈 댓글이나 close를 수행하지 않았다. [후속 처리 7.3·7.4](../../manual/pr_review/post_merge.md)와 [리뷰 기록 3.1](../../manual/pr_review/intake_and_review.md)을 따른다.

- 원 PR: [#6755](https://github.com/edwardkim/rhwp/pull/6755). 직접 merge가 아니라 출처를 보존한 체리픽 통합으로 수용했음을 명시한다.
- 이슈 처리 범위: Fixes 대상 [#6754](https://github.com/edwardkim/rhwp/issues/6754)에 PR과 같은 그림·표·캡션 이미지를 직접 표시한다. 실제 해결 범위와 auto-close를 확인하고 OPEN이면 승인 범위에서 종료한다.
- 검증 내용과 한계: engine 2020 PDF 3쪽 중 p3 비교: flagged=0/1, pixel match=90.28799%, visual_accuracy_proxy_percent=45.07580%. 원맥 사진과 4×8 표의 같은 줄 배치 및 하단 세 캡션을 확인했다. 글꼴·일부 셀 경계·사진 폭 차이는 남아 있다. 메인터너 보정은 공개 sample 필수 로딩이며 제품 레이아웃 허용치를 완화하지 않았다.
- 통합 PR 번호와 실제 merge SHA, 원 head·메인터너 보정 SHA를 구분하고, 최종 통합 head의 PR CI 및 해당 merge SHA의 devel CI 실제 run/job direct link와 결과를 게시 시점에 채운다. 아직 없는 번호·SHA·CI 성공을 미리 확정하지 않는다.
- 로컬 검증은 위 표의 실제 통과 수와 skip·미검증 범위만 요약한다. 원시 `.log`를 커밋·첨부하거나 로그 링크를 게시하지 않는다.
- 이 리뷰의 고정 링크: [개별 검토 기록](https://github.com/edwardkim/rhwp/blob/<merge-commit-sha>/mydocs/pr/archives/pr_6755_review.md).
- 이미지의 안정 경로는 `mydocs/pr/assets/ci_green_batch_20260905/`다. 아래 Markdown 이미지를 PR 댓글과 해당 이슈 댓글 본문에 직접 넣는다. 문서 링크나 이미지 다운로드 링크만으로 대체하지 않는다.

### 댓글에 직접 표시할 증적

```markdown
- 문서 비교 기준: [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)

![PR 6755 issue6754-compare-p003 검증](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/ci_green_batch_20260905/issue6754-compare-p003.png)
```

수치의 `pixel match`는 전체 canvas 자동 일치율, `visual_accuracy_proxy_percent`는 내용 픽셀 중심 자동 일치율 보조값이다. 높을수록 raster가 비슷하고 낮을수록 잉크 위치·형태 차이를 검토해야 한다. 사람의 판정 정확도나 전체 문서 합격률이 아니며, `flagged=0`도 모든 glyph 일치를 보장하지 않는다고 댓글에 함께 설명한다.


### 함께 남길 자료 링크

- [issue6754-summary.json](https://github.com/edwardkim/rhwp/blob/<merge-commit-sha>/mydocs/pr/assets/ci_green_batch_20260905/issue6754-summary.json)
- [issue6754-run-manifest.json](https://github.com/edwardkim/rhwp/blob/<merge-commit-sha>/mydocs/pr/assets/ci_green_batch_20260905/issue6754-run-manifest.json)
- [한컴 기준 PDF: issue6754-156585314-2020.pdf](https://github.com/edwardkim/rhwp/blob/<merge-commit-sha>/pdf/issue6754-156585314-2020.pdf)

### 게시·수정 및 종료 순서

1. 승인된 통합 PR merge 및 해당 merge SHA의 devel CI 성공 뒤, 이미지·PDF·리뷰가 그 commit과 devel에 실제 존재하는지 확인한다. 모든 `<merge-commit-sha>`를 확정 SHA로 치환한 뒤에만 게시한다.
2. PR·이슈 기존 댓글을 조회한다. 같은 merge·증적의 댓글이 있으면 그 댓글 ID를 수정하며 새 댓글을 중복 등록하지 않는다. 이미 내용이 완전하면 수정도 생략하고 permalink만 남긴다.
3. 신규 댓글은 UTF-8 BOM 없는 본문 파일을 `gh pr comment` 또는 `gh issue comment --body-file`로 게시한다. 기존 댓글은 같은 본문 파일을 `gh api --method PATCH repos/edwardkim/rhwp/issues/comments/<comment-id> -F body=@<body-file>`로 반영한다.
4. 게시·수정 뒤 API로 body를 재조회하여 한글·실제 줄바꿈·SHA 고정 이미지 Markdown·대상 PR/이슈와 첨부 범위를 대조한다. 인증 값이나 로컬 임시 경로를 댓글에 노출하지 않는다.
5. closing keyword와 실제 해결 범위를 확인한다. auto-close된 이슈도 검증 댓글이 없다면 남기고, 이미 있는 동일 댓글은 재사용한다. 미해결·참조 이슈는 종료하지 않으며 원 PR은 체리픽 수용 사실을 기록한 뒤 필요한 close만 수행한다. contributor fork branch는 보존한다.

## 이전 후보 이력

이전 후보 `a6a8001b30babbb9961c8b1568abf05d72a3be80`의 9,043 통과/Studio 1,461 통과는 과거 결과다. 이번 판정의 근거는 위 **현재 9건 통합 후보 재실행** 및 TS 보정 후 검증이며, 과거 결과는 이력으로만 구분한다.
