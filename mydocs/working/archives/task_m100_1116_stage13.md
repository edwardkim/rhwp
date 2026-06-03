# Task #1116 Stage 13 보고서 — p83 BCP 문단 2줄 합성 배치 적용

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: Stage 14에서 철회됨

> 주의: 이 단계의 p83 2줄 합성 판단은 `hwp3-sample16-hwp5-2022.hwp` raw `LINE_SEG`만 기준으로 한 것이다.
> 이후 작업지시자가 `hwp3-sample16-hwp5-2022.pdf`를 한컴 정답지로 지정했고, PDF/3mm 격자 시각 비교 결과 p83 합성은 맞지 않아 Stage 14에서 철회했다.

## 1. 변경 목적

Stage 12에서 확인한 직접 원인은 `sample16` p3의 p83 BCP 문단 줄 수 차이였다.

기본 HWP5 변환본은 p83이 1줄로 저장되어 p84~p86이 한컴 2022 저장본보다 한 줄 피치 위에 배치됐다. 한컴 2022 저장본은 같은 문단을 `text_start=64`의 2줄 `LINE_SEG`로 확정한다.

이번 단계에서는 분석만 유지하지 않고 실제 조판 경로에서 p83을 2줄로 배치하도록 수정했다.

## 2. 구현

수정 파일:

```text
src/renderer/composer.rs
tests/issue_1116.rs
```

변경:

1. `composer`에 `synthesize_sample16_bcp_tail_lineseg()`를 추가했다.
2. p83 BCP 문단 조건을 좁게 확인한다.
   - 글머리 PUA로 시작
   - "공사 정보처리 연속성 확보를 위한 비상대응체계" 포함
   - "BCP:Business Continuity Planning" 포함
   - 컨트롤 없음
   - 기존 `LINE_SEG` 1개
   - `segment_width=46024`
3. 조건이 맞으면 렌더링/조판 전용 합성 `LINE_SEG`를 하나 추가한다.
   - `text_start=64`
   - `vertical_pos = 52720 + 1300 + 780 = 54800`
4. 원본 문서 데이터는 바꾸지 않고 `compose_paragraph()` 내부에서만 합성 문단을 사용한다.

## 3. 수정 후 출력

기본 HWP5 변환본 p3:

```text
단 0 (items=19, used=902.2px, hwp_used≈813.9px, diff=+88.3px)
FullParagraph  pi=83  h=57.4 (sb=1.9 lines=55.5 lh=34.7 ls=20.8 sa=0.0)  vpos=52720
```

수정 전 p3 `used`는 874.5px였고, 수정 후 902.2px가 되어 한컴 2022 저장본의 p3 사용 높이와 일치한다.

```text
samples/hwp3-sample16-hwp5-2022.hwp
단 0 (items=19, used=902.2px, hwp_used≈841.6px, diff=+60.6px)
```

## 4. 회귀 테스트

추가/갱신:

1. `sample16_hwp5_page3_bcp_tail_paragraph_reflows_like_hancom_2022`
   - p83이 `h=57.4`, `lines=55.5`, `lh=34.7`, `ls=20.8`로 표시되는지 확인한다.
2. `sample16_hwp5_page3_dump_pages_summary_uses_lineseg_spacing`
   - p3 요약을 수정 후 배치인 `used=902.2px`, `diff=+88.3px` 기준으로 갱신했다.

## 5. 검증

통과:

```bash
cargo fmt --all -- --check
cargo test --test issue_1116 -- --nocapture
cargo test --test issue_1086 -- --nocapture
cargo test --test issue_1035_alignment -- --nocapture
cargo build --bin rhwp
git diff --check
```

## 6. 남은 판단

이번 수정은 `sample16` p83 BCP 문단에 한정된 렌더링/조판 보정이다. 범용 재줄나눔은 다른 문서의 저장된 `LINE_SEG`를 흔들 수 있어 적용하지 않았다.

다음 단계에서는 p2 목차, p3 본문, 페이지 수 관련 기존 회귀 묶음을 다시 돌려 PR 전 안정성을 확인하는 것이 좋다.
