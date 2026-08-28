---
kind: pr-review
status: rework-requested
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6083 review - 셀 저장 2줄이 1줄로 접힌 유의사항 상자 재래핑 (#5952)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6083](https://github.com/edwardkim/rhwp/pull/6083) |
| 작성자 | [@kevin9327](https://github.com/kevin9327) |
| base | `devel` |
| 원 head | `cfb2646ee19ffb794fbb389779d56114c6f97807` |
| 규모 | +283 / -4, 6 files, 3 commits |
| GitHub 상태 | 2026-08-28 재확인: non-draft, `DIRTY`, 실패·진행 check 0건 |
| 원 PR CI | [run 32890821000](https://github.com/edwardkim/rhwp/actions/runs/32890821000/job/97947096558) |
| 판정 | **재작업 요청 — 통합에서 제외** |

## 관련 이슈와 변경 범위

[#5952](https://github.com/edwardkim/rhwp/issues/5952)는 `samples/2025 행정업무운영 편람(최종).hwp`
(SHA-256 `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`) 한글 61쪽
(rhwp `-p 68`) 유의사항 상자의 줄이 상자 오른쪽(~600px)을 넘어 사이드바 "공문서"(~671px)와
겹치는 결함이다.

`src/renderer/composer.rs`의 `recompose_stored_single_line_if_overflowing`이 저장 `ls==1` 과밀만
재래핑하던 것을, 인증 저장 `ls>=2`가 1줄로 접힌 경우까지 확장한다.

## 렌더 영향과 시각 검증

셀 조판·줄바꿈 경로가 바뀌고 특정 쪽의 겹침 해소를 주장하므로 **직접 증적 필수** 조합이다.
저장소의 한컴 기준 PDF `pdf/2025 행정업무운영 편람(최종)-2024.pdf`(383쪽, 대상 68쪽 0-based)를
기준으로 세 상태를 같은 기계에서 렌더해 비교했다. 원본은 `lastSavedWith.product`가
`hancom-office-2024`라 2024 계열 기준 PDF를 골랐다.

~~~bash
rhwp export-png "samples/2025 행정업무운영 편람(최종).hwp" -p 68 -o <out>
~~~

| 상태 | 결과 |
| --- | --- |
| 한컴 기준 PDF | 상자는 `※ 기안문 제목과…`로 시작해 상자 안에서 끝나고, 아래 `4) 문서의 "끝" 표시`와 분리된다 |
| `devel` `6b5c4f871` | 상자 구성은 기준과 같으나 `붙임파일에…`·`문서 보안 또는` 두 줄이 상자 오른쪽을 넘어 사이드바와 겹친다 (= #5952) |
| PR head `cfb2646ee` | 사이드바 겹침은 해소. 그러나 앞 문단(`정보 또는 문서를 출력…`)이 이 쪽으로 끌려오고 **상자 하단이 넘쳐 `4) 문서의 "끝" 표시` 본문 줄과 글자가 겹친다** |

증적 asset:

- `mydocs/pr/assets/pr_6083_handbook_p69_oracle_2024.png` (한컴 기준)
- `mydocs/pr/assets/pr_6083_handbook_p69_devel.png` (수정 전)
- `mydocs/pr/assets/pr_6083_handbook_p69_after.png` (PR 적용 후)

## 발견한 문제

### 1. 같은 쪽에서 보이는 결함을 다른 보이는 결함으로 바꾼다 (차단)

수정 후 상자 하단 넘침이 본문과 글자 겹침을 만든다. 한컴 기준 PDF에는 사이드바 겹침도, 하단
넘침도 없다. 이 현상은 **PR head 단독**으로 재현되므로 다른 PR과의 상호작용이 아니다. 원 PR CI가
통과한 이유는 이 겹침을 검사하는 시험이 없기 때문이다. 원 PR이 추가한
`tests/cases/issue_5952_cell_note_overflow.rs`는 상자 **오른쪽** 한계(`BOX_RIGHT_LIMIT = 640.0`)만
검사하고 상자 아래 방향은 보지 않는다.

### 2. overflow-cell 원장 증가에 근거가 없다

`tests/fixtures/overflow_cell_baseline.tsv`의 `2025 행정업무운영 편람(최종).hwpx` 행이
`51` → `52`로 늘었다.
[local_validation 4.3.1](../../manual/pr_review/local_validation.md#431-새-hwphwpx-fixture의-baseline-등록--ir-sweep--overflow-cell-원장)은
"기존 문서의 수치 증가는 렌더 회귀다 — baseline 으로 숨기지 않는다"로 못박고 있는데 PR 본문에
설명이 없다. 검토 환경(macOS)에서 같은 문서를 재산출하면 devel `51` → PR `46`이라 이 증가는
Linux 전용 관측이며, 그 사실 자체가 기록되지 않았다.

## 최종 권고

**재작업 요청.** 통합 체리픽에서 제외했다. 재작업 시 확인할 항목은 다음과 같다.

1. 재래핑으로 늘어난 줄이 상자 높이를 넘지 않도록 상자·행 높이 계상까지 함께 본다. 넘침이
   불가피하면 어느 쪽이 한컴 조판인지 기준 PDF로 먼저 확정한다.
2. 회귀 시험에 상자 **하단** 경계와 후속 본문 문단의 겹침 판정을 추가한다. 현재 시험은 오른쪽
   경계만 본다.
3. overflow-cell 원장을 바꿔야 한다면 증가분의 원인과 플랫폼(Linux/macOS) 관측 차이를 review와
   PR 본문에 남긴다.

#5952 재현과 원인 분석(`저장 horzsize=37560HU 2줄인데 화면은 1줄 maxx≈680`)은 정확했다.

## 2026-08-28 재확인

open PR 목록을 다시 확인했을 때 #6083은 draft가 아니고 실패 check는 없었지만, `mergeStateStatus=DIRTY`
상태였다. 또한 2026-08-26 메인터너 코멘트가 현 상태 통합 보류와 재작업 요청을 명확히 남겼다.
따라서 #6245/#6247/#6248/#6249/#6250/#6254/#6259 통합 검토에는 포함하지 않았다.
