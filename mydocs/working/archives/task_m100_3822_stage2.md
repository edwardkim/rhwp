# Task #3822 Stage 2 — 최신 stack 줄바꿈 검증

- 이슈: [#3822](https://github.com/edwardkim/rhwp/issues/3822)
- 브랜치: stack/issue-3822-overlong-token-wrap
- 최신 기준: upstream/devel cf5d462dc
- code candidate: ba99fad54
- 작성일: 2026-08-04

## 최신 기준 focused 결과

- cargo test renderer::composer::tests --lib: 53 / 53 통과
- #3822 전용 Latin·한글·숫자·잔여 폭·hanging indent: 5 / 5 통과
- git diff --check: 통과

최종 검증 뒤 devel이 aeb5805cb로 전진했다. composer 파일과 직접 겹치지 않았지만
typeset 쪽 경계 수정이 실제 문서의 page count에 영향을 줄 수 있어 composer 53 / 53과
#3822 전용 5건을 다시 실행해 모두 통과했다. 리뷰에서 지적된 한글 무공백 어절도 별도
회귀로 고정해 Latin·숫자와 같은 prior-break 반복 분할 계약을 확인했다.

## 기존 실제 문서 증적

2026-08-03 production WASM snapshot에서 다음을 확인했다.

- HWP/HWPX 두 번째 숫자 줄바꿈: 2 / 2
- line count 5 → 6, caret 665.4 / cell right 672.8, overflow false
- HWP/HWPX × digits, Latin, 완료 한글→digits 저장·재열기: 6 / 6
- 실제 IME→공백→두 번의 숫자 wrap: 2 / 2
- #3822 미적용 control은 숫자 79번째에서 cellOverflowed=true

문자 수와 실제 높이가 충분히 늘어 page count가 115 → 116이 되는 것은 숨은 overflow가
정상 line으로 복원된 결과다.

## 귀속 정정

기존 integration gate의 advance 상한은 TextRun origin과 bbox가 셀 안에 있는지를 검증했지만
glyph outline의 실제 가로 비율을 직접 증명하지 않았다. 따라서 다음처럼 분리한다.

- #3822: token 재분할과 overflow 해결
- #3937: Canvas/SVG 영문·숫자 glyph outline 확대 해결

최신 최상단 stack의 production WASM에서 같은 combined E2E를 재실행했다. HWP/HWPX 모두
숫자 줄 전환 11 / 69, 최종 숫자 73, 최종 쪽수 116과 synchronous flush 0으로 GREEN이었다.
두 정확성 수정과 #3815 scheduler의 조합이 최신 typeset 변경 뒤에도 유지됐다.

검토 CI 중 devel이 중첩 표 배치 수정 #3949를 포함한 cf5d462dc로 전진했다. composer 제품
코드는 충돌하지 않았고 53 / 53과 #3822 focused 5 / 5를 유지했다. 새 production WASM
HWP/HWPX 통합 E2E도 줄 전환 11 / 69, 숫자 73, 최종 116쪽으로 GREEN이며 p95는
49.6 / 49.7ms였다.

Draft PR은 [#3945](https://github.com/edwardkim/rhwp/pull/3945)이며 부모는
[#3944](https://github.com/edwardkim/rhwp/pull/3944), 자식은
[#3946](https://github.com/edwardkim/rhwp/pull/3946)이다.
