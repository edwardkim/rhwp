# PR #9339 검토 — 표 한 칸의 자리차지 그림 배치

## 메인테이너 보정과 정본 보호 상태

현재 PR의 기여 head는 `f5bd466536d25250442d14d08e070b7061415e4c`이다. 표 칸의 그림별 정렬과 겹침 처리를 보완한 소스에 #9350·#9355를 포함한 devel `17b17c934b527aa523f037ca12d8713394e43c82`를 병합했다. 로컬 검증 소스는 `c1504cb12199081c010188557ff2f1e14d36d5ef`다. [보완한 공통 계산](pr_9339_review_impl.md)

공개 원본 두 개와 단일 그림의 속성을 바꾼 대조군 열 개 모두에서 한컴 2024 인쇄본과의 전쪽 실루엣 일치율은 Native·fresh WASM 각각 100.00%였다. 오른쪽·가운데 정렬, 겹침 허용, 칸 경계, 표 뒤 문단의 위치를 비교 그림에서도 확인했다. 보완 전에는 기존 검사 2개가 통과하고 새 경계 검사 3개가 실패했다. 보완 후 집중 검사 25개가 통과했다. [한컴·기여본·보완본 비교](assets/pr9339/final/README.md)

공개 글꼴만 노출한 환경에서 `cargo nextest`의 전체 회귀 14,017개가 통과했다. 건너뛰기 항목은 45개였다. 같은 호스트에서 기준 devel과 보정본으로 #9350의 문서 10종을 출력해 Native·WASM 각각 583쪽의 PNG가 모두 같은 것을 확인했다. [별도 빌드와 새 devel 기능 검증](assets/pr9339/final/README.md#새-devel의-기능-검증)

**정본 보호 검증을 완료할 때까지 머지 보류.** 명시적 공급 대상 312개 중 311개를 확보했다. 메인테이너가 허용한 Windows 11 설치본 6개로 문서 6종을 추가 측정했고, Native·fresh WASM 각각 276쪽 모두 로컬 devel과 후보의 PNG가 같았다. 그중 공식 하한 미달은 두 소스에서 똑같이 Native 65쪽, WASM 63쪽이다. 단독 `dotumche.ttf` 미확보로 화학물질 문서 103쪽의 정본 검증이 남았다. [파일별 허용 해시](assets/pr9339/final/font-supply.md) · [쪽별 결과](assets/pr9339/final/protected-windows-fonts.tsv)

## 접수 당시 판정

**원 기여 보완 필요.** 기여자가 제출한 두 원본은 개선됐다. 같은 위치에 놓인 네 번째 그림을 오른쪽 정렬로 저장하면 위아래 그림이 다시 포개진다. 네 번째 그림의 중심만 칸의 오른쪽 경계를 넘긴 경우에도 앞선 세 그림의 배치가 달라진다. 두 경우 모두 한컴 2024에서 같은 HWPX를 열어 인쇄하고 확인했다. [동일 위치 정렬 비교](assets/pr9339/equivalent-alignment.png) · [칸 경계 비교](assets/pr9339/center-crossing.png)

## 접수와 기준

