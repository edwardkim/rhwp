# PR #8951 리뷰 — footer 문서 body overflow 기준값 정리

## 최종 판정

**메인테이너 보정 후 수용 가능 — 필수 로컬 검증 완료.** Native/fresh WASM 전 13쪽과
실제 body overflow, 집중 8건·전체 회귀 13,432건·Native 공식 4,073건을 통과했다.
정본 보호 전 3,542쪽의 공식값 하락은 0건이다. 원격 처리는 사용자 승인과 새 통합 head의 CI가 필요하다.
GitHub 승인 리뷰·push·merge는 수행하지 않았다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 작성자·base | sacru2red / devel |
| 원 head | `22acc33b7e06647485bae503403ac6a465fb0dc8` |
| 최신 기준 | `180a4ec97205b67c4dc4f90416063c01ecfdf396` |
| 누적 검토 source | 제품 보정 `e5a6a1ff9f`, 회귀 보완 `b4b8c27260`(두 원 SHA를 조상으로 보존) |
| reviewer | edwardkim, 2026-11-13 지정 확인 |
| 원 head CI | 2026-11-13 재조회: success 32·정상 skip 5 |
| 병합 시뮬레이션 | 최신 base와 exit 0. 현재 GitHub MERGEABLE / CLEAN |
| 연결 이슈 | #7620, 현재 OPEN. 이번 검토만으로 자동 close하지 않음 |

## 변경과 선행 분석

제품 코드 변경 없이 `tests/fixtures/body_overflow_baseline.tsv`에서 footer 문서의 허용치 1행을
제거한다. #7644 → #7681(`4c7092397e6fb6d1d16e5521bb9049ef25814d5a`) 검토 당시
11쪽 표가 선언 높이보다 8.3px 크게 그려져 본문 바닥을 4.5px 넘었던 예외를 제거하는 변경이다.

현재 실제 표는 y=126.7px·h=916.5px, 끝 1043.2px다. 본문은 y=75.6px·h=971.4px로
바닥 1047.0px 안에 있다. 같은 입력의 전 13쪽 `layout-anomaly --overflow-tolerance 1 --json`에서
overflow·off-canvas·overlap·text-overlap 신호는 모두 0이다. baseline consumer는
`tests/cases/body_overflow_baseline.rs::count_doc`의 공통 `DocumentCore` render tree와 `scan_page`다.
baseline이 없는 문서는 0건이어야 하므로 허용치를 완화하는 변경이 아니다.

## 시각 검증

입력은 기존 `samples/issue6465/press_release_footer_logos.hwpx`, 기준은 기존 Desktop Hancom 2020
Print `pdf/pr6485-visual/pr6485-issue6465-press-release-footer-logos-2020.pdf`다. 기존 기준 PDF
SHA-256은 `d5a3db9320675907e92a3407d75ce0726fbe605e5bf19fe9bdc46befd07d1487`이다.

Native Skia·96dpi·`--compat 2022` 전 13쪽 최저 raw 96.17%(1쪽), 목표 11쪽 97.80%이며
90% 미달은 없다. fresh 최적화 WASM Canvas도 전 13쪽 최저 96.11%(1쪽), 목표 11쪽 97.52%로
모두 통과했다. Print와 1·11쪽을 직접 대조해 표 아래 괘선·본문 경계·쪽 번호 보존을 확인했다.
원 head의 과거 측정값과 이번 최신 devel 측정값을 섞지 않는다.
[보정 후 전쪽 TSV·대표 PNG·실행 조건](assets/sacru2red-20261113/final/README.md)을 함께 보존했다.

## 조판 원칙·검증 입력 검토

새 배치 규칙·휴리스틱·좌표 clamp·소유/분할 경로 변경은 없다. 기준값 제거의 근거는 현재
실제 공통 렌더 트리와 독립 Print다. 입력·PDF는 저장소의 기존 파일을 그대로 사용했다.
보정 전 Native는 source `60f5659d5b`의 producer 입력 3363개 mode/blob/path가 최신 기준과
같음을 확인했다. 이후 사용자 승인으로 #8975의 공통 간격을 보정한 `e5a6a1ff9f`에서도
footer Native/WASM PNG 전 13쪽은 각각 그대로이며 허용치 제거가 유효하다. 두 producer의
SHA와 보정 전후 비교는 위 최종 원장에 구분했다.

계획·#8975와의 범위 구분은 [공통 계획](sacru2red_20261113_review_plan.md)을 따른다.
