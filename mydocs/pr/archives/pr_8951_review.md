# PR #8951 리뷰 — footer 문서 body overflow 기준값 정리

## 최종 판정

**승인 — 통합 PR #9047에 원 기여를 보존해 수용 완료.** 검증 head `68c2441bfd`의
34 SUCCESS·정상 SKIP 5건과 셀프 리뷰 뒤 merge `a785e23f6838d5452e8904cae82f81eb5e185efc`로
반영했다. 원 PR #8951도 간접 MERGED를 확인했다. 범위는 불필요한 허용치 1행 제거다.


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
[보정 후 전쪽 TSV·대표 PNG·실행 조건](../assets/sacru2red-20261113/final/README.md)을 함께 보존했다.

## 조판 원칙·검증 입력 검토

새 배치 규칙·휴리스틱·좌표 clamp·소유/분할 경로 변경은 없다. 기준값 제거의 근거는 현재
실제 공통 렌더 트리와 독립 Print다. 입력·PDF는 저장소의 기존 파일을 그대로 사용했다.
보정 전 Native는 source `60f5659d5b`의 producer 입력 3363개 mode/blob/path가 최신 기준과
같음을 확인했다. 이후 사용자 승인으로 #8975의 공통 간격을 보정한 `e5a6a1ff9f`에서도
footer Native/WASM PNG 전 13쪽은 각각 그대로이며 허용치 제거가 유효하다. 두 producer의
SHA와 보정 전후 비교는 위 최종 원장에 구분했다.

계획·#8975와의 범위 구분은 [공통 계획](../sacru2red_20261113_review_plan.md)을 따른다.

## Merge 후 contributor PR comment 계획

- 실제 [수용 PR #9047](https://github.com/edwardkim/rhwp/pull/9047), merge `a785e23f6838d5452e8904cae82f81eb5e185efc`, 검증 head `68c2441bfd947609676b841e7ff43bf1c64983ab`를 쓴다.
- 문서 비교는 [정본 Visual Sweep 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 따른다.
- 실제 footer 전 13쪽의 Native 최저 96.17%, fresh WASM 최저 96.11%와 body overflow 0건을 구분한다. 대표 11쪽은 각각 97.80%·97.52%다.
- inline PNG는 같은 footer 11쪽의 `mydocs/pr/assets/sacru2red-20261113/final/footer-p011-three-way.png`다. 독립 Desktop Print·Native·fresh WASM을 같은 배율로 대조한 이미지이며, 게시 직전 merge SHA raw 응답의 PNG magic·SHA를 Git bytes와 비교한다.
- 사람의 판정은 표 아래 괘선·본문 바닥·쪽 번호의 보존과 overflow 0건이다. 수치를 사람 판정 정확도나 RGB 완전 동일로 설명하지 않는다.
- 공개 댓글은 이 archive·운영 기록의 devel 반영 후 게시하고 본문 일치·한글·BOM·줄바꿈을 API로 확인한다. 원 fork branch는 보존하며 #7620은 기존 전체 잔여 범위로 OPEN을 유지한다.
