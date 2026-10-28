# task_m100_3744 Stage 6 작업기록 — PR review 보정

- **Issue**: #3744
- **PR**: #3933
- **review**: `issuecomment-5174857326`
- **보정 기준 head**: `ce6a23bca`
- **최신 결합 base**: `upstream/devel` `301d0fe5f`
- **범위**: review 재현, 제품·회귀 보정, corpus 무회귀, self-merge 문서 준비와 push

## 1. review 판정

review의 네 제품 경계를 독립 대조했다.

1. 복합 번호에서 anchor를 영구 만료하면 뒤에서 즉시 이어지는 정상 `호`까지 누락된다: 유효.
2. 탭 없는 dotted leader 목차 행이 direct `목`으로 승격될 수 있다: 유효.
3. ParaShape negative가 부족하다: positive 기본 shape는 기존에 있었고, 정확한 하한·초과·중첩
   negative를 추가할 가치가 있다.
4. `-1280`은 단위와 선택 근거가 드러나지 않는 magic number다: 유효.

문서 보정도 함께 반영한다. 날짜 gate는 마지막 점이 없는 `YYYY. M. D`도 포함하고, corpus 수치는
`StructureMode::Clause` 명시 결과이며, `auto` 정책 자체는 그대로지만 auto가 clause를 선택한 문서는
분류 결과 변화를 상속한다.

## 2. red와 기각한 첫 보정

기존 8개에 다음을 추가했다.

- `3.5퍼센트`, `0.5퍼센트`, `3.14 이하`, `2.1항의 규정` 다음의 정상 `3.`/`4.` 회복
- `····`, `...`, `…` dotted leader와 쪽번호를 가진 `목` 후보 거부
- `indent=-1280` 경계 허용과 `-1281`, nonzero margin, nested level 거부·body 보존

보정 전 결과는 11개 중 9 passed / 2 failed였다. shape 경계는 기존 구현이 이미 만족했고,
복합 번호 뒤 회복과 dotted-leader TOC가 red였다.

최초 구현은 만료 anchor의 경계 앞 번호 또는 직전 정상 번호의 다음 번호가 나타나면 거리와 무관하게
복귀시켰다. 이 안은 Oracle sample10 세 변형에서 node 8→1,145, `호` 4→1,141로 대량 오탐을
되살렸다. 문단 2300의 `4.1 create...` 뒤에는 설명 문단과 SQL 단계가 이어져 번호 일치만으로는
정상 목록과 stale anchor를 구분할 수 없었다. 이 설계는 제품 후보에서 제거했다.

## 3. 최종 보정

- `ExpiredHoAnchor`에 경계 위치를 보관하고 같은 section의 바로 다음 문단에만 정상 목록 복귀를
  허용한다.
- 후보 번호는 복합 번호의 앞 번호 또는 경계 전 마지막 정상 `호`의 다음 번호여야 한다.
- dotted leader는 `.`, `·`를 1점, `‥`를 2점, `…`를 3점으로 계산하며, 공백을 허용하되 합계
  3점 이상 뒤에 숫자 쪽번호가 있을 때만 TOC tail로 거부한다.
- direct `목` indent 하한은 `DIRECT_MOK_MIN_INDENT_HWPUNIT`로 명명하고 `-1280` HWPUNIT
  (약 -4.52 mm), corpus에서 확인한 경계라는 근거를 주석으로 남겼다.

## 4. corpus 무회귀

기준 `ce6a23bca`와 최종 보정을 분리 checkout·Cargo target으로 실행했다.

| 항목 | 기준 | 보정 |
| --- | ---: | ---: |
| recursive 후보 | 673 | 673 |
| parse 성공 | 670 | 670 |
| 기존 password parse 실패 | 3 | 3 |
| 파일별 node/kind 차이 | 0 | 0 |

즉 synthetic으로 추가된 정상 회복·TOC negative는 고정하면서 기존 sample corpus의 explicit clause
출력은 바꾸지 않았다.

## 5. 검증

보정 후보 `ce6a23bca` 기준으로 다음을 통과했다.

- structure unit: 8 passed
- #3744: 11 passed
- #3693: 3 passed
- #3695: 13 passed
- CLI `export_structure_`: 4 passed
- `cargo test --profile release-test --tests`: lib 3,200 passed / 7 ignored, 전체 target exit 0
- `cargo fmt --check`, `git diff --check`, all-targets clippy `-D warnings`: 통과

이후 새 `upstream/devel` `301d0fe5f`를 충돌 없이 결합했다. #3744 제품·테스트 파일에는 upstream
변경이 없었고, 결합 head에서 위 focused 5개 gate를 다시 통과했다. push된 최종 head의 전체 결합
판정은 GitHub Actions를 merge gate로 사용한다.

변경은 structure query 분류와 회귀·문서에 한정되어 renderer/layout/paint·fixture를 바꾸지 않으므로
시각 검증 대상이 아니다.

## 6. 승인 경계

작업지시자가 보정 커밋과 collaborator self-merge review 문서 커밋의 push를 승인했다. GitHub
comment/review, ready 전환, merge, issue close는 이번 범위가 아니다. push와 최신 CI 시작 상태 확인 뒤
작업지시자의 merge 지점에서 중지한다.
