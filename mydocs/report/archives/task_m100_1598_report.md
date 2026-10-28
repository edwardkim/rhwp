# 최종 결과보고서 — Task #1598

**제목**: HWPX ellipse/arc 전용 지오메트리(center/축/시작끝점) 직렬화 완성
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1598 · **브랜치**: `local/task1598`
**판정**: **채택 + merge**

## 1. 요약

#1589 페이지 붕괴 군집의 잔여 long-tail 근본을 ellipse 에 대해 통제 테스트로 확정·해소.
HWPX 파서가 ellipse/arc 전용 지오메트리(`<hc:center>/<hc:ax1>/<hc:ax2>/<hc:start1>/`
`<hc:end1>/<hc:start2>/<hc:end2>`)를 읽지 않고(`..Default::default()`) 직렬화도 드롭하던
**parser+serializer 양쪽 갭**을 수정. IR diff 게이트가 비교하지 않는 IR-invisible 결함이라
한글 오라클(시각)만 검출하던 부류.

## 2. 근본원인

- 파서 `parse_shape_object`(section.rs)의 자식 루프가 점요소를 `_ => {}` 로 폐기.
- 직렬화 `render_common_shape_xml` 도 미방출.
- 결과: 한글이 타원/호를 center/축 없이 다르게 렌더 → 누적 레이아웃 미세 변동 →
  경계 근처 문서 페이지 붕괴(예: 36385226 3→2).

## 3. 통제 검증 (한글 오라클)

| 36385226 (ellipse×9) | 한글 PageCount |
|------|------|
| orig | 3 |
| rt (수정 전) | 2 (붕괴) |
| rt + 지오메트리 주입 | 3 (해소) |
| **new-rt (#1598)** | **3 (해소, end-to-end)** |

→ ellipse 는 `treatAsChar=1` + `sz` 고정으로 bounding box 불변임에도, 지오메트리 단독으로
붕괴 해소. 한글의 비-IR 레이아웃 신호 의존성 재확인.

## 4. 변경

| 파일 | 변경 |
|------|------|
| `src/parser/hwpx/section.rs` | `parse_xy` 헬퍼 + 7개 점요소 파싱 + ellipse/arc 생성자 적재 |
| `src/serializer/hwpx/section.rs` | 디스패치 `geom_tail` 빌드 + `render_common_shape_xml` 방출 |
| `tests/issue_1598_ellipse_geometry_roundtrip.rs` | 신규 단위 게이트 |
| `samples/hwpx/opengov/36385226_…hwpx` | 가드 샘플 |
| `tests/fixtures/opengov_snapshot.tsv` | 36385226 PASS/0 행 |
| `mydocs/tech/investigations/issue-1589/hwpx_page_collapse_cluster.md` | §7 확정 기록 |

## 5. 검증 결과 (회귀 0)

- 단위 #1598 / baseline 4종 / opengov 2 / **전체 lib 1970 passed, 0 failed**.
- clippy 0 / fmt clean / IR diff=0 유지.

## 6. 보류 (별 타스크 후보)

- ellipse/arc 태그 전용 속성(intervalDirty/hasArcPr/arcType) — 붕괴 무관, 모델 미보유.
- arc start/end 점 — ArcShape 모델 미보유, 실문서 출현 시 확장.

## 7. #1589 군집 종합

ClickHere(#1595, 지배) → holdAnchorAndSO(#1594) → generic-shape 지오메트리(#1596) →
ellipse/arc 지오메트리(#1598)로 IR-invisible 직렬화 결함 4종 누적 해소. 잔여 표본 붕괴
관측 안 됨.
