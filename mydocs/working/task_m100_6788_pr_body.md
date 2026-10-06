## 변경 요약

혼합 글자색에 형광펜을 적용하면 기존 글자색이 소실되고 Undo로도 복원되지 않는 문제를 수정한다.

- 기존 서식 구간마다 지정한 속성만 적용해 글자색·굵기 등 나머지 서식을 보존한다.
- Studio Undo/Redo는 적용 전후의 구간별 서식을 저장·복원한다.
- 적용·복원 오류를 구분해 안내하고, 복구가 덜 끝나면 Undo로 재시도할 수 있게 이력을 보존한다.
- 본문·일반/중첩 셀을 지원하며 renderer/layout 정책은 변경하지 않는다. JS와 WASM은 함께 갱신해야 한다.

Closes #6788

## 의도된 동작

Chrome·Firefox의 새 문서에서 `다라`를 보라색으로 지정하고, `나다라마`에 형광펜을 적용했다.
위에서 적용 전 → 형광펜 → Undo → Redo다. 글자색은 유지되고 형광펜만 제거·복원된다.

![Chrome·Firefox 형광펜 적용·Undo·Redo](https://raw.githubusercontent.com/edwardkim/rhwp/b296a4cddcf4ca5818fd476a660ba03f8f3dbe17/mydocs/pr/assets/issue6788_browser_behavior.png)

각 브라우저에서 저장한 HWP·HWPX를 다시 열어도 글자색과 형광펜이 유지된다.

![Chrome·Firefox HWP·HWPX 재열기](https://raw.githubusercontent.com/edwardkim/rhwp/b296a4cddcf4ca5818fd476a660ba03f8f3dbe17/mydocs/pr/assets/issue6788_browser_reopen.png)

두 이미지는 최초 구현 검증에서 로컬 Studio의 실제 UI 문서 영역을 잘라 배치한 스크린샷이다.

## 검증

- Chrome·Firefox: 전체/부분 형광펜 적용·Undo/Redo, 각 브라우저의 HWP·HWPX 저장 후 재열기 정상.
- 저장한 네 파일의 7글자 전체 색상·형광펜 보존 확인. 별도 CLI 8파일 왕복도 IR 차이 0.
- 리뷰 보정 후 fresh WASM으로 8파일 저장·재열기 및 PNG 8쌍 비교 정상(0픽셀 차이).
- Rust nextest **9,073 passed, 0 failed, 46 skipped**; Studio **1,428 passed**, binding 계약 **22 passed**.
- fmt·Clippy 3종·workspace build, Native Skia 회귀, Studio·Firefox 확장 build 통과.

검증에는 동일 소스의 `--no-opt` WASM을 사용했다. 상세 결과는
[리뷰 보정 검증 기록](https://github.com/edwardkim/rhwp/blob/codex/6788-preserve-mixed-char-format/mydocs/pr/archives/pr_6814_review_impl.md)에 기록했다.
