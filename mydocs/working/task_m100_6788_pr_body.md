<!-- PR 본문 초안. 아직 게시하지 않았다. 이미지 URL은 증적 commit을 upstream에 push한 후 유효해진다. -->

## 변경 요약

혼합 글자색에 형광펜을 적용할 때 기존 글자색이 소실되고, Undo에서도 돌아오지 않는 문제를 수정한다.

- 코어: 선택 시작의 모양 하나로 덮지 않고 기존 구간별로 지정 속성만 병합한다.
- Studio: 문단당 단일 ID 대신 적용 전후의 구간 경계·모양 ID 목록을 저장해 Undo/Redo한다.
- 본문·일반/중첩 셀을 지원하며 기존 F5·머리말/꼬리말 snapshot 경로를 유지한다.
- 복원 payload 전체를 mutation 전에 검증한다. 새 API가 없는 구버전 WASM은 서식 적용 전에
  명시적으로 실패하며, 손실되는 단일 ID 방식으로 fallback하지 않는다. JS/WASM은 함께 갱신해야 한다.
- renderer/layout 정책 변경은 없다.

## 관련 이슈

Closes #6788

## 의도된 동작 — Chrome 실제 UI 스크린샷

`가나다라마바사`에서 `다라`는 보라색·굵게, 선택 범위 `나다라마`만 노란 형광펜이다.
아래 네 행은 **수정 후 동일 문서**의 적용 전 → 형광펜 → Undo → Redo다.
모든 상태에서 보라색·굵기가 유지되고, Undo/Redo에서는 형광펜만 제거·복원된다.
선택 밖 `가`와 `바사`는 변하지 않는다.

