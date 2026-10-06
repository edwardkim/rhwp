---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-22
---

# PR #7328 검토 기록 — HWP3 저장 줄 상자와 글자 높이 분리

- 원 PR: [#7328](https://github.com/edwardkim/rhwp/pull/7328)
- 관련 이슈: [#4680](https://github.com/edwardkim/rhwp/issues/4680)
- 원 code head: `2ec1bb83feb01f079d8a0fa9d50bd555729a1f8b`
- 통합 검토 branch: `review/planet-open-20260922`
- 적용 commit: `8934fb71d` (`cherry-pick -x`, 충돌 없음)

## 변경과 범위

HWP3의 저장 line height가 글자 높이가 아니라 줄 상자일 때, 백분율 줄간격을 다시 곱해 부풀지 않도록
대표 글자 높이로 되돌린다. 더 큰 글자나 inline control이 있는 문단은 적용하지 않는 좁은 조건이다.

## 기준 입력과 시각 검증

- 입력: `samples/issue4680/20117321_gijang_handover_bond_ledger_form.hwp`; 저장 제품 메타데이터는 `null`이므로
  2020 engine 선택 규칙을 적용했다.
- 기준 PDF: `pdf/issue4680/20117321_gijang_handover_bond_ledger_form-2020.pdf`, Hancom 2020 engine, 1쪽,
  SHA-256 `2d7698cfbdea7770b3ffe1e9fdd2d1488f6943453582da2f96125d600acfd63d`.
- Native·fresh WASM Visual Sweep 1쪽: 구조 flag 0건. pixel match 97.906%, ink match 21.965%;
  글꼴 raster 차이는 overlay 증거로만 기록한다.
- overlay: `mydocs/pr/assets/pr7328_review/native_overlay_001.png`,
  `mydocs/pr/assets/pr7328_review/wasm_overlay_001.png`.

## 검증

- 신규 검사 2건과 HWP3 line-height/page-count 관련 군 24건이 `-j 8` nextest에서 통과했다.
- workspace/native·WASM Clippy, build, manifest 검사는 통과했다.

## 최종 판정

**승인.** 실제 HWP와 한컴 PDF를 같은 commit에 보관했고, 1쪽 유지와 HWP3 기존 반례를 함께 확인했다.

## Merge 후 contributor PR comment 계획

통합 PR의 **실제 merge SHA와 최종 head CI URL**이 확정된 뒤 원 PR에 한국어 감사 코멘트를 남긴다.
저장 line box를 글자 높이로 재사용하지 않는 조건과 실제 한컴 2020 PDF 1쪽, Native·fresh WASM
검증 범위를 적고 아래 두 overlay를 본문 이미지로 포함한다.

```text
![Native overlay](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7328_review/native_overlay_001.png)
![WASM overlay](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7328_review/wasm_overlay_001.png)
```

글꼴 raster 차이로 ink match가 21.965%인 제한과 구조 flag 0건을 함께 밝힌다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.


본문은 UTF-8 파일로 만든 뒤 `gh pr comment --body-file`로 게시하고, 게시 후 한국어 본문·실제 merge SHA·CI URL·두 이미지 URL이 최종 head를 가리키는지 다시 확인한다.
