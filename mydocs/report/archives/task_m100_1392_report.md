# Task M100 #1392 최종 보고서 — HWPX serializer hp:shapeComment 직렬화

- 이슈: #1392 "HWPX serializer: hp:shapeComment 미직렬화 — 도형 설명 소실"
- 마일스톤: M100 (v1.0.0), #1315 하위
- 브랜치: `local/task1392`
- 작성일: 2026-06-14

## 1. 결함과 해소 (이슈 범위 확장 — 1단계 발견)

이슈는 aift만 보고 "도형 설명 15건"이라 했으나, 전수 정밀 계수 결과 **shapeComment
229건/27파일, 4경로 중 3경로 결손**이었다.

| 경로 | 전수 | 종전 RT | 결손 축 | 해소 |
|------|------|---------|---------|------|
| pic (그림) | 148 | 0 | serializer | `write_picture` 방출 (caption 직후) |
| equation (수식) | 44 | 0 | **파서+serializer** | `parse_equation` arm 신설 + `render_equation` 방출 |
| container (묶음) | 7 | 0 | **파서+serializer** | `parse_container` arm 신설 + `write_container_close` 방출 |
| rect (사각형) | 30 | **보존** | 없음 | 무수정 (기준선 — `write_rect`가 이미 호출) |

게이트는 `description` 미비교 사각이었다 — `ObjectComment` 동승으로 해소.

## 2. 단계 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 전수 측정 (229건/4경로 중 3경로 결손 확정) + 구현계획 보정 승인 | `20407637` |
| 2 | 파서(equation/container) + serializer 4경로 + 게이트 ObjectComment + 테스트 9종 | `0f145aec` |
| 3 | 매뉴얼 갱신 + CI + 최종 보고서 | (본 커밋) |

수정 파일: `src/parser/hwpx/section.rs`, `src/serializer/hwpx/{picture,shape,section,
roundtrip}.rs` — 파서/serializer/게이트 한정, 렌더러·레이아웃 무변경.

## 3. 검증

### 3.1 전수 배치 (`output/poc/task1392/`)

- PASS 49 / **IR_DIFF 0** (게이트 동승 후에도 차이 0) / SERIALIZE_FAIL 4(#1384) /
  PARSE_FAIL 1(제외) / ROUND2_DIFF 0
- **RT 가능 파일 전수 shapeComment: 원본 119 → RT 119 (불일치 0)**
  (미복원분은 exam_kor 계열 #1384 SERIALIZE_FAIL로 RT 자체 부재 — 기존 xfail)
- baseline 4 passed — **신규 xfail 0** (1단계 예측 적중)

### 3.2 통합 테스트 (`tests/issue_1392_shape_comment_roundtrip.rs`)

aift(pic 13+container 2)/math-001(equation 44+pic 1)/온새미로(pic 31+container 1+
rect 1) — description 멀티셋 보존 + 2-round 안정, 3 passed.

### 3.3 CI급 검증 (release-test 프로필)

- `cargo test --profile release-test --tests` — **2273 passed, 0 failed** (기존 2264 + 신규 9: 게이트 4 + 방출 2 + 통합 3)
- `cargo fmt --check` 통과, clippy 경고 0

## 4. 한컴 판정 (선택)

shapeComment(도형 설명)는 편집기에서 도형 선택 시 설명으로 보이는 메타데이터로, 본문
시각 렌더에는 영향이 없다. 시각 회귀 검증은 전수 SVG 변동 없음(IR_DIFF 0·렌더러
무변경)으로 갈음. 도형 설명 표시 직접 확인이 필요하면 aift.rt를 한컴에디터에서 열어
그림 우클릭→설명을 점검할 수 있으나 필수는 아니다.

## 5. 잔존 한계 (기지 이슈)

| 한계 | 이슈 |
|------|------|
| MEMO 필드 subList 미직렬화 | #1391 |
| 표 pageBreak 일괄 TABLE 방출 | #1393 |
| borderFillIDRef SERIALIZE_FAIL 4건 | #1384 |
| hp:pic 크기 요소 IR 미반영 | #1389 |
| 열거 속성 표면 표기 정합 검사 | #1402 |

신규 발견 없음.

## 6. 산출물

- 계획서: `mydocs/plans/task_m100_1392{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1392_stage{1,2}.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md` (게이트 항목 + #1392·#1403 해소)
- 검증 산출물: `output/poc/task1392/` (전수 rt + shapecomment_dist.tsv)
