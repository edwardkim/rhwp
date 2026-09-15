# #6706 단독 해결 — 저장 줄의 표·그림 소유

Issue: [#6706](https://github.com/edwardkim/rhwp/issues/6706).
작업 브랜치: `codex/issue-6706-stored-inline-rows`.
기준: `upstream/devel` `769582fc856f162e57604b318d41414d7b026345`.

## 분석

사용자 지시에 따라 이슈를 하나씩 완료한다. 기존 `codex/jeongsik-issues-20260915`의
누적 커밋과 미커밋 변경은 원 작업 트리에 보존하고, 이 회차에는 #6706 변경만 분리했다.
분석 → 코드 수정·검증 → 결과보고 → 커밋 순서를 적용했다.

원본 `samples/hwp3-sample16-hwp5.hwpx` 문단 0.394의 표·그림은 원시 UTF-16 위치
0/8/18, 가시 문자 위치 0/0/2로 투영된다. 가시 문자 위치만으로 줄을 나누면 서로 다른
저장 줄의 표와 그림을 같은 줄에 배정한다. 측정과 배치가 같은 원본 소유 결과를 사용해야 한다.

최신 devel에서는 표 제목이 이미 표시되지만 x=147.1px로 오른쪽으로 밀리고,
그림은 (484.3,306.5)px에서 페이지 밖으로 잘린다. 최초 이슈의 “표 글자 소실”과
현재 재현 증상을 구분했다. 기대 위치는 Git에 있는 한컴 2022 PDF 18쪽에서 정한다.
기존 입력과 PDF를 이름만 바꾸어 중복 추가하지 않았다.

## 코드 수정과 공통 결과

단독 후보 `792919670`의 변경 및 누적 후보 `4b5a16e30` 중 수식 전용 재조판 제외
조건만 분리했다. 다른 이슈의 폰트·문단 간격·저장 프레임·분할 변경은 포함하지 않았다.
최종 제품 코드 4개 파일과 새 integration test 1개이며, golden·baseline·허용치는 바꾸지 않았다.

| 단계 | 호출 경로와 계약 |
| --- | --- |
| 소유 줄 결정 | `src/renderer/composer.rs:1532`의 `stored_tac_line_assignment`가 원시 컨트롤 위치와 저장 줄 경계로 `(control, line)`을 반환한다. 계산 줄 표식, 줄 수·높이·폭·문자 시작 불일치이면 적용하지 않는다. |
| 측정 | `src/renderer/typeset.rs:2257`의 `tac_control_indices_for_line`이 동일 결과를 소비해 각 줄의 개체 높이를 측정한다. |
| 실제 배치 | `src/renderer/layout/paragraph_layout.rs:4502`에서 동일 결과로 줄별 TAC를 제한하고, 정렬 폭과 run 방출(`:7018`)에 적용한다. |
| 좌표 | `src/renderer/layout/paragraph_layout.rs:5048`에서 본문 저장 줄의 column_start가 이미 문단 여백인 경우 중복 여백을 제거한다. 셀 내부 좌표 계약과 분리한다. |
| 반례 | `src/renderer/equation_tac_flow.rs:26`의 수식 전용 재조판은 현재 너비 기반 흐름을 유지한다. 같은 줄 그림+표, 뒤 줄 TAC, 양수/음수 저장 간격, 수식·그림 corpus를 검사했다. |

페이지 분할·rowspan·fragment 컷 알고리즘은 바꾸지 않았다. 측정과 배치의 개체 소유를
일치시키는 변경이다. 전체 회귀의 분할·이어받기 반례도 통과했다.

## 검증 결과

실행 환경은 macOS arm64, 논리 CPU 10개/메모리 32GiB, Rust 1.93.1이다.
전용 target은 `/Users/tsjang/rhwp/target/issue-6706-20260915`이며 기존 target을 삭제하지 않았다.
Cargo 검사는 순차 실행했고, 실행 중 Rust source/test를 수정하지 않았다.

| 검사 | 결과 |
| --- | --- |
| 집중 반례 | #6706/#6754/#5727/#7103/#6660/#1139 **102/102 통과** |
| 최종 전체 회귀 | **9,885 실행 / 9,885 통과 / 0 실패 / 51 skip**, 334.878초, exit 0 |
| Native root Clippy | `-D warnings` 통과 |
| WASM32 lib Clippy | `-D warnings` 통과 |
| Workspace build 및 all-target Clippy | `--locked`, `-D warnings` 통과 |
| Native Skia lib | **4,112 통과 / 0 실패 / 13 ignore** |
| Native Skia 그림 placeholder | **2/2 통과** |
| Native Skia 직접 PDF export | **4/4 통과** |
| Fresh WASM | locked wrapper의 release/web 빌드 및 wasm-opt 완료, exit 0 |
| 포맷·파생 manifest | fmt check와 manifest check 통과 |
| OVR5 | 표 개체 회귀 0건. 추가로 **5문서 142쪽 render tree가 수정 전후 동일** |
| Native Visual Sweep | 동일 원본 64쪽 중 **18쪽만 변경**, 나머지 63쪽 SVG·render tree byte-identical |
| Fresh WASM Visual Sweep | Chrome/152.0.7977.83에서 `renderPageSvg`와 `getPageRenderTree` 실행. 64쪽 생성, 18쪽 PNG가 native와 byte-identical |

전체 회귀 명령:

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /Users/tsjang/rhwp/target/issue-6706-20260915 \
  --tests --test-threads 8 --no-fail-fast
```

집중 검사 뒤 중첩 참조만 정리했으며, 위 전체 회귀·Clippy·Skia·WASM 검사는 모두
정리 후 최종 코드에서 실행했다. 최종 native 재캡처의 64쪽 SVG·render tree와 18쪽
compare/overlay PNG도 최초 단독 후보와 byte-identical임을 확인했다.
기존 누적 브랜치의 테스트 결과를 이 단독 후보의 통과 근거로 재사용하지 않았다.

## 시각 판정과 증거

| 대상 | 수정 전 | 수정 후 | 한컴 기준 |
| --- | --- | --- | --- |
| 표 제목 x | 147.1px | 107.1px | 약 107.0px |
| 표 제목 baseline | 저장 줄 유지 | 318.48px | 318.89px |
| 그림 좌상단 | (484.3,306.5)px | (108.1,340.2)px | 약 (107.9,339.8)px |

같은 18쪽의 native 수정 전/후와 fresh WASM 비교 이미지를 직접 열어 확인했다.
제목·그림·뒤 제목의 순서, 한 번만 배치됨, 그림 전체의 페이지 내부 표시를 확인했다.
그림의 색상 및 문서의 기존 글꼴·장식 차이는 남아 있다. #6706의 줄 소유와 위치 해결을
문서 전체의 raster identity나 다른 열린 이슈 해결로 확대하지 않는다.

- [수정 전 / 한컴 PDF](../pr/assets/issue6706/p018_before_pdf.png)
- [최종 native / 한컴 PDF](../pr/assets/issue6706/p018_after_pdf.png)
- [최종 native overlay](../pr/assets/issue6706/p018_after_overlay.png)
- [fresh WASM / 한컴 PDF](../pr/assets/issue6706/p018_wasm_pdf.png)
- [입력·소스·바이너리 해시와 64쪽 비교](../pr/assets/issue6706/visual_provenance.json)
- [OVR5 전수 render tree 비교](../pr/assets/issue6706/ovr_tree_audit.json)
- [명령·완료 코드·검증 집계](../pr/assets/issue6706/verification.json)
- [원시 검증 로그](../pr/assets/issue6706/verification_logs.tar.gz)

## 완료 범위

**#6706의 단독 로컬 수정·검증 완료.** 이번 회차에서 해결한 이슈는 1개다.
원격 push·PR·CI·merge·GitHub 이슈 종료는 아직 수행하지 않았다.
다른 11개 열린 이슈와 기존 누적 후보는 이 결과에 포함하지 않는다.
