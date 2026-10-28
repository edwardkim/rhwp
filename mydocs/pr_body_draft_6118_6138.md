## 변경 요약

Studio 상단의 기본 도구 상자와 서식 도구 모음을 하나의 반응형 chrome 정책으로 정리합니다.

- 기본 도구 상자는 모든 지원 너비에서 label과 56px 한 줄 높이를 유지합니다.
- 넘치는 기본 도구는 기존 명령 DOM을 복제하지 않고 divider 단위로 좌우 이동합니다.
- 시작·끝 이동 버튼은 toolbar와 같은 배경 위에서 자연스럽게 사라지며, 끝점 이동과 240ms 퇴장
  애니메이션이 같은 frame에 시작해 함께 끝납니다.
- 서식 도구 모음은 962/961px, 808/807px, 460/459px의 콘텐츠 경계에서 전체 1행, paragraph 더보기
  1행, inline 2행, paragraph 더보기 2행으로 전환합니다.
- 글꼴 이름은 136px로 고정하고 최소 폭에서는 다른 field가 먼저 줄어듭니다.
- 기본 도구 이름표는 유지하고 서식 field 이름표는 감춰 편집 영역을 확보합니다.
- 기존 ID, command wiring, active/disabled authority와 #6115 도구 상자 표시 설정을 그대로 사용합니다.

## 관련 이슈

Closes #6118
Closes #6138

상세 설계와 검증 근거:

- [#6118 최종 보고서](https://github.com/edwardkim/rhwp/blob/b6e734d99a1d87ddd3626a9c88eaf7952961df29/mydocs/report/task_m100_6118_report.md)
- [#6138 최종 보고서](https://github.com/edwardkim/rhwp/blob/b6e734d99a1d87ddd3626a9c88eaf7952961df29/mydocs/report/task_m100_6138_report.md)

## 테스트

- [x] `npx tsc --noEmit`
- [x] focused Studio 계약: 35 passed, 0 failed
- [x] `npm --prefix rhwp-studio test`: 1,181 passed, 0 failed, 1 skipped
- [x] `npm --prefix rhwp-studio run build`
- [x] `npm --prefix rhwp-studio run e2e:manifest-check`: 116 tracked, 116 manifest
- [x] 실제 Chrome responsive/theme E2E: 821 passed, 0 failed
- [x] 14개 viewport와 default/flat/oldschool × light/dark 24개 테마 조합
- [x] `git diff --check`
- [x] 별도 review checkout의 `cargo fmt --all`과 `cargo fmt --all -- --check`
- [x] 새 integration test와 `src/**` Rust test 변경 없음

현재 source checkout에는 정책상 생성하지 않는 `tests/generated/regression_suite_001.rs`~`032.rs`가 없어
`cargo fmt --all`이 target 경로 확인 단계에서 중단됩니다. Rust source 변경은 없고, 파생 suite를 준비한
별도 review checkout에서 동일 format 게이트를 통과했습니다. 파생 suite와 manifest는 PR에 포함하지
않습니다.

## 성능 영향 및 측정 결과

- 예상 영향: 문서 renderer·WASM 성능 영향 없음
- 측정: 기본 도구 상자의 scroll은 240ms 동안 requestAnimationFrame 한 개만 사용하고 완료·취소 시 정리함

## 스크린샷

### #6118 서식 도구 모음

| 992px 전체 1행 | 460px 2행 inline | 375px 2행 더보기 |
| --- | --- | --- |
| ![992px 전체 1행](https://raw.githubusercontent.com/edwardkim/rhwp/b6e734d99a1d87ddd3626a9c88eaf7952961df29/mydocs/report/assets/task_m100_6118/stylebar-full-992.png) | ![460px 2행 inline](https://raw.githubusercontent.com/edwardkim/rhwp/b6e734d99a1d87ddd3626a9c88eaf7952961df29/mydocs/report/assets/task_m100_6118/stylebar-inline-460.png) | ![375px 2행 더보기](https://raw.githubusercontent.com/edwardkim/rhwp/b6e734d99a1d87ddd3626a9c88eaf7952961df29/mydocs/report/assets/task_m100_6118/stylebar-overflow-375.png) |

[#6118 상세 시각 결과](https://github.com/edwardkim/rhwp/blob/b6e734d99a1d87ddd3626a9c88eaf7952961df29/mydocs/report/task_m100_6118_report.md#3-시각-결과)

### #6138 기본 도구 상자

| 1280px 전체 표시 | 1024px group scroll | 375px group scroll |
| --- | --- | --- |
| ![1280px 전체 표시](https://raw.githubusercontent.com/edwardkim/rhwp/b6e734d99a1d87ddd3626a9c88eaf7952961df29/mydocs/report/assets/task_m100_6138/toolbar-wide-1280.png) | ![1024px 한 줄 스크롤](https://raw.githubusercontent.com/edwardkim/rhwp/b6e734d99a1d87ddd3626a9c88eaf7952961df29/mydocs/report/assets/task_m100_6138/toolbar-scroll-1024.png) | ![375px 한 줄 스크롤](https://raw.githubusercontent.com/edwardkim/rhwp/b6e734d99a1d87ddd3626a9c88eaf7952961df29/mydocs/report/assets/task_m100_6138/toolbar-scroll-375.png) |

[#6138 상세 시각 결과](https://github.com/edwardkim/rhwp/blob/b6e734d99a1d87ddd3626a9c88eaf7952961df29/mydocs/report/task_m100_6138_report.md#3-시각-결과)

Studio chrome의 DOM/CSS/접근성 변경이며 renderer 출력은 바꾸지 않아 PDF/SVG visual sweep은 적용하지
않았습니다.
