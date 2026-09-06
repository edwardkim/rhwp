---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage18_pr_readiness.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 18단계: #6712 종료 범위와 PR 준비

Issue: #6712. 코드 후보: `ad990fd3f`.

## 경로와 범위

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `rework_and_exceptions.md` (큰 누적 diff)
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본/보조 문서,
  `verification/visual_sweep_guide.md`, `codex/docs_and_git_workflow.md`.
- 사용자의 최신 지시는 #6712까지 완료한 뒤 PR 준비다. 추가 열린 이슈 구현은 중지한다.
- 현재 후보에는 #6714/#6699/#6708의 앞선 보정이 포함된다. 개별 완료 여부와 PR 포함 여부를
  구분한다. 특히 #6699의 dx<1px 기준은 최종 측정으로 확인하기 전까지 Closes로 쓰지 않는다.
- 번호를 예측한 PR review 파일이나 오늘할일을 만들지 않는다. PR 생성 승인을 받은 뒤
  같은 PR의 trailing commit으로 작성한다. 현재 단계에서 push/PR/merge/close하지 않는다.

## 최종 게이트

1. 파생 suite 준비와 포맷 확인.
2. 최종 코드의 전체 nextest (release-test, target/pr-review, 12 threads, no-fail-fast).
3. native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy, manifest check.
4. visual sweep 도구의 Node/Python/실제 Chrome viewport 계약.
5. 두 #6712 원본의 기존 Hancom PDF와 새 sweep 1/2쪽을 직접 확인한다.
   수정 전 SVG는 `1f861362a`/binary SHA가 기록된 초기 산출물을 현재 래스터 도구로 다시 그린다.
6. 코멘트에 실제 사용할 최종 비교 PNG만 commit한다. 로그와 중간 SVG/JSON은 제외한다.

## 결과

**PR 준비 보류.** 전체 nextest는 9,089 passed (5 slow), 46 skipped, 514.962초,
exit 0으로 완료했다(빌드 포함 811.009초). Native Clippy(93.185초), WASM Clippy(71.214초)
도 exit 0이다. 나머지 workspace 게이트는 실행 중이며 다음 단계에서 결과를 인계한다.
실제 Chrome viewport 계약 1 test(1배/2배) 2.580초, Python visual sweep 46 tests
3.701초 모두 OK다. source-side inventory 4,205 tests / 298 modules check 및 diff check도 통과했다.

사용자가 한국어 1쪽 제목 '여름철 영유아 감염병 예방'의 글자 겹침을 지적했다.
아래의 '폰트 차이' 분류는 불충분했으며 수용 근거에서 철회한다. 이슈 전체 완료가 아니다.
SVG의 '여' x=255.8667, '름' x=275.6933으로 advance=19.8267px다. 실제 Chrome CDP는
Pretendard-Regular(custom webfont)를 사용하며 '여' bbox width=29.9534px를 보고했다.
원본 휴먼굵은팸체 메트릭과 fallback paint 폭 불일치다. 전체 회귀 통과만으로 검출되지 않았다.
19단계에서 제목 fallback glyph 폭 보정을 별도로 분석·수정·검증한다.

이번 단계에서는 분석 문서만 고정한다. 아래 PNG는 **거절된 후보의 로컬 식별 정보**이며
최종 증적으로 커밋하지 않는다. 다음 단계 재산출 후 경로/해시/코멘트 계획을 갱신한다.

## 18단계 시각 후보와 판정 정정

- 비교 원본: `samples/issue6712/한국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방.hwp`,
  `samples/issue6712/중국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방.hwp`.
- `rhwp info --json`: 한국어 `hancom-office-2020 / 11.0.0.9136`,
  중국어 `hancom-office-2024 / 13.0.0.3379`.
- 기준은 `pdf/issue6712/`의 같은 이름 `-2020.pdf`와 `-2024.pdf`다. 기존 PDF를 그대로
  재사용했다. 두 원본 및 PDF는 [fixture README](../../../samples/issue6712/README.md)에 연결되어 있다.
- HWP SHA-256: 한국어 `70a6663e75fefedc001b2c249bd20f5b994596954120740252f484b1892e2097`,
  중국어 `34a5964fa791ae662052cba8482efae682f13b45a0c0cd6ce633425566a2d5a9`.
- PDF SHA-256: 한국어 `ff0ab5e0cc70c4104d9dae960be01f736c0bf7a6d9a394212efb860dda1bdfd8`,
  중국어 `aa85b871b5d8049af5bd8240fa210678a48ebe4b258fe6497a528de234cb6f97`.
- 수정 전 코드 `1f861362ab372f9fa26e38f9a534a89286f641c4`, CLI SHA-256
  `911f17e119cf40a73f58d28e1109fe238201ba163c312346c6952d0d4d71de8c`.
  당시 3쪽 SVG와 provenance를 재사용하되 과거의 viewport 잘림 PNG는 재사용하지 않고
  수정된 래스터 도구로 다시 그렸다.
- 수정 후 코드 `ad990fd3fafe3d484dd8bb5c3e07a69a059460c6`, CLI SHA-256
  `2d245e71f465742b2bcef1702c5e13f5b28ab03278b5f78f4cbaebd9f48c5fe9`.
  바이너리는 `target/pr-review/release-test/rhwp`다.
- 재현: `VISUAL_SWEEP_CHROME=<Chromium 경로> venv/bin/python scripts/visual_sweep.py
  --hwp <위 원본> --pdf <기존 PDF> --pages 1,2 --rhwp-bin target/pr-review/release-test/rhwp
  --key <6712-ko 또는 6712-zh> --out /tmp/rhwp-stage18-sweep`.
  Chrome은 `/home/tsjang/.cache/ms-playwright/chromium-1187/chrome-linux/chrome`을 사용했다.
  `/tmp/rhwp-stage18-audit`는 좌표 진단, `/tmp/rhwp-stage18-final-assets`는 최종 패널 조립 경로다.

