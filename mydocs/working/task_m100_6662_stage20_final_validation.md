---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage20_final_validation.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 20단계: 제목 보정 후 최종 검증

코드 후보: `46e6761db`. #6712까지 완료하고 PR을 준비한다. 외부 push/PR 생성/close는 하지 않는다.

## 계획

1. fmt 후 파생 suite 준비 및 전체 nextest를 실행한다. #6712 계약/기존 SVG 제어군 결과는
   19단계에 고정했지만, 이를 전체 회귀 결과로 대신하지 않는다.
2. Native/WASM/all-target Clippy, workspace build, manifest check를 순차 실행한다.
3. 두 새 원본의 security 검사, IR/overflow baseline dump 및 차이를 확인한다.
4. native-Skia lib/공식 제어군, Docker 부재에 따른 명시적 WASM `--no-opt` 진단을 실행한다.
5. 기존 한국어2020/중국어2024 PDF로 두 문서 1/2쪽 visual sweep을 다시 실행한다.
   제목 확대, 본문, 중첩 표, footer를 직접 확인하고 18단계 거절된 패널을 대체한다.
6. 18단계의 코멘트 계획을 새 패널·해시·판정으로 확정한다. 중간 로그/JSON/SVG/PNG는
   커밋하지 않는다. 오늘할일/번호가 붙은 PR review는 생성 후 같은 PR trailing 단계에 둔다.

## 결과

이 단계는 실패 원인을 확정한 checkpoint이며 최종 수용은 보류한다.

- fmt/prepare exit 0. 전체 nextest: 9,095개 실행, 9,093 통과, 2 실패, 46 skip,
  실행 시간 452.783초. 이후 lint/native-Skia/WASM 단계는 fail-fast로 실행하지 않았다.
- 실패는 `svg_snapshot::form_002_page_0`, `svg_snapshot::issue_157_page_1`이다.
  XML 구조 비교 결과 각각 124/28개의 `함초롬바탕` text에 `textLength`/`lengthAdjust`만 추가됐다.
  원점·요소 수·텍스트는 동일하다. 원본 golden은 수정하지 않는다.
- 원인: 한글 폭이 em보다 작다는 조건만으로 비례폭 글꼴이라고 잘못 판단했다.
  `HCR Batang`은 em=1000, 모든 한글 폭=970으로 일정하지만 보정 대상에 포함됐다.
  `휴먼굵은팸체`는 em=512, 조합별 폭 253~397로 실제 비례폭이다.
- 다음 단계에서는 한글 metric의 양수 폭이 실제로 여러 값인지 확인하여 등폭 한글을 제외한다.
  제목 보정과 기존 두 스냅샷을 함께 검증하고 전체 검증을 다시 수행한다.
- 아래 시각 결과는 `46e6761db`의 중간 결과이며 최종 PR 증적 확정은 다음 단계 이후다.
  임시 actual SVG와 검증 로그는 커밋하지 않는다.

#6699는 dx=-1.41px가 남으므로 Ref이며 Closes로 포함하지 않는다.

## 최종 시각 증적과 코멘트 계획

- 두 원본과 기존 PDF의 경로/해시는 18단계 기록과 같다. 새 CLI SHA-256은
  `47c5248eebcaa4d9f040a1bf91183d5afa31fccd57a243a711968e2b77fbe824`다.
  `visual_sweep.py --hwp <원본> --pdf <기존 PDF> --pages 1,2
  --rhwp-bin target/pr-review/release-test/rhwp --key <6712-ko 또는 6712-zh>
  --out /tmp/rhwp-stage20-sweep/<key>`를 실제 실행했다. Chrome은 18단계와 같다.
- 한국어·중국어 모두 PDF 2쪽, 새 SVG 2쪽, 선택한 1/2쪽 래스터 각 2장, sweep exit 0.
  수정 전 SVG는 18단계에서 provenance를 확인한 `1f861362a`의 3쪽 결과다.
- 최종 4개 패널의 모든 페이지와 헤더·본문·하단을 직접 열어 확인했다. 한국어 1쪽 제목
  겹침이 사라졌으며, 두 문서의 개선된 본문/그림 어울림과 2쪽 footer 배치는 유지된다.
  제목의 실제 ink 비중첩 1/2배 검증은 19단계에 기록했다. 전각 폰트로의 완전한 디자인
  일치가 아니라 원본 advance를 보존한 대체 글리프 폭 보정이다.
- GitHub #6712 코멘트에는 다음 4개를 실제 표시한다. 18단계의 거절된 파일/해시가 아니라
  이번에 재산출한 파일을 사용한다. 다른 임시 확대 캡처·JSON·SVG·로그는 보존 커밋에서 제외한다.
  - `mydocs/pr/assets/issue_6712_ko_p1_compare.png`
  - `mydocs/pr/assets/issue_6712_ko_p2_compare.png`
  - `mydocs/pr/assets/issue_6712_zh_p1_compare.png`
  - `mydocs/pr/assets/issue_6712_zh_p2_compare.png`
- 코멘트에는 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment),
  실제 페이지와 아래 지표/자동 후보 및 수동 판정의 한계를 함께 적는다. 최종 코드의 검증을
  완료한 뒤 PR review에 이 계획을 옮기며, merge 전 수용 완료 코멘트를 먼저 게시하지 않는다.

```markdown
![#6712 한국어 1쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6712_ko_p1_compare.png)
![#6712 한국어 2쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6712_ko_p2_compare.png)
![#6712 중국어 1쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6712_zh_p1_compare.png)
![#6712 중국어 2쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6712_zh_p2_compare.png)
```

Merge SHA로 고정한 URL의 자산 존재를 확인한 뒤 UTF-8 `--body-file`로 게시하고 API 재조회로
이미지 URL/한글/줄바꿈을 확인한다. #6708은 기존 표지 before/after 2장을 쓰고,
#6714는 signed offset 계약 결과를 명시한다. #6699는 부분 개선/잔여 dx=-1.41px를 적고 close하지 않는다.

| 문서 | 1쪽 pixel match | 2쪽 pixel match | 1쪽/2쪽 ink proxy |
| --- | --- | --- | --- |
| 한국어 | 84.51% | 81.95% | 14.79% / 48.95% |
| 중국어 | 83.12% | 77.89% | 12.09% / 41.24% |

96dpi/threshold 32. 자동 Square wrap 후보는 한국어·중국어 1쪽 각 1건이며 2쪽은 0건이다.
18단계처럼 TextLine의 가용 폭이 그림 경계에 닿는 후보로, 실제 비어 있지 않은 TextRun과
직접 비교한 결과 그림과 글리프의 교차는 아니다. 이 후보와 19단계 제목 문자 겹침은 별개다.
제목은 실제 ink 검사를 추가하여 재판정했다. 폰트 디자인·수 px 위치·중국어 테두리 길이 등의
잔여 차이가 있으므로 pixel-perfect 또는 자동 지표만으로 수용했다고 쓰지 않는다.
