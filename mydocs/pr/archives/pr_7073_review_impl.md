---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-13
---

# PR #7073 등 7건 — 체리픽 통합 검토 계획

- 사용자 지시: `upstream/devel` 동기화 후 planet6897의 열린 PR 중 #7089를 제외하고 체리픽 통합·검토한다.
- 기준: `897c6a3d8d7559d314bf863c93bbe28c0d65e945`; 주 작업공간 branch `review/planet6897-20260913`.
- 대상: #7073, #7074, #7075, #7082, #7083, #7087, #7088. #7089 고유 변경은 제외한다.
- 원 contributor branch는 보존하고 `git cherry-pick -x`로 출처를 기록한다.

## 적용 순서

| 원 PR | 적용할 고유 commit |
| --- | --- |
| #7073 | `fa25d28113f3a1d09265e2ca90ca8b1c5a544be2`, `b898ae3060c688ccdd96a1d720fe6be55658dae2`, `6952477ae783235377495caeaedd0a23f1a21373` |
| #7074 | `416e894a8133608829788cb06f7cb757751f4b5a` |
| #7075 | `2f41f46f1fd0a268a324e7187edafc732d739f27` |
| #7082 | `85b8726dd175b25f5efb743c009e8da81e146d1b` (#7075 중복 제외) |
| #7083 | `4d9fd37b186332607640915825fa5d3973f4beb6` (#7074 중복 제외) |
| #7087 | `16ee2974ba7b057e39f2fd0ab572b54c18406e72` |
| #7088 | `186953b0369aa2f962848fdadb70f85f1a60a6ef` (#7075·#7082 중복 제외) |

## 단계와 판정 범위

1. 최신 devel 동기화·원 PR reviewer 지정·head fetch 완료. 비교용 baseline 바이너리를 전용
   `target/pr-planet6897-20260913`에서 빌드한다. 공유 target과 다른 작업 산출물은 보존한다.
2. 위 commit을 순차 적용하고 충돌 여부·대응 SHA를 기록한다. 원 PR별 변경 목적과 공통 조판 원칙,
   baseline 변경의 독립 근거를 검토한다. #7083의 개체 위/뒤 leading 설명 불일치도 바로잡는다.
3. 검증에 쓴 원본 HWP/HWPX/PDF를 같은 branch에 보존한다. 기준 PDF가 있는 #7083·#7088은
   기존 blob을 재사용하고, #7073·#7087은 engine 2020 PDF를 보완한다. #4680 실측 원본도 보존한다.
4. 통합 code head의 Rust lint 묶음, 집중·전체 nextest, Native Skia 3종, WASM과 실제 PDF/SVG
   before/after visual sweep을 완료한다. 실패는 허용치 증가로 숨기지 않고 원인을 판정·보정한다.
5. 검증 완료 후 원 PR별 archive review·대표 PNG·필요한 오늘할일을 같은 통합 PR에 포함한다.
   owner를 reviewer로 자동 지정하지 않는다. 별도 통합 PR 번호의 self-review 문서는 만들지 않는다.
6. 최신 통합 head CI 확인 후 merge한다. 통합 merge가 확인된 뒤에만 원 PR에 고정 증적과 통합
   provenance를 게시하고 close한다. #4680의 부분 해결을 전체 완료로 처리하지 않는다.
7. post_merge.md에 따라 devel 동기화·duration refresh 결과·원 PR/이슈 후속 상태를 확인하고,
   소유 branch/ref/전용 target만 정리한다. devel push에서 CI를 추가 실행하지 않는다.

현재는 검토 계획이며 테스트 통과·원격 통합·merge 완료를 주장하지 않는다.

## 통합 시각 검토에서 발견한 잔여 호스트 전진 — 검증 재개

- 최초 통합 code `4aa80b96a`의 전체 nextest는 9,564 PASS / 46 skipped였다.
  이 결과만으로 수용하지 않고 하단 로고까지 before/after/한컴 PDF를 재검토했다.
- 3쪽 법무부 로고의 그림 상단은 후보 1029.7px, 국토교통부·서울시 로고는 1047.7px다.
  한컴 PDF의 세 이미지 rect 상단은 모두 784.901pt(1046.535px)다.
- `TopAndBottom` 그림 host pi=81이 빈 문단 pi=82로 이어질 때 저장 LineSeg가 있는데도
  기존 overlay/Square 전용 사다리 질의에 들어가지 않는다. 같은 빈 개체 host 전진 계열이며
  #7089의 저장 LineSeg 없는 host 앞 spacer 변경은 포함하지 않는다.
- 해당 저장 사다리 질의의 일반 적용 범위를 검토하고, 로고의 동일 상단·글상자와의 간격을
  독립 한컴 출력 기준으로 회귀 고정한다. 좌표 상수·clamp·문서 ID 분기는 사용하지 않는다.
- 앞서 native/WASM Clippy는 통과했으나 workspace build를 중단했다. 새 code head의 전체
  검증·세 Clippy·Native Skia·WASM·시각/OVR 증거를 다시 실행하기 전에는 머지 가능으로 판정하지 않는다.
