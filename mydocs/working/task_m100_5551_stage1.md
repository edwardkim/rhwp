---
kind: report
status: active
canonical: mydocs/working/task_m100_5551_stage1.md
last_verified: 2026-09-11
---

# #5551 Stage 1: 캡션 소유 컨트롤 식별자 누락 분석

- Issue: [#5551](https://github.com/edwardkim/rhwp/issues/5551)
- 분석일: 2026-09-11
- 분석 기준: `upstream/devel@376c6b605c6be3b735bf6b8b9464fcd16b833a10`
- 작업 브랜치: `fix/5551-caption-owner-20260911`
- 단계 판정: **이슈 유효. 원인 분석과 로컬 보정 완료, 집중 검증 6개 통과.**
- 후속 검증: [Stage 2 전체 회귀·실제 동작 결과](task_m100_5551_stage2.md).
  전체 Rust 회귀는 통과했지만 꼬리말 그룹의 공개 `secIdx`에 내부 센티널이 노출되는
  잔여 결함을 발견했다. Stage 1의 집중 통과를 전체 해결 판정으로 확대하지 않는다.
- 작성 경위: 원인 분석과 1차 코드 보정 뒤 누락된 Stage 1 기록을 보완했다.
  이 문서가 구현 전에 작성되었거나 테스트가 이미 통과한 것으로 해석하지 않는다.

## 1. 요구사항과 현재 유효성

이슈는 `getPageRenderTree()`의 캡션 `TextLine`에서 해당 캡션을 소유한 표·그림·도형을
정확히 식별할 수 있도록 선택적 `captionOwner`를 추가하자는 요청이다.
분석 시 GitHub 상태는 OPEN이며 댓글은 없었다.

현재 기준 구현에는 `captionOwner`와 대응하는 소유 주소 전달 경로가 없다.
따라서 단순히 오래된 버전에 대한 이슈가 아니라 **현재 코드에도 남아 있는 API 정보 누락**이다.
캡션의 시각적 오배치나 본문 `pi` 자체의 오류로 범위를 바꾸지 않는다.

## 2. 원인과 코드 근거

| 위치 | 기준 구현에서 확인한 사실 | 영향 |
| --- | --- | --- |
| `src/wasm_api.rs`의 `get_page_render_tree` | 캐시된 페이지 트리의 `root.to_json()`을 반환 | 실제 소비자가 받는 JSON 경계를 특정할 수 있다. |
| `src/renderer/render_tree.rs`의 `TextLineNode` | 줄·문단 식별자는 있지만 캡션 소유자는 없다. | 캡션 줄과 원 컨트롤을 직접 연결할 수 없다. |
| 같은 파일의 `RenderNode::write_json` | `TextLine`에서 기존 `pi`만 방출 | 캡션 subList 문단 주소를 원 소유 문단 주소로 간주할 수 없다. |
| `src/renderer/layout/picture_footnote.rs`의 `layout_caption` | 캡션 문단들을 조판하지만 소유 컨트롤 주소를 인자로 받지 않는다. | 캡션 문단·줄을 만든 시점에 provenance가 유실된다. |
| `src/document_core/queries/rendering.rs`의 `get_page_control_layout_native` | 컨트롤 노드의 주소를 `secIdx`, `paraIdx`, `controlIdx`로 노출 | 캡션 소유 주소가 따라야 할 기존 주소 계약이다. |

캡션은 소유 컨트롤의 자식 노드가 아니라 같은 부모 아래의 별도 줄로 방출될 수 있다.
제목 문자열, 가까운 좌표, 출력 순서로 연결하면 같은 제목이 반복되거나 페이지가 분할될 때
오인할 수 있다. 따라서 **원 컨트롤을 레이아웃하는 호출 지점에서 주소를 전달**해야 한다.

## 3. 보정 설계

다음 메타데이터를 소유 주소가 확인되는 캡션 `TextLine`에만 추가한다.

```json
{
  "captionOwner": {
    "secIdx": 0,
    "paraIdx": 42,
    "controlIdx": 0,
    "controlKind": "table",
    "captionOrdinal": 0
  }
}
```

1. `CaptionOwner`와 `CaptionControlKind`를 명시적 타입으로 정의한다.
2. 표 전체·분할 표, 본문·인라인 그림, 도형의 캡션 호출부에서 기존 컨트롤 주소를 전달한다.
3. 캡션 문단 하나가 생성한 직접 `TextLine`들에 동일한 소유자와 `captionOrdinal`을 기록한다.
4. `captionOrdinal`은 캡션 문단 순서이며, 줄바꿈으로 늘어난 줄 번호가 아니다.
5. 캡션 내부의 다른 컨트롤이나 앞서 생성한 형제 노드까지 재귀적으로 덮어쓰지 않는다.
6. 주소 구성 요소가 하나라도 불명확하면 필드를 생략한다. 임의의 `0` 주소로 보충하지 않는다.

### 호환성 경계

- 기존 `pi`, 텍스트, bbox, 노드 계층, 줄바꿈과 페이지 배치를 변경하지 않는다.
- 일반 본문 줄에는 `captionOwner` 키 자체가 없다. `null`이나 빈 객체를 내보내지 않는다.
- `getPageRenderTree()`에 대한 선택적 확장이며 다른 레이어 트리의 schemaVersion을 올리지 않는다.
- 중첩·분할 컨트롤은 해당 컨트롤 노드에 실제 기록되는 주소와 일치해야 한다.
  주소를 확정할 수 없는 경로는 추정값을 만들지 않는 것이 우선이다.

## 4. 검증 계획과 수용 기준

| 이슈 기준 | 확인할 동작 |
| --- | --- |
| A1·A2 | 표 캡션에 `table` 소유자가 있고, 컨트롤 레이아웃의 주소와 일치한다. |
| A3 | 같은 캡션 제목을 가진 서로 다른 표의 주소가 구분된다. |
| A4 | 한 캡션 문단의 여러 출력 줄에 동일 소유자·동일 ordinal이 유지된다. |
| A5 | 여러 캡션 문단의 ordinal이 0부터 문단 순서대로 부여된다. |
| A6 | 일반 본문과 셀 본문은 캡션으로 오인되지 않는다. |
| A7 | 그림과 도형에서 각각 `image`, `shape`를 전달한다. |
| A8 | 새 필드를 제외한 기존 JSON과 렌더 출력에 변화가 없다. |
| A9 | 불완전한 소유 주소에는 필드를 생략한다. |

- 생산 코드의 `DocumentCore::build_page_render_tree`와
  `get_page_control_layout_native`를 사용해 집중 테스트한다.
- 테스트용 IR 변형은 주소·줄·문단 계약 검증을 위한 합성 입력이며 한컴 시각 정답지가 아니다.
- 기존 fixture의 렌더링 회귀와 메타데이터 확장을 구분한다.
- 신규 회귀 테스트는 `tests/cases/issue_5551_caption_owner.rs`에 둔다.
- 전용 빌드 경로는 `target/review-5551-20260911`이다. 공유 빌드 산출물은 삭제하지 않는다.

## 5. 현재 상태와 미완료 항목

- 이슈 유효성 및 주소 누락 원인은 코드로 확인했다.
- 1차 보정과 집중 테스트 소스는 로컬에 적용했다.
- 최초 작성 시 테스트는 컴파일 중이었다. 이후 집중 테스트 5개와 부분 주소 직렬화
  경계 테스트 1개가 통과했다. 전체 회귀·WASM 실행까지 완료한 의미는 아니다.
- 이슈 본문에 언급된 `fixtures/minimal-caption.hwpx`는 첨부된 원본을 확보한 상태가 아니다.
  해당 파일을 직접 재현했다고 주장하지 않는다.
- 실제 캡션 문단의 여러 줄, 여러 캡션 문단, 표·그림·도형의 종류와 소유 주소,
  다중 구역·컨트롤 없는 구역 및 SVG 출력 불변을 집중 테스트로 확인했다.
- 분할·중첩 경로에는 기존 컨트롤 노드와 같은 주소 전달을 반영했으나,
  모든 실물 문서 조합을 회귀 검증한 것은 아니다.
- 원격 push, 이슈 댓글·close, PR 생성은 수행하지 않았다.

## 6. 추가 범위: 컨트롤의 `secIdx` 누락

사용자 요청에 따라 이슈 본문의 다중 쪽 문서 `secIdx` 누락도 함께 조사·보정한다.
다음 두 현상은 구분해야 한다.

- 반환된 컨트롤 자체에 알려진 `secIdx`가 누락되는 경우는 API 정보 유실이다.
- `sectionCount`보다 `secIdx` 종류가 적다는 사실만으로 결함이라고 단정할 수 없다.
  컨트롤이 없는 구역이나 조회하지 않은 페이지의 구역은 분포에 나타나지 않는 것이 정상이다.

코드에서 확인한 직접 원인은 표·수식·그림 직렬화가 세 주소 필드를 한꺼번에 검사해,
`paraIdx` 또는 `controlIdx`가 없으면 **이미 알고 있는 `section_index`까지 버리는 것**이다.
완전한 주소의 기존 JSON은 유지하고, 부분 주소에서는 실제 알려진 `secIdx`만 보존하도록
보정한다. 누락된 구역을 페이지 번호·최댓값·0으로 추정하여 채우지 않는다.

다중 구역·다중 페이지 입력과 컨트롤 없는 구역을 포함해 올바른 분포를 검사한다.
부분 주소의 보존과 캡션 소유 주소의 완전성 요구는 별개다. `secIdx`만 있는 컨트롤에서
완전한 `captionOwner`를 만들어 내지 않는다.

## 7. 범위 밖

- 캡션 위치·높이·페이지 수를 바꾸는 레이아웃 보정.
- 텍스트·좌표를 이용한 소비자 측 소유자 추정 알고리즘.
- 관련 없는 CI·테스트 기준선 완화.

## 8. 실행 결과

다음 명령은 `CARGO_TARGET_DIR=target/review-5551-20260911`,
`CARGO_BUILD_JOBS=8` 환경에서 실행했다. 실행 시 해당 신규 테스트가 배치된 파생 suite는
`regression_suite_019`였다. 테스트 5개를 모두 추가한 뒤 manifest를 다시 준비한 현재 배치는
`regression_suite_028`이며 원본 테스트 파일은 동일하다.

```sh
cargo test --locked --profile release-test --test regression_suite_019 \
  issue_5551_caption_owner -- --test-threads=8
cargo test --locked --profile release-test --lib \
  task1280_v2_control_layout_exposes_plane_z_order_stable_index -- --test-threads=8
```

| 검증 | 실제 결과 |
| --- | --- |
| 캡션 소유자 집중 테스트 | 5 passed, 0 failed, 0 ignored |
| 부분 주소의 알려진 구역 보존 | 대상 테스트 1 passed, 0 failed |
| `cargo fmt --all -- --check` | 통과 |
| 파생 suite 재준비 후 manifest `--check` | 통과 |
| source-side unit tier `--check` | 통과, 기존 4205 tests / 298 modules 유지 |
| `git diff --check` | 통과 |
| 전체 회귀, Native Skia, WASM 실행, Clippy 전체 묶음 | 미실행 |

초기 집중 실행에서 합성 도형·그림 입력의 캡션이 방출되지 않아 2개 실패했다.
그림을 최상위 `Control::Picture`로 구성하고, 검증 대상인 본문 캡션 경로를 명시하도록
문단 기준 `TopAndBottom` 앵커를 지정한 뒤 다시 실행했다. 캡션 종류·줄 수·소유 주소
assertion을 제거하거나 성공 기준을 완화하지 않았다. 용지 기준 도형 캡션 배치 전체를
이번 테스트로 검증했다고 확대 해석하지 않는다.

부분 주소 테스트는 실제 쿼리가 읽는 렌더 트리 캐시의 그림 노드에서 `control_index`만
제거한 입력을 사용한다. 실제 `get_page_control_layout_native()` 반환값에 알려진
`secIdx`는 유지되고 모르는 `controlIdx`는 추가되지 않는지 확인했다.
원 제보자의 다중 쪽 원본 파일 자체를 확보해 동일 증상을 재현한 것은 아니다.

## 9. 다음 단계

PR 준비 시 저장소 변경 범위별 검증 게이트를 완료한다. 원 제보 문서나 동등한 추가
분할·중첩 fixture가 확보되면 해당 주소 경로를 추가 검증한다. 원인 분석·집중 검증과
아직 실행하지 않은 전체 검증을 완료 결과로 섞지 않는다.
