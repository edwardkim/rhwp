# Task #1116 Stage 17 분석 보고서 — PR 취소 및 k-water-rfp-2024 3mm 격자 차이

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 취소 PR: [edwardkim/rhwp#1118](https://github.com/edwardkim/rhwp/pull/1118)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: PR close 완료, k-water-rfp-2024 시각 차이 분석 중, 소스 수정 전 승인 대기

## 1. 작업지시자 피드백

작업지시자가 `k-water-rfp-2024.hwp`가 한컴오피스 3mm 격자 정답지와 다르다고 지적했다.

또한 PR은 준비만 하고, 실제 PR 생성과 진행은 별도 승인을 받아야 한다고 지시했다. 이에 따라 이미 열린 PR #1118을 close했다.

## 2. PR 처리

수행:

```text
gh pr close 1118 --repo edwardkim/rhwp --comment ...
```

확인:

```text
gh pr view 1118 --repo edwardkim/rhwp --json number,state,isDraft,mergeable,url,title
```

결과:

```text
state=CLOSED
url=https://github.com/edwardkim/rhwp/pull/1118
```

장기 규칙은 다음 문서에 반영했다.

```text
mydocs/manual/codex/docs_and_git_workflow.md
mydocs/manual/memory/feedback_pr_requires_explicit_approval.md
mydocs/manual/memory/MEMORY.md
```

## 3. k-water-rfp-2024 비교 대상

작업지시자 캡처의 상태바는 `5 / 27쪽`이고, 페이지 하단 인쇄 번호는 `-3-`이다. 따라서 실제 대상은 전역 5쪽과 6쪽 사이, 섹션 1의 page_num 3/4에 해당한다.

관련 산출물:

```text
output/poc/render-spacing/k-water-rfp-2024-stage17-pages5-6-grid-3mm/k-water-rfp-2024_005.svg
output/poc/render-spacing/k-water-rfp-2024-stage17-pages5-6-grid-3mm/k-water-rfp-2024_006.svg
output/poc/render-spacing/k-water-rfp-2024-stage17-pages5-6-grid-3mm/k-water-rfp-2024_005.png
output/poc/render-spacing/k-water-rfp-2024-stage17-pages5-6-grid-3mm/k-water-rfp-2024_006.png
```

## 4. 현재 RHWP 배치

현재 브랜치의 `dump-pages` 결과:

```text
=== 페이지 5 (global_idx=4, section=1, page_num=3) ===
PartialTable   pi=52 ci=0  rows=0..4  cont=false  4x4  vpos=12480  start_cut=[] end_cut=[3, 4, 2, 4, 4, 2, 21]

=== 페이지 6 (global_idx=5, section=1, page_num=4) ===
PartialTable   pi=52 ci=0  rows=2..4  cont=true  4x4  vpos=12480  start_cut=[3, 4, 2, 4, 4, 2, 21] end_cut=[]
```

SVG 기준 주요 y 좌표:

| 페이지 | 항목 | RHWP y |
| --- | --- | ---: |
| 5 | `품목매체분야세부내용` | 295.2px |
| 5 | `㉮ 정성적 평가...` | 577.0px |
| 5 | `㉯ 발표자료` | 885.4px |
| 5 | `◦ 유의사항...` | 995.5px |
| 6 | 이어지는 표 머리 `품목매체분야세부내용` | 128.8px |
| 6 | `금지, 제안서에 없는 내용 포함 금지` | 151.2px |
| 6 | 본문 `(1) 상기 세부내용별...` | 453.7px |

## 5. upstream/devel 대조

`upstream/devel` `dd4bbfed` worktree를 별도로 만들고 같은 파일을 빌드/덤프했다.

```text
/Users/tsjang/Documents/Codex/2026-05-25/3m-p3-3-line-seg-vpos/rhwp-upstream-devel
cargo build --bin rhwp
target/debug/rhwp dump-pages samples/k-water-rfp-2024.hwp -p 4
target/debug/rhwp dump-pages samples/k-water-rfp-2024.hwp -p 5
```

결과:

1. `pi=52`의 `rows=0..4`, `start_cut/end_cut`는 현재 브랜치와 동일하다.
2. SVG 텍스트 y 좌표도 현재 브랜치와 동일하다.
3. 따라서 `k-water-rfp-2024.hwp`의 3mm 격자 차이는 #1116 변경으로 새로 발생한 회귀가 아니라, 기존 #1105의 페이지 수 가드가 잡지 못한 `pi=52` RowBreak 표 분할 시각 차이로 분류한다.

## 6. 후속 구현 후보

소스 수정 전 승인 필요.

후속 승인 시 먼저 할 일:

1. 한컴 3mm 캡처 기준으로 p5 하단 `pi=52`의 마지막 표시 줄과 p6 상단 이어짐 표의 시작 y를 칸수로 계측한다.
2. `src/renderer/layout/table_layout.rs`의 RowBreak rowspan `start_cut/end_cut` 계산이 한컴의 셀 내부 line reset을 어느 기준으로 페이지 상단에 배치하는지 재검토한다.
3. `tests/issue_1105.rs`에 페이지 수뿐 아니라 `k-water-rfp-2024.hwp` p5/p6의 `pi=52` 절단 위치 또는 SVG 좌표 회귀 테스트를 추가한다.
4. #1116 sample16 보정과 #1105 k-water 표 분할 보정이 서로 독립인지 회귀 묶음으로 확인한다.
