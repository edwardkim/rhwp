---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_6812.md
issue: 6812
last_verified: 2026-09-07
---

# #6812 Stage 2 — 엔진 규칙 구현 작업 기록

## 1. 현재 완료 범위

메인테이너가 승인한 [구현계획](../plans/task_m100_6812_impl.md)에 따라 착수했다.
**Stage 2 전체 완료가 아니다.** 본문 블록 TAC의 그림 회피·페이지 예산 연결 후, 세 번째
절편에서 후속 표 겹침·본문 넘침 두 건을 이번 구현의 회귀로 확정하고 수정했다.
#6812 집중 시험은 16건 통과했다. 본문 inline·셀 등 소비 경로 연결과 확대 검증은 남아 있다.
최신 결과는 6절이며, 2~3절과 5절은 앞선 절편의 이력이다.

| 로컬 commit | 역할 |
| --- | --- |
| `7c67bff3a` | 엔진 규칙 구현계획 승인·Stage 1 정리 |
| `e211a33f3` | 원본 fixture와 RED/보호 시험 5건 |
| `b0c292abe` | 공통 좌표 계산 분리, 기준 영역·정렬·여백 보호 시험 추가 |

원격 push·PR 생성·Stage 3 검증·Studio 서버 교체는 수행하지 않았다.

## 2. 구현에서 확인한 중요한 사실

### 2.1 원본의 겹침 허용 속성

공개 `DocumentCore::from_bytes`로 읽은 첫 문단 그림은 다음과 같다.

```text
common.attr       = 0x040a6000
treat_as_char     = false
text_wrap         = Square
text_flow         = BothSides
allow_overlap     = true
```

파서는 HWP5 공통 속성 bit 14를 `allow_overlap`으로 보존한다. 원본의 이 비트가 켜져
있는데도 메인테이너가 확인한 한컴 배치는 TAC 표를 그림 아래 줄에 놓는다.
따라서 `allow_overlap=true`를 일괄적으로 본문/TAC 줄 회피 중지 조건으로 사용하면
이 문제를 해결할 수 없다. 원본 bit를 끄거나 시험 조건을 바꿔 해결한 것처럼 만들지 않는다.
다른 floating 개체끼리의 겹침 허용 의미 전체를 이 한 사례로 확정한 것은 아니다.

### 2.2 측정·분할의 실제 운영 경로

`document_core/queries/rendering.rs`는 기본적으로 `TypesetEngine`을 사용한다.
`RHWP_USE_PAGINATOR=1`인 경우 구형 Paginator로 전환한다. `MeasuredParagraph::total_height`
주석도 운영 페이지 분할이 이 필드를 소비하지 않음을 명시한다. 따라서 구현의 필수 연결
대상은 `typeset.rs`이고, 구형 pagination 또는 문단 측정값만 변경해서는 충분하지 않다.

### 2.3 기존 공통 계산의 재사용 경계

- `LayoutFrame::carve`는 줄 높이와 배제 영역에서 가용 가로 구간을 계산한다.
- 하지만 `supports_picture_band_frame_controls`는 TAC 표를 거절하고 non-TAC 그림을
  최대 하나로 제한한다. 이 제한을 무시하고 현재 경로가 본 과제를 처리한다고 간주하지 않는다.
