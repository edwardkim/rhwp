---
kind: investigation
status: active
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-29
---

# Task #3486 Stage 5 — HWP3 글자처럼 취급되는 표의 기준 좌표 분석

- 이슈: [#3486](https://github.com/edwardkim/rhwp/issues/3486)
- 브랜치: `task_m100_3486_hwp3_render_fidelity_v2`
- 선행 커밋: `f8996a76b` (Stage 4 분석), `24a3f7429` (HWP3 표 셀 색상·음영 보존)
- 비교 입력: `samples/HWP3-password-123456.hwp` ↔
  `samples/HWP5-nopassword-123456.hwp`

## Stage 4 후 시각 확인

HWP3 3쪽의 4×2 시스템 사양 표는 색상 word=`0x0007`, 음영=`100`을 흰색으로 합성한 뒤 검정 우측
셀과 본문 가림이 사라졌다. 이 결과는 암호 해독과 무관한 HWP3 표 채움 결함을 하나 제거한 것이다.

그러나 HWP3은 같은 쪽의 1×4 폴더 구성 표와 그 아래 설명 영역이 HWP5 변환본보다 왼쪽 기준에서 크게
밀리고, Studio에서 보이는 제목 조합 자모·글리프 문제도 남아 있다. 따라서 Stage 4의 수정으로 HWP3
정합을 완료했다고 판단하지 않는다.

## 원시 표 앵커와 공통 IR 비교

대상은 구역 0, 문단 30의 1행×4열 폴더 구성 표다. HWP3 원시 표 정보 offset 8은 `0`이고, HWP3
형식에서 이는 글자처럼 취급되는(text/treat-as-char) 기준이다.

| 항목 | HWP3 현재 parser | HWP5 변환본 parser | 영향 |
| --- | --- | --- | --- |
| `treat_as_char` | `true` | `true` | 두 입력 모두 inline 표임 |
| `vert_rel_to` | `Paper` (기본값 잔존) | `Para` | HWP3만 종이 좌표 기준으로 layout에 들어감 |
| `horz_rel_to` | `Paper` (기본값 잔존) | `Column` | HWP3 후속 설명의 시작 x가 오른쪽으로 밀림 |
| 표 하단 뒤 `\\HNC` 설명 run | x=269.8px | x=160.1px | 109.7px 수평 차이 |
| 표 상단 | y=514.7px | y=520.0px | 주된 차이는 수직이 아니라 앵커 축 |

현재 `src/parser/hwp3/mod.rs`의 표 처리에서 `ref_pos == 0`이면 `treat_as_char=true`만 설정하고
`horz_rel_to`/`vert_rel_to`를 지정하지 않는다. `CommonObjAttr::default()`의 Paper 기준이 그대로 남는다.
같은 파일의 HWP3 그림 처리에는 이미 `ref_pos == 0 → Para/Para` 보정이 있으나 표에는 없다.

HWP3 3쪽 render tree와 HWP5 변환본 tree를 다음 명령으로 재현했다. 비밀번호는 표준 입력으로만 전달했다.

```bash
target/task_3486_render_v2/release-test/rhwp export-render-tree \
  samples/HWP5-nopassword-123456.hwp -o <out>/hwp5 -p 2

printf '%s\n' "$PASSWORD" | \
  target/task_3486_render_v2/release-test/rhwp --password-stdin export-render-tree \
  samples/HWP3-password-123456.hwp -o <out>/hwp3 -p 2
```

## 구현 경계와 회귀 계약

다음 변경은 HWP3 표에서만 `ref_pos=0`을 명시적으로 문단 기준으로 바꾼다. HWP5/HWPX parser나
일반 부동 표(`ref_pos=1..=3`)는 변경하지 않는다.

1. HWP3 `ref_pos=0` 표는 `treat_as_char=true`, `HorzRelTo::Para`, `VertRelTo::Para`가 된다.
2. 실제 암호 HWP3 fixture의 문단 30 폴더 표가 위 세 속성을 가진다는 focused 회귀를 추가한다.
3. HWP3 3쪽의 표 뒤 `\\HNC` 설명 run의 x가 HWP5 비교 입력의 160.1px 계열로 되돌아가는지 render tree로
   확인한다. 표 내부 padding·font glyph 차이는 이 계약과 분리한다.
4. HWP3 제목 `ᄒᆞᆫ`의 Studio Canvas 표시 문제는 이 좌표 보정의 부작용으로 취급하지 않는다. HWP5/HWPX도
   같은 조합 자모를 갖는지, Canvas font fallback과 paint 경로를 별도 Stage에서 분석한 뒤에만 수정한다.

이 문서는 코드 변경 전에 작성한 Stage 5 분석 기록이다. 다음 구현·focused test·수정 후 시각 대조는
이 문서의 계약에 한정한다.

## 구현·검증 결과

`src/parser/hwp3/mod.rs`에서 HWP3 표의 `ref_pos=0`을 `Para/Para`로 명시했고, 실제 암호 HWP3
fixture 문단 30의 1×4 표를 검사하는 focused 회귀를 추가했다. 이 회귀는 통과했다. 따라서 IR에서
inline 표가 기본 `Paper/Paper`로 남는 상태는 제거됐다.

그러나 수정 후 다시 만든 3쪽 SVG/render tree에서 표 뒤 `\\HNC` run은 여전히 x=269.8px였다.
HWP5 비교 입력의 x=160.1px 계열로 이동한다는 시각 계약은 이 단계에서 충족하지 못했다. 즉, 이번
수정은 parser IR 정합에는 유효하지만 renderer가 inline 표의 기준 좌표를 실제 배치에 반영하지 않는
별도 문제를 해결하지 않는다. 이 미해결 경로는 다음 Stage에서 새 분석 문서와 함께 다룬다.
