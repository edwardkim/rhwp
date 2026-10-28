# Task M100 #1392 구현계획서 — HWPX serializer hp:shapeComment 직렬화

- 수행계획서: `mydocs/plans/task_m100_1392.md` (승인 완료)
- 브랜치: `local/task1392`
- 작성일: 2026-06-13
- 단계: 3단계

## 0. 사전 조사 확정 사항 (코드·실물 확인 완료)

### 0.1 방출 순서 (실물 + OWPML 교차)

- aift pic 내 caption·shapeComment **공존 9건 전수가 `caption → shapeComment` 순서**
  (comment 단독 4건) — OWPML 자식 순서 문서(table.rs doc: "(caption, shapeComment, …)")와
  일치. → `write_picture`의 방출 지점은 **caption(#1403) 직후**.
- 공존 9 + 단독 4 = 13 pic 블록 vs 총 15건 — 차이 2건은 간이 블록 추출의 중첩(컨테이너
  내 pic 등) 한계로 추정, **1단계에서 정밀 계수로 해명**.

### 0.2 재사용 지점

- `write_shape_comment(w, &CommonObjAttr)`(shape.rs:688) — 빈 description 미방출 가드
  포함, 현재 rect 경로(:109)만 호출. **`pub(super)` 공유로 picture.rs에서 호출**
  (#1403의 `write_caption` 공유와 동일 패턴).
- Picture IR: `pic.common.description` (parse_picture :2182 적재) — 모델 변경 불요.

### 0.3 게이트

- `IrDifference::ObjectComment { section, paragraph, path, detail }` 신설 —
  detail은 `"expected={:?} actual={:?}"` (짧은 문자열이므로 값 직접 표기).
- 비교 지점: #1403의 Picture arm(`pic.comment`)과 `diff_shape_char_shapes`
  (도형 — description 접근자 신설, `shape_caption` 패턴) — 빈 문자열 동일 시 차이 없음.

## 1단계 — 전수 측정 + 정밀 계수

코드 수정 없음.

1. samples/hwpx 전수의 shapeComment 분포 (aift 외 보유 샘플 유무, 부모 단어 경계 분류)
2. aift 15건 정밀 계수 — 13 pic 블록과의 차이 2건 해명 (중첩 구조 확인)
3. 도형 경로 커버리지: `write_shape_comment` 호출자 전수 + legacy
   `render_common_shape_xml` 경로의 comment 방출 여부 — 공백 발견 시 범위 포함 판정
4. 게이트 동승 영향 사전 판정 (수정 후 aift 대칭 → 신규 xfail 0 예상 확정)
5. 보고: `_stage1.md` → 승인 요청

## 2단계 — serializer + 게이트 + 테스트

### 2.1 serializer

- `write_shape_comment` `pub(super)` 전환 + `write_picture`에서 caption 직후 호출.
- 1단계에서 도형 경로 공백 발견 시 (승인 범위 내) 동일 함수로 보강.

### 2.2 게이트

- `ObjectComment` variant + Display + Picture arm·`diff_shape_char_shapes`에 비교 추가
  (0.3 설계). 소비처 3곳 자동 동승 (#1380~#1403 패턴).

### 2.3 단위 테스트

- pic shapeComment 방출 + **caption 공존 시 순서**(caption 직후) 고정
- description 빈 문자열 → 미방출 (기존 동작 무변화)
- 게이트: 소실 주입 → `ObjectComment` 검출 (path·detail 고정)
- 실샘플(aift) parse→serialize→재parse — description 보존 + 게이트 0

### 2.4 보고: spot 수치 포함 `_stage2.md` → 승인 요청

## 3단계 — 전수 검증 + 문서

1. `hwpx-roundtrip --batch samples/hwpx` 전수 → `output/poc/task1392/` (aift 15건 복원 확인)
2. baseline + CI급 (release-test + fmt + clippy)
3. 매뉴얼 `hwpx_roundtrip_baseline.md` 갱신 (#1392 해소 + 게이트 항목)
4. 최종 보고서 + (시각 영향 없는 메타데이터이므로 한컴 판정은 도형 설명 표시 확인 1건 선택 제안)

## 위험 관리

| 위험 | 단계 | 대응 |
|------|------|------|
| caption 공존 순서 어긋남 | 2 | 실물 9건 전수 일치 확인 완료 + 순서 단위 테스트 고정 |
| 도형 경로 커버리지 공백 | 1·2 | 호출자 전수 측정 후 범위 판정 (임의 확대 금지) |
| description 내 XML 특수문자 | 2 | `text()` 이스케이프 경유 (기존 함수 그대로) — 테스트에 특수문자 케이스 포함 |
