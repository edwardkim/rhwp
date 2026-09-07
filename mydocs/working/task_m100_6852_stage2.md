# #6852 재착수 Stage 2 — 일반 도형 실선 보존

- Issue: [#6852](https://github.com/edwardkim/rhwp/issues/6852).
- 2026-09-08, [수정 수행계획](../plans/task_m100_6852.md) 승인에 따른 재착수.
- 브랜치 `task_m100_6852_baseline`, 기준 제품 `7138fe784`, 검증 후보 `9f4f451b7`.
- **최소 구현·집중 회귀·CLI SVG 비교 완료. 메인테이너 시각 판정 및 Stage 3 광범위 검증 대기.**

## 1. 제품 변경과 보호 경계

제품 변경은 `src/renderer/layout/shape_layout.rs` 한 파일이다. 일반 사각형의 흰색 채우기를
마스크로 추정해 선을 지우던 B 분기를 제거했다. 기존 함수는
`should_suppress_group_textbox_construction_stroke`로 이름을 바꾸고 `text_box`가 없으면
즉시 false를 반환하도록 했다. 빈 글상자는 `Some(TextBox)`로 남고 가시 텍스트 조건을 통과하지 않는다.

기존 A 글상자 분기의 조건은 유지했다. 주석은 경험적 보정임을 명시하며, 글상자라는 사실이
곧 비인쇄를 뜻한다고 주장하지 않는다. 원본의 명시적 선 없음도 실선으로 강제하지 않는다.
앞선 `task_m100_6852@07188b9f7`의 그룹 선 API·UI 코드는 가져오지 않았다.

## 2. 수정 전 실패와 집중 시험

새 원본 시험은 `tests/cases/issue_6852_group_rectangle_stroke.rs`다. SVG XML의 실제 사각형을
선택하여 검정 stroke·굵기와 앞뒤 사각형의 순서·좌표·쪽수를 검사한다. 문자열 속성창 테스트가 아니다.

검증용 worktree는 기존 `/home/edward/mygithub/rhwp-review-6852`를 재사용했다.
각 검증 commit으로 detached 전환하고 suite를 그곳에서만 준비했다. Cargo target도 기존
`/home/edward/mygithub/rhwp-6812-review-target`를 유지하며 Cargo 명령을 동시에 실행하지 않았다.

- `ee794278f` (제품 baseline): **4 PASS / 2 FAIL**.
  실제 HWP·HWPX의 앞쪽 실선 검사만 `stroke=None`으로 실패했다.
  일반 무채움 도형, 명시적 선 없음, 빈/공백 글상자, 기존 A 동작의 네 시험은 통과했다.
- `9f4f451b7` (수정 후보): **6 PASS / 0 FAIL**. 두 원본의 실선 검사가 GREEN으로 전환됐다.
  실행 필터로 제외된 다른 suite 시험은 이번 focused 통과 건수에 포함하지 않는다.
- 테스트 최초 작성에서는 `<clipPath>` 안의 사각형까지 셌던 준비 오류가 있었다.
  실제 paint 사각형만 선택하도록 고친 뒤 위 결과를 확정했다. 제품 기대값은 낮추지 않았다.
- fmt check에서 그 선택자의 줄바꿈을 정정했다. review에서 생성한 포맷 변경을 source에 반영해
  새 commit으로 검증을 재시작했다. 파생 suite·manifest는 source에 가져오지 않았다.

검증 후보 `9f4f451b7`에서 전체 fmt check, suite 배정 정책 21건, manifest check,
locked release-test CLI build를 통과했다. Rust 전체 integration·Clippy·Native Skia 결과는 아니다.

## 3. 시각·범위 증적

로컬 산출물은 `output/6852/restart/` 아래 보존한다. 원본 HWP/HWPX는 읽기 전용이다.
수정 전 네이티브 CLI는 `ee794278f`에서 직접 빌드했다. 이전 WASM으로 만든 참고 baseline과
`before-native-*` CLI 증적을 구분한다. 수정 전후 CLI의 같은 `layer` 출력 경로를 비교한다.

대상 HWP·HWPX 전체 11쪽과 기존 편람 HWPX의 현 baseline 382쪽을 비교 대상으로 잡았다.
편람의 과거 보고서 쪽수를 이번 환경의 실측으로 대체해 적지 않는다.
비교는 새로 복원된 흰 사각형의 stroke 외에 좌표·그림자·텍스트·이미지·쪽수가 달라졌는지 확인한다.
전수 코퍼스 sweep 또는 새로운 한컴 출력 수집은 하지 않았다.

### 비교 결과

`comparison.json`과 `comparison.log`에 CLI 전후 결과를 기록했다. 페이지별 사각형 수·순서를
비교하고, 변경 사각형마다 흰 채우기에 검정 stroke와 stroke-width만 추가됐는지 검사했다.
추가된 속성만 되돌린 SVG가 수정 전 SVG와 바이트 동일함을 확인하여 비대상 변경도 검사했다.

| 입력 | 수정 전후 쪽수 | 변화 페이지(1-based) | 변화 내용 |
| --- | --- | --- | --- |
| 실제 HWP | 11 → 11 | 5, 6, 7, 8 | 일반 흰 사각형 총 7개에 원본 선 복원, 다른 SVG 내용 동일 |
| 메인테이너 제공 HWPX | 11 → 11 | 5, 6, 7, 8 | 일반 흰 사각형 총 7개에 원본 선 복원, 다른 SVG 내용 동일 |
| 기존 편람 HWPX | 382 → 382 | 없음 | 382쪽 전체 SVG 바이트 동일 |

따라서 1쪽은 전체 출력이 동일하다. 7쪽에는 같은 오탐으로 사라졌던 도형 선이 복원됐지만,
앞서 수용한 표 배치·간격을 포함한 나머지 SVG는 동일하다. ‘7쪽 전체 SVG 불변’이라고 적지 않는다.
편람의 A/B 분기를 각각 모두 실행했다는 뜻도 아니며, 현재 fixture·baseline에서 결과가 같다는 증거다.

메인테이너 판정 경로:

- HWP 5쪽: `output/6852/restart/after-hwp/156160455-social-pig-farm-income_005.svg`
- HWPX 5쪽: `output/6852/restart/after-hwpx/156160455-social-pig-farm-income_005.svg`
- 각 수정 전 CLI 출력은 `before-native-hwp/`, `before-native-hwpx/`의 같은 파일명이다.
- 대상 5쪽의 흰 사각형은 `stroke="#000000" stroke-width="0.5"`를 가지며 뒤쪽 그림자는 불변이다.

참고용 baseline Node 스크립트의 최초 실행은 import 상대경로 오류로 실패했고,
경로를 고친 뒤 실행했다. 이 참고 산출물은 최종 동일-backend CLI 비교와 구분했다.
5쪽은 아직 메인테이너의 수정 후 시각 판정을 받지 않았으므로 수용 완료로 표시하지 않는다.

## 4. 남은 승인 경계

- 후보 코드의 집중 시험·SVG 비교는 완료했다. 보고서 후속 commit은 문서만 변경한다.
- 5쪽 SVG의 실제 선 복원은 메인테이너가 판정한다. 편람의 변화는 별도로 보고하며 자동 PASS로
  간주하지 않는다. 기준선과 같음 또는 모든 시험 통과만으로 한컴 정합을 선언하지 않는다.
- Stage 3 Rust 전체·Native Skia·새 fixture 래칫/보안·Docker WASM은 아직 실행하지 않았다.
- GitHub 이슈 본문은 기존 API 바인딩 범위가 남아 있다. 별도 승인 없이 원격 본문을 수정하거나
  PR·push·close를 진행하지 않았다.