| 항목 | 값 |
| --- | --- |
| 원 PR·연결 이슈 | [#9339](https://github.com/edwardkim/rhwp/pull/9339), [#9338](https://github.com/edwardkim/rhwp/issues/9338) |
| 기여자·리뷰어 | hyunmyungJaneLee · edwardkim (2026-11-23 15:04 KST 지정) |
| 원 기여 head | `1cc8f4c457c81cf3378d2467e1bca5c869060b3e` |
| 검토 기준 devel | `7d6628982c81e702d58d8d0e6f52059f68a0b9ae` |
| 로컬 병합 소스·트리 | `50aa1a326c1edc2721be81301df5e3c00de6a1fc` · `d8df38b6a768ec798cb1530088b22539b1febc96` |
| 기여 기준과 최신 변경 | 공통 조상 `197ce19e99` 뒤 devel 313커밋. 최신 기준과 실제 로컬 병합 및 `git merge-tree --write-tree`가 같은 트리를 만들었다. |
| 원 head CI | [Build & Test 성공](https://github.com/edwardkim/rhwp/actions/runs/45096480154) · [정책 감사 성공](https://github.com/edwardkim/rhwp/actions/runs/45099277302) |

## 인쇄와 배치

공개 원본은 `grid2x2.hwpx`와 `row3-mixed.hwpx`다. 두 원본에서 직접 다시 인쇄한 1쪽씩은 제출된 데스크톱 인쇄본과 96 DPI 전쪽 RGB 픽셀이 같았다. 추가 대조군 10개는 첫 원본의 HWPX에서 그림 하나의 속성을 수정했다. 각각 같은 Windows 한컴 2024 13.0.0.3901과 Hancom PDF 1.3.0.550으로 직접 인쇄했다. 입력·PDF·인쇄 작업과 SHA-256은 원장에 있다. [인쇄 원장](assets/pr9339/print-inputs.tsv)

| 문서 또는 변경값 | 한컴 2024에서 확인한 동작 | devel Native | PR 적용 Native |
| --- | --- | ---: | ---: |
| 원본 2×2 | 각 열에서 위아래로 두 장씩 표시 | 58.57% | 100.00% |
| 원본 혼합 배치 | 세 열 두 줄로 여섯 장 배치 | 18.55% | 100.00% |
| 그림 4만 같은 위치의 오른쪽 정렬 | 원본 2×2와 전쪽 픽셀이 동일 | 58.57% | 58.57% |
| 그림 4만 겹침 허용으로 직렬화 | 그림 4만 위쪽으로 이동, 왼쪽 열 두 장의 위치 유지 | 42.94% | 42.94% |
| 그림 4 가로 오프셋 27000 → 28000 | 그림 4만 오른쪽으로 13.43px 이동 | 50.35% → 49.26% | 100.00% → 49.26% |
| 그림 1의 위·아래 바깥 여백 / 그림 4의 양수 세로 오프셋 | 각 저장 속성에 따라 그림 위치와 칸 높이 결정 | 48.67% / 20.56% | 100.00% / 100.00% |

오른쪽 정렬 대조군의 네 그림 위끝은 한컴 인쇄에서 204.74, 204.74, 355.77, 355.77px이다. 후보 Native는 네 장을 모두 358.0px에 배치해 두 장만 보인다. 마지막 그림의 가로 정렬 또는 가로 위치 하나가 왼쪽 열의 세로 배치까지 취소한다. 공식 비교는 96 DPI, `--compat 2024 --font-path ttfs/opensource`다. 기여자가 사용한 `--compat 2022`에서도 Native의 두 원본은 100.00%, 동일 위치 오른쪽 정렬은 58.57%였다. [그림별 좌표](assets/pr9339/picture-bounds-96dpi.tsv) · [쪽별 실측](assets/pr9339/page-scores.tsv) · [겹침 속성 비교](assets/pr9339/overlap-flag.png)

## 원인과 검증 범위

`unstored_float_stack::paragraph_frame`은 개별 그림이 오른쪽 정렬이거나 중심이 칸 폭 바깥이거나 겹침 허용이면 함수 전체에서 `None`을 반환한다. `reflow_picture_cell_frame` 역시 문단 프레임 하나의 부재를 칸 전체에 전파한다. 높이 측정, 행 높이, 그림 위치, 칸 세로 정렬이 이 결과를 사용하므로 다른 그림의 배치와 뒤 문단 위치까지 달라진다. 기존 그림 프레임에는 가로 기준·정렬·오프셋을 물리 좌표로 변환하는 경로가 있다. [중단 조건](../../src/renderer/float_placement/unstored_float_stack.rs#L59) · [칸 프레임과 정렬 계산](../../src/renderer/float_placement.rs#L975) · [높이 측정](../../src/renderer/height_measurer.rs#L5132) · [실제 배치](../../src/renderer/layout/table_layout.rs#L7885)

원 기여의 회귀 두 개는 관계형 검사다. 테스트 첫머리에 웹한글기안기 기준이 남아 있어 제출 시 사용한 데스크톱 Print, Native·WASM 실측, 측정 시각과 증거 경로를 함께 반영할 필요가 있다. 공개 원본과 경계 입력으로 독립 검토했고, 기여자가 설명한 사내 원문 23개는 직접 재검증하지 않았다. [회귀 검사](../../tests/cases/cell_negative_offset_picture_stack.rs) · [보완 계획](pr_9339_review_impl.md)

원 head의 원격 CI는 위 표의 실행이다. 현재 로컬 비교는 최신 devel과 기여 head의 병합 소스를 사용한다. 전체 회귀·정본 보호 4,046행은 수용 후보의 검증 항목으로 남는다. #9338의 완료 여부는 재현된 그림 겹침을 해소한 뒤 판정한다.


## 검토 도중 갱신된 원 PR과 devel

2026-11-23 16:27 KST에 원 PR의 현재 head `f5bd466536d25250442d14d08e070b7061415e4c`를 확인했다. 지정 당시 head와 여섯 커밋의 메시지 이력이 바뀌었으며, 두 끝의 Git 트리는 모두 `fe72add8719994c88a0d250ceacfc3db0ee46973`이다. 수용 후보는 기여자의 현재 여섯 커밋을 부모로 삼는다.

같은 사이 devel에 [#9351](https://github.com/edwardkim/rhwp/pull/9351)이 병합되어 기준을 `be6f60aa6a93889b1bbc10e7e4a113dcc27a7f07`로 갱신했다. 이 기준과 원 기여를 실제로 병합한 커밋은 `e4e0b16bca2bec78ab5e64d9b4da44b383edaf85`다. 새 의존성으로 공개 원본, 경계 대조군, 최근 조판 변경, 정본 보호와 Rust 검증을 다시 수행한다. 앞 절의 결과는 표에 적은 시점의 소스에서 측정했다.
