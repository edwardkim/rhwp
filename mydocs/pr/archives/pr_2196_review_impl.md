# PR #2196 처리 계획 - CanvasKit 글머리 기호 폰트 보정

## 커밋

| SHA | 제목 | 비고 |
|---|---|---|
| `a8fd0c5e2624979ab6efa8ddb97b4229efb30bf6` | `task 2190: CanvasKit 글머리 기호 폰트 보정` | 코드/폰트/검증/보고서 |
| 문서 보강 커밋 | `docs: PR #2196 검토 기록` | review 문서, PR asset, 오늘할일 |

## Stage 구성

1. PR review 문서와 PR 번호 기반 visual asset을 PR head에 포함한다.
2. `git diff --check`와 변경 범위 확인 후 remote push 한다.
3. PR head 최신 커밋 기준 GitHub Actions 통과를 확인한다.
4. 작업지시자 승인 상태에서 PR #2196을 merge 한다.
5. `devel`을 `upstream/devel`로 fast-forward sync 한다.
6. #2190 close 여부를 확인하고, 검증 요약과 visual asset 링크를 issue 후속 코멘트로 남긴다.
7. PR head 원격 브랜치와 로컬 작업 브랜치를 정리한다.

## Merge 전 조건

- PR head 최신 커밋 기준 GitHub Actions 통과
- PR review 문서와 visual asset이 PR diff에 포함됨
- `pdf/hwpx_sample2-2020.pdf`가 기준 PDF로 PR diff에 포함됨
- 작업지시자 승인

## 오늘할일

`mydocs/orders/20260711.md`에 #2196 self PR 처리 항목을 같은 PR head 커밋으로 추가한다.

## 후속 코멘트 초안

```markdown
PR #2196 merge로 #2190을 처리했습니다.

확인한 내용:

- CanvasKit 기본 WOFF2에서 `■`, `▪`, `□`, `○`, `─`, `가` glyph ID가 모두 non-zero임을 확인
- `hwpx_sample2.hwpx` 브라우저 CanvasKit 표시에서 글머리 기호가 tofu 없이 표시됨을 확인
- HWP 2020 MCP 기준 PDF: `pdf/hwpx_sample2-2020.pdf`, 29 pages
- visual sweep p1: pixel match 82.46432%, visual accuracy proxy 52.99198%

시각 검증 자료:

![PR #2196 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/devel/mydocs/pr/assets/pr_2196/visual_sweep_review_001.png)

visual sweep의 남은 차이는 native SVG export와 HWP 2020 PDF 사이의 기존 font/layout fidelity 잔여로 보며,
이번 #2190의 직접 blocker로 보지 않았습니다.
```
