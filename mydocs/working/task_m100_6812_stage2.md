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
**Stage 2 전체 완료가 아니다.** 첫 절편에서 원본 결함의 실패 시험을 고정하고,
그림의 기준 좌표 계산을 렌더 노드 생성과 무관한 공통 계산으로 분리했다.
TAC 표의 줄 이동은 아직 구현되지 않았고 핵심 실패 시험 3건이 그대로 실패한다.

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

1. 공통 좌표 계산으로 활성 어울림 점유 영역을 해석하고, 개체/문단/셀의 흐름 소유 범위를 정한다.
2. 가용 가로 구간에 TAC advance가 들어가는지 판단하고, 불가할 때 유한한 그림 경계에서
   다음 후보 줄을 찾는 규칙을 연결한다. 그림을 옮기거나 원시 저장 vpos를 대입하지 않는다.
3. `TypesetEngine`의 페이지 예산과 블록/inline 렌더가 같은 배치 결과를 소비하게 한다.
   셀 문맥·복수 개체·선후 관계를 미연결 상태로 두고 완료 판정하지 않는다.
4. 3개 RED 시험을 GREEN으로 전환하고, 충분한 옆 공간·이미 아래·다른 기준 좌표·복수
   개체·텍스트 혼재·쪽 하단·LineSeg 변형과 인접 회귀를 확대한다.
5. Stage 2 전체 결과를 보고한 뒤 Stage 3 승인 경계를 따른다.
