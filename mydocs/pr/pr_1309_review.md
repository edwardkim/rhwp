# PR #1309 리뷰 — HWPX 그림 직렬화 flip/rotation/orgSz 및 isEmbeded 보존

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1309 |
| 제목 | fix: HWPX 그림 직렬화 flip/rotation 하드코딩 및 isEmbeded 누락 수정 |
| 작성자 | wonbbnote |
| 관련 이슈 | #1269 |
| 대상 브랜치 | `devel` |
| PR head | `fix/issue-1269` / `b4abd882` |
| 검토 기준 | `local/pr1309-upstream` |

## 2. 변경 범위

변경 파일:

| 파일 | 변경 내용 |
|---|---|
| `src/serializer/hwpx/content.rs` | BinData manifest item에 `isEmbeded="1"` 추가 |
| `src/serializer/hwpx/picture.rs` | `orgSz`, `flip`, `rotationInfo`를 `ShapeComponentAttr` 실제 값으로 직렬화 |

PR diff는 두 파일 29 lines 추가, 16 lines 삭제로 매우 좁다.

## 3. 문제 구조

현재 devel의 HWPX picture serializer는 Stage 4 간이 구현 흔적이 남아 있어 다음 값을 고정값으로 출력한다.

| 항목 | 현재 devel 출력 | 문제 |
|---|---|---|
| `<hp:orgSz>` | `width="0" height="0"` | 원본 크기 손실 |
| `<hp:flip>` | `horizontal="0" vertical="0"` | 좌우/상하 뒤집기 손실 |
| `<hp:rotationInfo>` | `angle="0" centerX="0" centerY="0" rotateimage="0"` | 회전 중심 및 rotateimage 손실 |
| `content.hpf` image item | `isEmbeded` 없음 | 한컴에서 embedded image로 인식하지 못할 수 있음 |

관련 이슈 #1269의 재현 조건은 HWPX roundtrip 후 XML 비교 및 한컴 편집기 열기다. 특히 `samples/pic2.hwpx` 같은 그림 포함 HWPX에서 원본 `rotationInfo`가 직렬화 후 0으로 덮이는 문제가 핵심이다.

## 4. PR 변경 평가

PR 방향은 타당하다.

- `Picture`는 이미 `shape_attr: ShapeComponentAttr`를 가지고 있다.
- `ShapeComponentAttr`에는 `original_width`, `original_height`, `horz_flip`, `vert_flip`, `rotation_angle`, `rotation_center`, `rotate_image`가 보존되어 있다.
- HWPX parser도 `<hp:orgSz>`, `<hp:flip>`, `<hp:rotationInfo>`를 해당 필드로 파싱하고 있으므로 serializer가 같은 필드를 쓰는 것이 parser/serializer 대칭성에 맞다.
- `content.hpf`의 BinData 항목은 현재 serializer가 ZIP 내부에 직접 기록한 embedded BinData만 생성한다. 따라서 `isEmbeded="1"`을 붙이는 것은 내부 이미지 manifest 계약과 맞다.

주의할 점:

- `content.hpf`의 `isEmbeded` 철자는 한컴/HWPX 쪽에서 실제 쓰는 `isEmbeded`를 따라야 한다. `isEmbedded`로 교정하면 호환성이 깨질 수 있다.
- 향후 외부 링크 BinData까지 HWPX serializer가 지원되면 `isEmbeded="1"`을 고정하면 안 된다. 현재 serializer는 `bin_data_content` 기반 내부 ZIP entry만 쓰므로 이번 PR 범위에서는 blocker가 아니다.
- PR에는 테스트가 없다. 변경 폭은 작지만 저장 호환성 회귀를 막기 위해 통합 시 단위 테스트 보강이 필요하다.

## 5. GitHub 체크

`gh pr checks 1309 --repo edwardkim/rhwp` 확인 결과:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Analyze (rust) | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| CodeQL | pass |
| WASM Build | skipping |

