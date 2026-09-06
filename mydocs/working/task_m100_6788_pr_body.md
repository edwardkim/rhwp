<!-- PR 본문 초안. 아직 게시하지 않았다. SHA 고정 이미지·보고서 링크는 push 후 유효하다. -->

## 변경 요약

혼합 글자색에 형광펜을 적용하면 기존 글자색이 소실되고 Undo로도 복원되지 않는 문제를 수정한다.

- 기존 서식 구간마다 지정한 속성만 적용해 글자색·굵기 등 나머지 서식을 보존한다.
- Studio Undo/Redo는 적용 전후의 구간별 서식을 저장·복원한다.
- 본문·일반/중첩 셀을 지원하며 renderer/layout 정책은 변경하지 않는다. JS와 WASM은 함께 갱신해야 한다.

Closes #6788

## 의도된 동작

Chrome·Firefox의 새 문서에서 `다라`를 보라색으로 지정하고, `나다라마`에 형광펜을 적용했다.
위에서 적용 전 → 형광펜 → Undo → Redo다. 글자색은 유지되고 형광펜만 제거·복원된다.

![Chrome·Firefox 형광펜 적용·Undo·Redo](https://raw.githubusercontent.com/edwardkim/rhwp/b296a4cddcf4ca5818fd476a660ba03f8f3dbe17/mydocs/pr/assets/issue6788_browser_behavior.png)

각 브라우저에서 저장한 HWP·HWPX를 다시 열어도 글자색과 형광펜이 유지된다.

![Chrome·Firefox HWP·HWPX 재열기](https://raw.githubusercontent.com/edwardkim/rhwp/b296a4cddcf4ca5818fd476a660ba03f8f3dbe17/mydocs/pr/assets/issue6788_browser_reopen.png)

두 이미지는 로컬 Studio의 실제 UI 문서 영역을 잘라 배치한 스크린샷이다.

## 검증

- Chrome·Firefox: 전체/부분 형광펜 적용·Undo/Redo, 각 브라우저의 HWP·HWPX 저장 후 재열기 정상.
- 저장한 네 파일의 7글자 전체 색상·형광펜 보존 확인. 별도 CLI 8파일 왕복도 IR 차이 0.
- Rust nextest **9,071 passed, 0 failed, 46 skipped**; Studio **1,427 passed**, binding 계약 **22 passed**.
- fmt·Clippy 3종·workspace build, Native Skia 회귀, Studio·Firefox 확장 build 통과.

검증에는 동일 소스의 `--no-opt` WASM을 사용했다. nextest의 LEAK 표시 1건 등 상세 결과는
[검증 보고서](https://github.com/edwardkim/rhwp/blob/b296a4cddcf4ca5818fd476a660ba03f8f3dbe17/mydocs/working/task_m100_6788_stage3.md)에 기록했다.