- `resolve_picture_exclusion`은 Paper/Page를 같은 원점으로 취급하고 여백을 그림 밖으로
  확장한다. 반면 실제 그림 출력은 Paper를 종이, Page를 본문 영역으로 구분하며, 여백이
  포함된 개체 상자를 정렬한 뒤 그 안쪽에 잉크를 배치한다(#6596).
- 첫 절편에서 실제 출력의 `compute_object_position` 계산을
  `float_placement::ObjectPlacementFrame::position`으로 분리했다. 기존 호출 인터페이스와
  좌표 결과를 유지한다. 아직 다른 배제 영역 helper를 이 결과로 전환한 것은 아니다.

## 3. focused 검증

원본 source/test를 먼저 commit하고, 다음 전용 detached worktree에서 suite를 준비했다.

- 검증 source SHA: `b0c292abe`
- worktree: `/home/edward/mygithub/rhwp-review-6812`
- 고정 target: `/home/edward/mygithub/rhwp-6812-review-target`
- 기존 #6798 review worktree·7708 서버의 산출물을 덮어쓰지 않았다.

| 검사 | 결과 |
| --- | --- |
| 초기 RED, `e211a33f3`, suite 021의 #6812 필터 | 5건 실행: 2 PASS, 3 의도한 결함 재현 실패 |
| 공통화 후, `b0c292abe`, suite 020의 #6812 필터 | 6건 실행: 3 PASS, 3 동일한 결함 재현 실패 |
| Paper/Page × 가로 3정렬 × 세로 3정렬, 비대칭 바깥 여백 | 위 6건 중 1개 시험에서 18개 조합 PASS |
| #6596 그림 바깥 여백·정렬 | suite 003, 1개 시험/실물 3건 PASS |
| `cargo fmt --all -- --check` | PASS |
| suite manifest `--check` | PASS, 1180 sources / 48 targets |
| integration 배정 정책 계약 검사 | `node --test scripts/tests/rust-test-suite-manifest.test.mjs`, 21/21 PASS |
| native Clippy `-- -D warnings` | PASS, exit 0, 1m02s |

실패 내용은 다음과 같다. 가로 교집합은 모두 약 639.667px이고 그림/표 모두 첫 쪽에
정확히 1개씩 존재함을 확인한 뒤 겹침을 검사한다.

| 입력 | 그림 y/높이(px) | 표 y(px) | 세로 교집합(px) |
| --- | --- | --- | --- |
| 원본 | 83.133 / 60.360 | 81.240 | 60.360 |
| 그림 높이 +1500 HU | 83.133 / 80.360 | 81.240 | 80.360 |
| 그림 세로 offset +750 HU | 93.133 / 60.360 | 81.240 | 60.360 |

배경 그림과 가로 영역 밖 그림의 세로 offset 변경은 표 흐름을 이동시키지 않는다(보호 시험 PASS).
변형은 `Document` clone 뒤 `set_document`로 캐시를 재구축한다. 원본 파일은 불변이다.

공통화 후 CLI로 원본 1쪽을 다시 내보낸 결과, JSON SHA-256은 이전 Stage 1과 같다.

```text
7333991996ac6e4d8b40993e62627209639f662259544b546613de718a7c3740
```

산출 위치: review worktree의 `output/6812/stage2/coordinate-baseline/render_tree_001.json`.
이 동일성은 좌표 계산 분리의 동작 보존 증거이며, 겹침 해결 증거가 아니다.

### 재실행 시 suite 이름을 고정하지 않는다

시험 추가 후 자동 배정이 021 → 020으로 바뀌었다. 기존 021에 필터를 다시 적용한
중간 실행은 0건이어서 검증으로 인정하지 않았다. 생성 파일에서 실제 배정을 다시 찾아
020에서 6건이 실행됐음을 확인했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
rg -n 'cases/issue_6812' tests/generated/regression_suite_*.rs
# 이 SHA의 배정은 regression_suite_020이다.
cargo test --locked --target-dir /home/edward/mygithub/rhwp-6812-review-target \
  --test regression_suite_020 issue_6812 -- --nocapture
```

generated suite·manifest는 검증 worktree에만 두고 commit하지 않는다.
WASM cfg Clippy·workspace/all-targets 전체 lint 묶음과 Docker WASM·시각 검증은
이번 첫 절편에서 실행하지 않았다. native Clippy 하나를 PR 제출 게이트 전체 통과로
표기하지 않으며, 미완료인 RED 상태로 push하지 않는다.

## 4. 남은 구현 순서

1. 5.4절의 두 경고는 6절에서 실패 assertion·동일 변형 대조·수정을 완료했다.
   두 시험을 유지하고, 이후 inline·셀 연결에서도 재실행한다.
2. 본문 inline TAC·텍스트 혼재 및 셀 문맥에 동일한 규칙을 연결한다. 비표 문단의
   Para 기준 그림은 실제 문단 원점 소유자를 연결한다. 원시 vpos로 추정하지 않는다.
3. 다단/zone·caption·LeftOnly/RightOnly·단보다 넓은 표·합성 LineSeg와 증분 재조판에서
   중복 적용·영역 누출·종료 보장을 확대 검증한다. 증분 수렴 비교/복사에는 배치 metadata를
   포함했지만 공개 API 기반 증분 시험은 아직 실행하지 않았다.
4. 전체 결과 보고 후 Stage 3 승인 경계를 따른다. 제출 전 전체 Rust lint 묶음과 Docker
   WASM, 정책에 따른 시각 검증을 수행한다.

## 5. 두 번째 절편 — 본문 블록 TAC 흐름 연결

### 5.1 구현과 책임

- `ObjectPlacementFrame::picture_exclusion`: paint와 같은 기준 좌표·정렬에서 그림의
  크기·바깥 여백·캡션을 포함한 Square 점유 상자를 만든다. non-Square와 TAC 그림은
  등록하지 않는다. 원본의 `allow_overlap=true`는 그대로 유지한다.
- `place_inline_box`: `LayoutFrame::carve`로 후보 줄 높이와 겹치는 점유 영역을 제외한다.
  실제 연속 가로 구간에 바깥 여백 포함 TAC advance가 들어가면 옆에 놓고, 불가하면
  그림 하단 경계로 전진한다. 파일명·빈 문단·그림 1개 조건과 저장 vpos 4810은 사용하지 않는다.
- `TypesetState`: 이미 해당 단에 배치된 선행 그림을 개체 키로 수집한다. 여러 그림과
  다른 문단의 Paper/Page 그림도 처리한다. 단을 종료하면 점유 영역을 해제한다.
- `ColumnContent::inline_placements`: 분할기가 확정한 단 상대 좌표를 렌더로 전달한다.
  렌더가 독립적으로 그림 회피를 다시 판정하지 않는다. 증분 수렴 비교와 문단 번호 재매핑에도
  이 자료를 포함한다. 기존 생성자 변경은 새 필드 기본값 추가이며 src 회귀 시험 추가가 아니다.
- `typeset_tac_table`: 회피 이동량과 측정된 표 높이·바깥 여백을 쪽 예산에 반영한다.
  새 단으로 넘어가면 이전 단 그림의 이동량을 재사용하지 않는다. 렌더는 줄 pen에서
  테두리 좌표로 변환할 때 표의 위/왼쪽 바깥 여백을 한 번 더한다.

본문 inline과 셀의 조판 소유자는 아직 연결하지 않았다. 표 없는 문단에서 Para 기준
그림의 원점이 전달되지 않으면 등록하지 않는 명시적 미연결 경계도 있다. 이는 최종 정책이나
승인 범위 축소가 아니며, 후속 절편에서 처리할 항목이다.

### 5.2 원본에서 확인한 변화

CLI render tree 산출물은 검증 worktree의
`output/6812/stage2/flow-placement/render_tree_001.json`이다. JSON 표시는 0.1px로 반올림되므로
첫 표의 정밀 좌표와 교집합은 공개 API 시험의 `BoundingBox`로 판정했다.

| 항목 | 이전 y(px) | 이번 y(px) |
| --- | --- | --- |
| 첫 그림 | 83.133333 | 83.133333 — 불변 |
| 첫 TAC 표 | 81.240000 | 145.373333 |
| 다음 제목 표(pi 1) | 239.7 | 303.9 |
| 본문 첫 줄(pi 3) | 380.7 | 444.8 |

첫 그림 하단 143.493333px + 표 위 바깥 여백 1.88px = 표 위 145.373333px이다.
그림과 표의 세로 교집합은 0이며 뒤 제목·본문도 함께 이동한다. 원본 전체 쪽 수는 11쪽을
유지한다. 이것은 geometry/흐름 증거이며 한컴과 전체 페이지의 시각 일치를 승인받았다는 뜻은 아니다.

### 5.3 시험 결과와 중간 실패의 원인

구현 commit은 `348d07da1`부터 시작했다. `c30b0b31e`에서 줄 pen/테두리 변환을 수정하고,
`5a4864409`에서 쪽 예산에 실제 표 상자 높이를 반영했다. 최종 source/test 포맷 SHA는
`be641b9cf`이며 검증 worktree와 고정 target은 3절과 동일하다.

| 검사 | 결과 |
| --- | --- |
| #6812 focused | 14/14 PASS — 원본·높이/offset·작은 표 옆 배치·이미 아래·non-Square·영역 밖·LineSeg 없음·기준 좌표·복수 그림·순서·쪽 경계·문단 간 그림 |
| #6754 TAC 그림과 표의 같은 줄 | 2/2 PASS |
| #6596 그림 여백·정렬 | 1/1 PASS |
| #6104 TAC 제목/자리차지 표 | 1/1 PASS |
| #5929 표/그림과 host 줄 높이 | 서로 다른 source 2건 PASS |
| source-side unit tier 정책 | PASS — 4205 tests / 298 modules |
| 최종 `cargo fmt --all -- --check` | PASS |
| 재준비 후 suite manifest `--check` | PASS — 1180 sources / 48 targets |
| native Clippy `-- -D warnings` | PASS — exit 0, 43.14s |
| integration 배정 정책 계약 | 21/21 PASS |

최종 포맷 후 #6812는 suite 020, #6754는 007, #6596은 027, #6104는 012,
#5929의 두 source는 026/027에 배정됐다. 최종 시험 로그는 위 `flow-placement` 폴더에
`test-{suite}-{filter}.log`로 남긴다. WASM cfg·workspace/all-targets Clippy와 Docker WASM은
미실행이며, native Clippy 통과를 제출 게이트 전체 통과로 대체하지 않는다.

원본 첫 쪽 JSON SHA-256:
`91870e650f8e5bfd81e96cc7351b2ad0eaa31111d9c69d00f533ca9a4d32359d`.
고정 검증 target은 현재 약 16GB이며 새 target 경로를 추가하지 않았다.

중간 오류도 다음과 같이 구분한다.

- 첫 컴파일에서 상태 구조체의 DPI 접근 위치를 잘못 참조했고 `self.layout.dpi`로 수정했다.
- 첫 geometry 실행은 겹침 제거 후에도 5건이 실패했다. 줄 pen을 테두리 y로 그대로 전달해
  위 바깥 여백 1.88px가 빠졌기 때문이며 assertion을 완화하지 않고 변환을 고쳤다.
- 200px 본문 변형은 host 높이가 4px로 산출되어 약 155px 표가 넘쳤다. 물리 표 상자를
  fit 하한으로 반영한 뒤 2쪽 귀속과 새 쪽에서 회피 중복 가산 없음이 통과했다.
- 문단 간 시험은 구역 첫 문단의 명시적 나눔 속성까지 복제해 표가 다음 쪽에 있었다.
  후속 문단의 `column_type`/`raw_break_type`을 일반 문단으로 구성한 뒤 같은 쪽 시험이 통과했다.
- root `cargo fmt --all`은 아직 파생 harness에 연결되지 않은 integration source를 놓쳤다.
  원본에 `rustfmt`를 적용했다. source 줄 수 변경으로 suite 배정도 바뀌므로 마지막 commit에서
  다시 `--prepare` 후 `--check`하며 generated 파일은 commit하지 않는다.

### 5.4 두 번째 절편에서 남았던 경고 — 세 번째 절편에서 해결

14개 시험의 현재 assertion은 각 이름에 명시된 계약만 검사한다. 다음 stderr를 무시한
"전체 무회귀" 판정을 하지 않는다.

1. 그림/표 순서를 뒤집은 변형: 먼저 놓인 표의 y 보존은 통과하지만, 다음 문단 표가
   y=145.4로 놓여 앞 표(81.2~236.0)와 90.6px 겹친다는 경고가 있다.
2. 복수 그림 변형: 두 번째 그림 아래 첫 표를 놓는 시험은 통과하지만, 뒤 본문 pi 8의
   줄이 단 하단을 23.6px 넘는 경고가 있다.

두 항목의 수정 전 동일 변형 대조와 후속 내용 assertion을 추가해야 한다. 이 경고가
기존 문제인지 새 회귀인지 아직 확정하지 않았으며, Stage 3·push의 통과 증적으로 사용할 수 없다.
원본 첫 그림/표의 겹침 해결과 일반 규칙 전체 완성을 구분한다.

## 6. 세 번째 절편 — 후속 흐름 회귀 두 건 수정

### 6.1 동일 변형 대조와 분류

2026-09-07 메인테이너의 다음 절차 승인에 따라 5.4절 두 경고를 공개 API 회귀 시험으로
고정했다. 첫 쪽의 표 두 개가 모두 존재하면서 선행 표 하단을 침범하지 않는지 검사하고,
복수 그림 변형에서는 문단 pi 8의 세 줄이 전 페이지에 걸쳐 누락/중복 없이 존재하며
각 단의 실제 하단 안에 놓이는지 검사한다. 경고 메시지 유무 대신 RenderTree bbox를 판정한다.

| 코드 기준 | 후속 표 겹침 시험 | 후속 본문 넘침 시험 |
| --- | --- | --- |
| `b0c292abe` — 그림 회피 구현 전 + 동일한 새 시험 | PASS | PASS |
| `8980a498e` — 두 번째 절편 구현 + 새 실패 시험 | FAIL, 겹침 90.6px | FAIL, 넘침 23.6px |
| `20e40349e` — 선행 물리 하단/좌표 되감김 보호 | PASS | FAIL — host cap 문제 잔존 |
| `653d7f08d` — host cap의 물리 높이 보존 추가 | PASS | PASS |

**두 건 모두 이번 그림 회피 구현에서 발생한 회귀다.** 이 분류는 두 변형의 후속 흐름에
대한 것이며, 최초 원본의 그림/표 겹침 발생 계보를 새롭게 확정한 것은 아니다.

수정 전 대조 commit `a4589c1d588c12f5f802bcbae9847f3b0577f0cb`는 `b0c292abe` tree에
`8980a498e`의 `tests/cases/issue_6812_square_picture_tac_table.rs` blob만 얹은 로컬 검증
snapshot이다. `git diff --stat b0c292abe a4589c1d`로 test source 1개만 다른 것을 확인했다.
기존 detached review worktree와 고정 target을 재사용했고 새 작업 브랜치는 만들지 않았다.
대조 뒤 review HEAD는 수정 후보로 복귀했다. 이 snapshot을 원격 commit으로 참조하지 않는다.

### 6.2 원인과 수정 경계

1. **선행 표 하단 누락**: 저장 host 높이가 짧을 때 분할기의 cursor만으로 후보 줄을
   찾으면, 이미 놓인 표보다 위에서 새 그림 회피를 시작한다. 새 metadata가 렌더의
   정상적인 선행 표 아래 위치를 덮어써 후속 표가 올라갔다. `inline_box_flow_bottom`으로
   TAC의 측정 높이·바깥 여백에 따른 하단을 추적하고 후보 줄의 시작을 제한한다.
   그림이 실제로 겹치지 않는 후보에는 새 배치를 강제로 만들지 않는다.
2. **host cap이 물리 높이를 삭제**: 복수 그림 변형에서 표 배치 직후 높이는 282.6px인데
   저장 host의 4px cap + 회피량으로 128.1px까지 되감겼다. 렌더는 실제 표 높이를 유지해
   뒤 본문이 분할기의 fit보다 아래에 그려졌다. 해당 단에 확정된 회피 배치가 있으면 cap을
   물리 하단보다 작게 줄이지 않는다. 후속 vpos 스냅도 렌더처럼 순차 cursor를 보호한다.

물리 하단은 표 배치 전에 기록한다. 배치 중 후속 텍스트가 새 단으로 넘어가면 flush가
폐기하므로, 이전 단의 하단이 새 단으로 유출되지 않는다. 새 테스트는 `tests/cases/`에만
추가했으며 src 테스트나 generated suite·manifest·Cargo 파일을 제출 대상으로 만들지 않았다.

### 6.3 검증 증적

- 검증 source: `653d7f08d`; #6812 suite 013에서 **16/16 PASS**.
- 인접 회귀: #6754 2건, #6596 1건, #6104 1건, #5929 두 source 각 1건 — **6/6 PASS**.
- 포맷 검사와 suite manifest 검사 PASS: 1180 sources / 48 targets / 5007 static test attrs.
- native Clippy `-- -D warnings` PASS(exit 0, 28.19s), integration 배정 정책 계약 21/21 PASS.
  WASM cfg·workspace/all-targets Clippy는 제출 전 별도 게이트로 남아 있다.
- 원본 CLI는 11쪽을 유지한다. `output/6812/stage2/followup-flow/render_tree_001.json`의
  SHA-256은 `91870e650f8e5bfd81e96cc7351b2ad0eaa31111d9c69d00f533ca9a4d32359d`로 직전
  절편과 동일하다. 첫 그림·표와 후속 제목·본문의 개선 좌표를 유지했다.
- 로그는 review worktree의 `output/6812/stage2/flow-placement/`에 보존한다.
  `following-red.log`, `following-before-rule.log`, `following-text-trace.log`,
  `followup-{suite}-{filter}.log`가 각각 실패·대조·원인 추적·수정 후 검증에 해당한다.

전 페이지를 읽는 새 시험에서 7쪽(pi 70/71) 표 겹침 159.1px 경고도 관찰했다.
**그림 회피 구현 전 대조에서도 동일하게 재현**되므로 이번 두 회귀와 구분한다. 이를 없애기
위해 #6798의 코드를 이번 절편에 가져오지 않았으며, 전체 문서가 무경고라고 보고하지 않는다.

본문 inline·셀·비표 문단 Para 원점 연결, 다단/증분 재조판 등 4절의 후속 범위는 남아 있다.
Stage 3·Docker WASM·Studio 서버 교체·원격 push·PR 생성은 이번 절편에서 수행하지 않았다.

## 7. 네 번째 절편 — 본문 inline·셀의 미연결 계약 확인

### 7.1 수행 범위와 재현 입력

2026-09-07 다음 절차 승인에 따라 기존 블록 배치 결과를 본문 inline·셀 경로에 연결할 수
있는지 조사하고, 두 경로를 구분하는 실패 시험을 추가했다. 제품 코드는 이번 절편에서
수정하지 않았다. 공개 fixture의 메모리 clone만 변경하며 파일명별 분기는 추가하지 않는다.

- 본문: 첫 문단을 `앞 + TAC 표 + 뒤`로 구성한다. 선행 그림은 기존 Paper/Square 속성을
  유지하고 표는 1셀로 줄인다. 문자 offset과 control 위치를 맞추며 저장 LineSeg는 제거한다.
  공개 `is_tac_table_inline_in_para` 판정으로 블록 아닌 inline 경로의 입력임을 확인한다.
- 셀: 위 구조를 외부 표의 한 셀에 넣는다. 그림은 Para/Para 기준으로 설정하고 선행 그림과
  중첩 TAC의 문자 anchor가 각각 0/1임을 검사한다. 바깥 host는 텍스트 없이 네 extended
  control과 문단 끝에 맞는 문자 수를 사용한다. 그림·표가 각각 하나 존재하는지도 검사한다.
- 두 시험은 현재 실제 가로 교집합과 세로 교집합을 각각 검사한다. 표가 사라지거나 바깥
  여백만 무시해서 통과하지 못한다. 향후 GREEN 검증에는 앞뒤 텍스트의 보존·순서와 측정
  높이·쪽 귀속 assertion도 추가해야 한다. 이 두 RED만으로 전체 계약을 대표하지 않는다.

### 7.2 구조적 원인과 기존 보호 경계

| 경로 | 확인된 소유 경계 | 단순 표 좌표 전달이 부족한 이유 |
| --- | --- | --- |
| 본문 블록 TAC | `typeset.rs`의 PageItem 표 배치 → column metadata → block renderer | 앞 절편의 16건이 검증한 경로이며 본문 텍스트 중간의 inline 표와 다름 |
| 본문 inline | `paragraph_layout.rs::layout_inline_table_paragraph`와 일반 `layout_line`의 inline 제어 처리 | renderer가 텍스트/표 경계와 줄 높이를 별도로 결정하므로, 표 y만 옮기면 뒤 텍스트와 fit 예산이 달라짐 |
| composer | `line_breaking.rs::flow_inline_controls`가 TAC 표를 의도적으로 제외 | 표 폭을 다음 글자에 합산하면 기존 `abc + table + efg` 경계가 사라진다는 보호 주석이 있음. 제외 조건 삭제로 해결할 수 없음 |
| 그림 band | `supports_picture_band_frame_controls`는 TAC 표를 수용하지 않음 | 기존 band 지원을 모든 inline 개체의 지원으로 간주할 수 없음 |
| 셀 재조판·측정 | `recompose_cell_lines_in_frame`의 저장 줄 경로는 exclusion을 빈 배열로 전달하며 #5818 셀 계약을 명시. `measure_table_impl`은 절대 page/cell 원점 인자를 받지 않음 | 셀의 상대 폭/높이 측정과 실제 종이 위 좌표가 확정되는 시점이 다름. 본문 map을 붙이는 것만으로 셀 높이·행 분할이 함께 갱신되지 않음 |

따라서 현재 필요한 연결은 **표 한 개의 이동 좌표가 아니라, 텍스트와 표를 함께 놓은
줄 결과를 측정·페이지 분할·그리기가 공유하도록 하는 것**이다. 셀의 기존 정책을 제거하거나
표만 그린 뒤 이동시키는 우회 수정을 하지 않았다. 실제 데이터 구조와 소비자 변경은
[구현계획서 7절](../plans/task_m100_6812_impl.md#7-본문-inline셀-연결-수정안--승인됨)의 수정안이다.

### 7.3 검증과 현재 판정

- RED 시험 추가: `1e42a8928`; 셀 anchor·문자 수·교집합 판정 보완: `826f941e7`.
- source를 로컬 commit한 뒤 기존 detached review worktree를 같은 SHA로 전환하고 suite를
  다시 준비했다. 원본/파생물의 분리와 고정 target 재사용을 유지했다.
- 최종 `826f941e7`의 suite 026에서 18건 실행: **16 PASS / 2 FAIL**, exit 101,
  실행 1.62초(컴파일 6.13초). 로그는 review worktree의
  `output/6812/stage2/flow-placement/inline-cell-red-final.log`다.
  `cargo fmt --all -- --check`와 manifest `--check`는 PASS(1180 sources /
  5009 static test attrs / 48 targets). 이번 시험 추가 후 Clippy·전체 회귀·WASM은
  실행하지 않았으며, 이전 절편의 통과를 이번 SHA의 제출 게이트로 전용하지 않는다.
- 기존 16건의 PASS와 새 두 건의 RED를 구분한다. 본문은 실제 교집합 가로 133.333px /
  세로 33.333px, 셀은 가로 115.213px / 세로 33.333px로 겹쳤다. 아래 여백만의 문제가 아니다.
- 새 입력은 한컴으로 재출력한 별도 정답지가 아니라 승인한 일반 조판 규칙에 대한 합성
  계약 시험이다. 원본 PDF 대조와 동일한 수준의 증거라고 표시하지 않는다.
- 이 절편은 미연결 경로의 현재 실패를 확인한 것이다. 새 두 입력의 과거 devel 대조는
  아직 수행하지 않았으므로 최초 발생 시기나 기존 버전의 회귀 여부까지 단정하지 않는다.
- pi 8의 단 하단 초과 경고도 새 변형에서 관찰했다. 현재 두 RED는 그림/표 교집합에 관한
  assertion이므로 이 경고가 해결되었다고 주장하지 않는다. 기존 7쪽 pi 70/71 경고의 대조
  결과는 6.3절과 별개로 유지한다.

Stage 2는 진행 중이며 RED 상태를 제출 가능으로 판정하지 않는다. 수정안 승인 전에는 공통
줄 결과/소유자 확장 구현을 시작하지 않는다. Stage 3·WASM·Studio 교체·push·PR도 미수행이다.
