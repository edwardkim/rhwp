# PR #428 처리 보고서

**제목**: fix: 그룹 내 그림(Picture) 직렬화 구현
**작성자**: oksure (Hyunwoo Park)
**처리 결과**: cherry-pick 머지 (1 commit)

## 처리 본질

`serialize_group_child()` 의 `ShapeObject::Picture` 분기가 빈 TODO 였던 것을 구현. 그룹 내 그림이 포함된 HWP 파일 저장 시 그림 데이터 유실 결함 정정.

작업지시자 통찰: *"이건 IR 내 시리얼라이제이션이라서 메인테이너의 판단"* — IR 시리얼라이제이션 영역은 메인테이너 직접 판단.

## cherry-pick 정합

| Commit | Cherry-picked | 영역 |
|--------|---------------|------|
| `6d93573` | `f436252` | Hyunwoo Park author 보존 |

## 변경 영역

| 파일 | 변경 |
|------|------|
| `src/serializer/control.rs` | +14 / -3 |

## 정정 영역

- **그룹 내 그림 직렬화**: `SHAPE_COMPONENT` + `SHAPE_COMPONENT_PICTURE` 레코드 추가
- **Chart / OLE 자식과 동일 패턴**
- **매직 상수 정리**: `0x24706963` → `tags::SHAPE_PICTURE_ID`

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1086 passed** ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |

## 영향

- 알고리즘 변경 없음 — 누락 영역 구현 추가만
- 기존 영역 영향 없음 — 그룹 내 Chart / OLE 자식 영역과 동일 패턴
- 회귀 위험 매우 낮음

## 다음 단계

- PR #428 댓글 + close
