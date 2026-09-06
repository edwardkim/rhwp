# Task M100 #6788 — 혼합 글자 서식 보존 최종보고서

- Issue: [#6788](https://github.com/edwardkim/rhwp/issues/6788)
- 작성일: 2026-09-06
- 상태: 로컬 구현·승인된 범위의 검증 완료, remote push·PR 생성 승인 대기.
- 계획: [수행계획](../plans/task_m100_6788.md), [구현계획](../plans/task_m100_6788_impl.md).
- 단계 증빙: [1단계](../working/task_m100_6788_stage1.md),
  [2단계](../working/task_m100_6788_stage2.md), [3단계](../working/task_m100_6788_stage3.md).
- 기준 devel: `51ad998e33ef7f5191b0e1b0b656dc44cef33a1c`.
- 제품 source: `ff716d797dedb60ee8db236b28428be88089af9f`.
  이후 Rust 변경은 순수 조회 API의 passthrough 가드 분류 1건이며 변경 후 필수 검증을 재실행했다.

## 결과와 원인

혼합 글자색 선택에 형광펜을 적용해도 원래 색·굵기 등 미지정 속성이 유지되고,
Undo/Redo는 형광펜 적용 전후의 구간별 모양을 복원한다. 선택 밖 글자는 바뀌지 않는다.

코어는 선택 시작의 모양 ID 하나로 범위 전체를 덮었고, Studio history도 문단당 ID 하나만
저장했다. 두 계층을 각각 수정했다. 코어는 기존 구간별로 지정 속성만 병합하고,
Studio는 구간 경계·ID 목록을 capture/restore한다. 본문·일반/중첩 셀과 기존 F5·머리말/꼬리말
경로를 검증했다. renderer/layout 정책은 바꾸지 않았다.

## 의도된 동작 — Chrome 실제 화면

사용자가 재열어둔 `03-undo-cli.hwpx`에서 `다라`는 보라색·굵게, `나다라마`만 노란 형광펜이다.
위에서 적용 전 → 형광펜 → Undo → Redo. 실제 화면 본문 crop에 상태 라벨만 추가했다.

![Chrome 형광펜 적용·Undo·Redo](../pr/assets/issue6788_chrome_behavior.png)

## 검증 요약

| 검증 | 결과 |
| --- | --- |
| fmt·native/WASM/workspace-all-targets Clippy·workspace build·manifest | 모두 통과. |
| Rust 전체 nextest | 9071 passed, 0 failed, 46 skipped. |
| Native Skia lib / placeholder / direct PDF | 3930 + workspace 182 / 2 / 4 passed; lib 13 ignored. |
| Studio 전체 / binding 계약 | 1427 / 22 passed, 실패·skip 0. |
| 실제 WASM+Studio history focused | 13개 시나리오 통과. Rust focused 15개, Studio 관련 62개 통과. |
| Studio·Firefox 확장 빌드 | 통과. web/Node/Firefox WASM 동일 해시. |
| Chrome 직접 UI | 전체·부분 적용, Undo/Redo; 사용자 재열기 HWPX 문서 확인. |
| HWP/HWPX 4상태 × 2포맷 | CLI 재저장 8개 IR diffCount 0, 1페이지 유지. 재적재 후 7글자의 이슈 대상 5속성 일치. |
| Native PNG 전체 페이지 비교 | before=undo, highlight=redo, 동일 상태 HWP=HWPX: 총 8쌍 모두 0픽셀 차이. |

아래는 CLI 렌더링이다. 왼쪽 HWP/오른쪽 HWPX, 위에서 적용 전/형광펜/Undo/Redo.
Chrome 스크린샷이나 한컴 독립 정답지가 아니다.

![저장 후 재열기 8개 파일의 native 렌더링](../pr/assets/issue6788_cli_roundtrip.png)

## 한계와 후속 게이트

- 사용자 승인에 따라 확장을 교체하지 않고 로컬 Studio 및 CLI 분리 경로로 검증했다.
  Firefox 직접 UI, 브라우저 OS 저장 창, 8개 파일의 브라우저 UI 전수 열기는 완료하지 않았다.
- Docker daemon 부재로 locked native `--no-opt` WASM을 사용했다. 최적화 배포 산출물 검증은 아니다.
- nextest 성공 실행에 `issue_2007_single_cell_continuation_does_not_repaint_boundary_fragments`
  LEAK 1건이 있었다. 잔류 출력 핸들/자식 프로세스의 원인은 확정하지 않았다.
- 최초 HWPX export의 fillType/patternColor/patternType 차이는 적용 전부터 관찰됐다.
  이슈 대상 textColor/shadeColor/bold/fontSize/fontFamily 보존과 전체 속성 무손실을 구분한다.
- 브라우저 webfont와 native 시스템 fallback의 글꼴 모양은 다를 수 있다. 광범위 렌더링 fidelity
  또는 전체 visual sweep 통과를 주장하지 않는다.
- PR 준비까지 완료하며 push·PR 생성·CI·merge·이슈 종료는 별도 승인/후속 게이트다.
  PR 번호 확정 전 archive self-review 및 오늘할일을 만들지 않는다.

로컬 Studio 서버는 사용자 추가 확인용으로 유지한다. 상세 명령·초기 실패·재실행·산출물 경로는
3단계 보고서에 있다. 원시 로그·중간 JSON·중복 PNG는 커밋하지 않는다.