PR base SHA는 현재 `origin/devel`보다 이전 시점이므로, 통합은 현재 devel 위에서 다시 적용하는 절차가 필요하다.

## 6. HWPX→HWP 교차 확인

PR #1309 자체는 HWPX→HWPX serializer(`src/serializer/hwpx/*`)를 수정한다. HWPX→HWP 저장은 별도 경로다.

경로:

```text
HWPX parse
→ DocumentCore::export_hwp_with_adapter()
→ document_core/converters/hwpx_to_hwp.rs
→ serializer/control.rs
→ HWP5 SHAPE_COMPONENT
```

대조 결과:

- `serializer/control.rs`는 이미 `ShapeComponentAttr.original_width/original_height`, `current_width/current_height`, `horz_flip/vert_flip`, `rotation_angle`, `rotation_center`, rendering matrix를 HWP5 `SHAPE_COMPONENT`로 기록한다.
- 따라서 #1309의 `orgSz/flip/rotationInfo` 보정은 HWPX 재저장 경로의 누락을 메우는 것이며, HWPX→HWP adapter 경로와 직접 충돌하지 않는다.
- HWPX→HWP 저장 스모크 테스트를 추가해 교차 확인했다.

실행 결과:

```text
cargo test --test issue_1279_picture_rotation_save -- --nocapture
4 passed
```

추가한 확인 항목:

- `samples/hwpx/ta-pic-001-r.hwpx` 로드
- `export_hwp_with_adapter()`로 HWP 저장
- 저장 HWP 재파싱
- 회전된 그림의 bbox, `curSz`, rotate-image bit, rendering matrix 보존 확인

## 7. 권장 처리

권장안: **수용**.

단, 통합 커밋에서 다음 테스트를 추가하는 것을 권장한다.

1. `src/serializer/hwpx/picture.rs` 단위 테스트
   - `shape_attr.original_width/height`가 `<hp:orgSz>`에 반영되는지 확인
   - `horz_flip/vert_flip`이 `<hp:flip>`에 반영되는지 확인
   - `rotation_angle`, `rotation_center`, `rotate_image`가 `<hp:rotationInfo>`에 반영되는지 확인
2. `src/serializer/hwpx/mod.rs` 또는 `content.rs` 단위 테스트
   - 그림 포함 HWPX serialize 결과의 `Contents/content.hpf`에 `isEmbeded="1"`이 포함되는지 확인

권장 절차:

1. `local/devel` 기준 통합 브랜치 생성
2. PR #1309 코드 변경 적용
3. 위 회귀 테스트 추가
4. `cargo fmt --all -- --check`
5. `cargo test --lib serializer::hwpx -- --nocapture`
6. 필요 시 `cargo test --test issue_1279_picture_rotation_save -- --nocapture`
7. 완료 보고서 작성 후 승인 게이트

## 8. PR 코멘트 초안

```markdown
검토했습니다. HWPX picture serializer가 `orgSz`, `flip`, `rotationInfo`를 0으로 고정 출력하던 문제를 `ShapeComponentAttr` 실제 값으로 바꾸는 방향은 타당합니다.

또한 현재 HWPX serializer는 ZIP 내부 embedded BinData만 manifest에 등록하므로 `content.hpf`의 image item에 `isEmbeded="1"`을 명시하는 것도 한컴 호환성 측면에서 맞는 처리로 보입니다.

HWPX→HWP 저장 경로도 교차 확인했습니다. 이 경로는 `export_hwp_with_adapter()`를 통해 HWP5 `SHAPE_COMPONENT`를 기록하며, 현재 devel에서 회전 그림의 bbox/curSz/rendering matrix 보존 스모크가 통과했습니다.

GitHub checks도 확인했습니다.

- Build & Test: pass
- CodeQL: pass
- Analyze jobs: pass

통합 시에는 `orgSz/flip/rotationInfo` 직렬화와 `isEmbeded="1"` manifest 출력을 회귀 테스트로 고정한 뒤 반영하겠습니다. 감사합니다.
```