![Chrome: 적용 전·형광펜·Undo·Redo](https://raw.githubusercontent.com/edwardkim/rhwp/c3b1398e4745f6d0030321df525e787d575f8ab3/mydocs/pr/assets/issue6788_chrome_behavior.png)

사용자가 CLI 저장본 `03-undo-cli.hwpx`를 재열어둔 로컬 Studio에서 실제 UI로 조작·캡처했다.
본문 영역을 crop해 배치하고 바깥 상태 라벨만 추가했다. Firefox 화면 또는 UI 저장 성공 증적은 아니다.
마지막에는 적용 전 내용으로 되돌렸고 파일을 덮어쓰지 않았다.

재현/확인 순서:

1. 문구 입력 후 `다라`에 보라색·굵기를 지정한다.
2. 서식 경계를 가로질러 `나다라마`를 선택해 노란 형광펜을 적용한다.
3. Undo·Redo를 반복하여 기존 글자색·굵기와 선택 밖 영역이 유지되는지 확인한다.
4. 적용 전·적용·Undo·Redo를 HWP/HWPX로 저장·재적재하여 대상 속성을 비교한다.

## 저장 후 재열기 — CLI 렌더링 증적

실제 Studio command/history와 Node WASM으로 4상태를 만들고, 엔진 export → CLI 재저장 →
재적재 → native-skia PNG export로 **HWP·HWPX 총 8개 파일**을 확인했다.

왼쪽 HWP, 오른쪽 HWPX; 위에서 적용 전·형광펜·Undo·Redo.
아래는 브라우저 스크린샷이 아닌 CLI 렌더링이다.

![HWP·HWPX 네 상태의 CLI 렌더링](https://raw.githubusercontent.com/edwardkim/rhwp/c3b1398e4745f6d0030321df525e787d575f8ab3/mydocs/pr/assets/issue6788_cli_roundtrip.png)

- CLI `--verify --verify-pages`: 8개 모두 IR diffCount 0, 1페이지 유지.
- 최종 파일 재적재: 7글자 전체의 textColor/shadeColor/bold/fontSize/fontFamily 기대값 일치.
- 1588×2245 전체 PNG 비교: 각 포맷 before=undo·highlight=redo, 각 상태 HWP=HWPX —
  총 8쌍 모두 **0픽셀 차이**. 브라우저 JPEG 캡처에는 이 수치를 적용하지 않았다.

## 테스트

로컬 검증 후보: `c3b1398e4745f6d0030321df525e787d575f8ab3`.
제품 source는 2단계 `ff716d797dedb60ee8db236b28428be88089af9f`이며, 이후 가드 분류 보완 후
필수 lint·build·manifest·전체 nextest를 다시 통과했다. 별도 review worktree에서 source 일치를
확인하고 파생 suite를 준비했다. 아래 수치는 CI가 아닌 로컬 실행 결과다.

- [x] `cargo fmt --all -- --check`.
- [x] native / WASM lib / workspace-all-targets Clippy `-D warnings` 3종, locked workspace build.
- [x] 새 integration 원본은 `tests/cases/`에만 추가. 파생 suite·manifest·Cargo target 미포함.
- [x] 전체 Rust nextest: **9071 passed, 0 failed, 46 skipped** (slow 2, leaky 1).
- [x] Native Skia lib: rhwp 3930 passed·13 ignored, workspace member 182 passed.
  missing-picture placeholder 2 passed, direct PDF 4 passed.
- [x] Studio 전체 **1427 passed**, binding/manifest 계약 **22 passed**; 실패·skip 0.
- [x] Rust focused 15 passed, Studio 관련 62 passed, 실제 WASM/command/history 13개 시나리오.
- [x] Studio TypeScript·Vite·PWA 및 Firefox 확장 build.
- [x] 합성 문서 SVG/PNG export와 Chrome 실제 WASM 편집·Undo/Redo 확인.

source-side `#[cfg(test)]` 및 capability skill/agent 변경은 없다. 영수증 캡슐은 생성하지 않았다.
편집 command의 구간 보존·사전 검증·실패 시 복구·history·binding 회귀는
[2단계 보고서](https://github.com/edwardkim/rhwp/blob/c3b1398e4745f6d0030321df525e787d575f8ab3/mydocs/working/task_m100_6788_stage2.md)에 기록했다.
상세 검증·초기 실패·재실행·화면 증적 범위는
[3단계 보고서](https://github.com/edwardkim/rhwp/blob/c3b1398e4745f6d0030321df525e787d575f8ab3/mydocs/working/task_m100_6788_stage3.md),
최종 요약은 [최종보고서](https://github.com/edwardkim/rhwp/blob/c3b1398e4745f6d0030321df525e787d575f8ab3/mydocs/report/task_m100_6788_report.md)를 참조한다.

## 성능 영향

기존 모양 구간 수에 비례한 목록 capture/restore 비용이 추가된다. 문자마다 JS↔WASM을 왕복하지
않으며, Undo/Redo 복원 호출은 문단 단위라는 회귀 계약을 확인했다. 정량 wall-clock 벤치마크는
미측정이다. 구간별 reflow 반복이나 renderer 정책 변경은 없다.

## 검증 한계 및 잔여 사항

- 사용자 승인에 따라 기존 확장을 교체하지 않고 로컬 Studio와 CLI 저장·재열기 경로로 분리했다.
  **Firefox 직접 UI, OS 저장 대화상자, 8개 파일의 브라우저 UI 전수 열기는 미검증**이다.
- Docker daemon 부재로 locked native `--no-opt` WASM을 사용했다. 최적화 배포 산출물 검증이 아니다.
  web·Node·Firefox 패키지 WASM SHA-256은
  `ed080899b874b47c50727f2beffcad4b584e52c7a6d44f709e9f6313e7e8e336`로 일치한다.
- nextest의 `issue_2007_single_cell_continuation_does_not_repaint_boundary_fragments`는
  성공했지만 LEAK 표시가 있었다. 종료 시 출력 핸들/자식 프로세스 잔류 원인은 확정하지 않았다.
- 최초 HWPX export의 fillType/patternColor/patternType 차이는 형광펜 적용 전부터 관찰했다.
  이슈 대상 속성 보존과 전체 속성 무손실을 구분하며, 이 별도 차이는 수정 범위에 포함하지 않았다.
- native 시스템 fallback과 브라우저 webfont의 모양은 다를 수 있다. 한컴 독립 정답지나
  광범위 visual fidelity 통과를 주장하지 않는다. Studio build에는 chunk 크기 경고가 있었다.

이미지는 증적 commit SHA에 고정했다. 최신 PR head의 GitHub Actions 및 merge 승인은 별도 게이트다.