| 문서 | PDF / 수정 전 / 수정 후 쪽수 | 1쪽 pixel match | 2쪽 pixel match | 1쪽/2쪽 ink proxy |
| --- | --- | --- | --- | --- |
| 한국어 | 2 / 3 / 2 | 84.48% | 81.95% | 14.93% / 48.95% |
| 중국어 | 2 / 3 / 2 | 83.12% | 77.89% | 12.09% / 41.24% |

96dpi, pixel threshold 32 기준이며 지표는 자동 수용 기준이 아니다. PDF/수정 전/수정 후
세 열을 나란히 놓은 2414x1175 패널 4장의 제목·쪽번호·본문·하단을 직접 열어 확인했다.
1쪽의 예방수칙이 남고, 2쪽은 농가진 제목부터 시작하며 중첩 그림 표, 안내 상자, footer가
한 번씩 보인다. 중국어 마지막 footer를 가르던 테두리도 분리됐다.

자동 `square_wrap_text_overlap` 후보는 한국어·중국어 1쪽 각각 1건, 2쪽 0건이다.
TextLine의 가용 폭이 그림 경계에 닿는 `edge_clearance_loss`이며, 실제 비어 있지 않은
TextRun 및 패널을 확인한 결과 글리프가 그림과 겹치는 사례는 아니었다. 후보가 없었다고 쓰지 않는다.

한계: 한국어 제목은 단순 폰트 차이가 아닌 **문자 겹침 결함으로 보류**한다.
그림의 수 px 좌우 위치, 중국어 본문 세부 세로 위치 및 테두리 길이는
PDF와 완전히 같지 않다. 한국어 2쪽 중첩 그림 dy는 -0.05~-0.07px지만 dx는 +3.20~+3.35px다.
중국어 대응 텍스트 dy 중앙값은 1쪽 -4.3px, 2쪽 -9.4px다. #6712의 어울림/중복 높이/
쪽 밀림/말미 소실 개선은 확인했지만, 제목 겹침이 해결되기 전 전체 #6712 수용을 선언하지 않는다.

## 거절된 후보 PNG SHA-256 (로컬 전용)

| 파일 (`mydocs/pr/assets/`) | SHA-256 |
| --- | --- |
| issue_6712_ko_p1_compare.png | c524d5e45f9ca6465556d935918eec0ed88b2425278321f1bd9c5c7101df7409 |
| issue_6712_ko_p2_compare.png | 7ce21539db8b5c0103097ee95d4d70759a542fda316f0636f3f2fee60537fea7 |
| issue_6712_zh_p1_compare.png | d011cf86fbb9a75a3ddf9b70b2693ffff9ff6afa63d306a9d0d922d8a75cb41b |
| issue_6712_zh_p2_compare.png | 0f479e6fdb86af8b7bd6150f935d82b999319bd8c0ba949749d9ed1feff721df |

## Merge 후 issue comment 계획

PR 생성 전 기록이므로 번호는 비워 둔다. 생성 후 같은 PR review 문서로 아래 계획과
실제 게이트 결과를 옮기고, merge 전에는 게시하지 않는다.

- #6712: 사용자가 확인한 한국어·중국어 2개 HWP의 1/2쪽을 모두 비교했다.
  기존 PDF를 재사용하고, 3쪽으로 밀리던 본문이 2쪽에 배치되는지, 그림 옆 제목/본문,
  중첩 표 뒤 공백, 마지막 footer와 테두리 분리를 확인한다. 세 번째 내부 ID를 별도 검증했다고
  주장하지 않는다. 아래 네 패널을 모두 표시한다.
  - `mydocs/pr/assets/issue_6712_ko_p1_compare.png`
  - `mydocs/pr/assets/issue_6712_ko_p2_compare.png`
  - `mydocs/pr/assets/issue_6712_zh_p1_compare.png`
  - `mydocs/pr/assets/issue_6712_zh_p2_compare.png`
- #6708: 기존 `mydocs/pr/assets/issue_6708_cover_before.png` 및
  `issue_6708_cover_after.png`로 표지 그림 위치의 전후를 보여 준다.
- #6714: signed offset 수정과 음수/0/양수 계약 결과를 명시한다. 미확보 내부 원본 8개를
  모두 시각 검증한 것으로 쓰지 않는다.
- #6699: 부분 개선이며 **Ref만 사용**한다. 큰 그림 dx=-1.41px로 이슈의 dx<1px 기준은
  충족하지 않는다. 현재 범위에서 추가 구현/close하지 않는다. 수치와 한계를 기록하며
  완료 증거로 잘못 해석될 비교 이미지를 추가하지 않는다.
- 문서 비교 방법은 [Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)을
  GitHub `blob/devel` 직접 링크로 인용한다. 각 PNG는 다음 형식으로 실제 표시한다.

```markdown
![#6712 한국어 1쪽 비교](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_6712_ko_p1_compare.png)
```

같은 형식으로 다른 3개 패널 및 #6708 두 이미지를 연결한다. `<merge-commit-sha>`는 실제
merge SHA로 교체하고 자산 존재를 확인한다. UTF-8 파일을 `--body-file`로 게시한 다음 API로
본문, 이미지 링크, 줄바꿈을 재조회한다. 게이트 미완료 상태에서는 수용/완료 코멘트를 게시하지 않는다.
오늘할일, 번호가 붙은 PR review, 외부 코멘트는 이번 준비 단계에 포함하지 않는다.
