---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-29
---

# Task #3486 Stage 6 — HWP3 음수 들여쓰기의 첫 줄 기준 정규화

- 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486)
- 브랜치: `task_m100_3486_hwp3_render_fidelity_v2`
- 선행: `aeca28c18` (Stage 5: HWP3 inline 표 IR 앵커 보정)
- 기준 오라클: `pdf/HWP3-password-123456.pdf` 3쪽

## Stage 5의 잔여가 표 앵커가 아닌 이유

Stage 5에서 문단 30의 1×4 inline 표 `ref_pos=0`을 `Para/Para`로 바꿔 IR 기본값
`Paper/Paper` 잔존을 제거했다. fixture 회귀도 통과했다. 그러나 표 다음 설명 문단의
`\\HNC` run은 x=269.8px에 그대로였다. 대상 run은 표 셀이 아니라 문단 32~34의 일반 본문이므로,
표 `CommonObjAttr`는 이 수평 위치를 소비하지 않는다.

한컴 PDF 3쪽을 raster로 다시 대조한 결과 첫 `\\HNC`의 화면 x는 약 160px이며, HWP5 변환본의
render tree x=160.1px와 일치한다. HWP3 원본의 현재 x=269.8px는 수용할 수 없다.

## 원시 HWP3 문단 모양과 공통 IR의 음수 들여쓰기 의미

대상 문단 32~34의 HWP3 문단 모양은 다음과 같다.

| 항목 | HWP3 raw | 현재 HWP3 IR | HWP5 변환본 IR | 의미 |
| --- | ---: | ---: | ---: | --- |
| `left_margin` | 2932 hunit | 23456 HU | 7000 HU | HWP3 raw는 후속 줄 기준 |
| `indent` | -2057 hunit | -16456 HU | -16456 HU | 내어쓰기 폭 |
| 첫 줄 기준 | `2932 + (-2057) = 875` hunit | 23456 HU로 오해 | 7000 HU | `875 × 8 = 7000` HU |
| `\\HNC` x | — | 269.8px | 160.1px | PDF는 HWP5 계열과 일치 |

HWP3 사양의 `left_margin`/`indent`는 hunit(1/1800 inch)이다. HWP3 음수 들여쓰기에서
`left_margin`은 후속 줄, 첫 줄은 `left_margin + indent`로 해석된다. 반면 공통 renderer는
음수 `indent`일 때 `ParaShape.margin_left`를 첫 줄, `margin_left + |indent|`를 후속 줄의
시작점으로 쓴다. 현재 parser가 raw `left_margin`을 그대로 `×8`하면 이 의미가 뒤집혀 첫 줄이
41.4mm 오른쪽으로 밀린다.

한컴 HWP3→HWP5 변환본은 이미 이 표현 변환을 수행했다. `left_margin=7000`과
`indent=-16456`은 각각 `max(0, 2932 + -2057) × 8`, `-2057 × 8`이다. 따라서 이 보정은 PDF를
향한 문서별 좌표 상수가 아니라 legacy HWP3 음수 들여쓰기와 공통 IR의 의미 차이를 정규화하는 것이다.

## 구현 계약

`convert_para_shape()`에서만 HWP3 `indent < 0`일 때 `margin_left`를 raw 첫 줄 기준
`max(0, left_margin + indent)`으로 변환한다. `indent` 자체, 양수 들여쓰기, 오른쪽 여백, 탭,
HWP5/HWPX parser는 바꾸지 않는다.

회귀는 두 층으로 둔다.

1. 단위 변환: `(left_margin=2932, indent=-2057)`이 `(margin_left=7000,
   indent=-16456)`이 되는지 확인한다.
2. 실제 암호 HWP3 fixture: 문단 32의 ParaShape가 위 값을 갖고, 새 3쪽 render tree의
   첫 `\\HNC` run이 HWP5 비교값 160.1px 계열로 복귀하는지 확인한다.

이 문서는 코드 변경 전 분석 기록이며, 아래 계약의 parser 보정·focused test·시각 대조 결과와 같은
커밋으로 보존한다.

## 구현·검증 결과

`convert_para_shape()`는 음수 `indent`의 HWP3 `margin_left`를 raw 첫 줄 기준으로 정규화한다.
실제 fixture 문단 32의 IR은 `margin_left=7000`, `indent=-16456`이 됐고, parser 단위 회귀와
`hwp3_password_fixture` 5건은 모두 통과했다.

격리 target에서 새 CLI로 3쪽 SVG/render tree를 다시 만들었다.

| 항목 | 수정 전 | 수정 후 | 기준 |
| --- | ---: | ---: | --- |
| HWP3 문단 32 첫 `\\HNC` x | 269.8px | 160.1px | HWP5 render tree 160.1px, 한컴 PDF raster와 같은 첫 줄 기준 |
| HWP3 문단 32 첫 `\\HNC` y | 778.5px | 778.5px | 이번 단계는 수평 들여쓰기만 변경 |

시각 확인 PNG는 `tmp/pdfs/task3486/stage6-hwp3-p3.gii8s1/HWP3-password-123456_003.png`에
생성했다. 1×4 폴더 표 뒤 설명 문단의 수평 위치는 복구됐지만, 제목 조합 자모·표 셀의 legacy
글꼴/자간 차이는 여전히 남아 있다. 이들은 문단 들여쓰기와 독립된 다음 Stage의 분석 대상으로
분리한다.
