# Task #1584 — Stage 3 완료보고서

**단계**: 통제 비교 검증 (채택 게이트)
**브랜치**: `local/task1584`
**바이너리**: `local/task1584` HEAD (f5d50f7e 위 빌드)

## 1. fidelity 전수 통제 비교 (hwpdocs)

| 항목 | 수정 전 (devel HEAD, fidelity9) | 수정 후 (fidelity10) |
|------|------:|------:|
| 총 파일 | 9350 | 9660 (수집 진행) |
| PASS | 9279 | 9638 |
| **IR_DIFF** | **59** | **10** |
| PARSE_FAIL | 12 | 12 (손상 다운로드, 불변) |

**공통 9350건 per-file 통제 비교**:

| 분류 | 건수 |
|------|----:|
| 개선 (IR_DIFF→PASS) | **49** |
| **회귀 (PASS→IR_DIFF)** | **0** |
| 잔존 (IR_DIFF→IR_DIFF) | 10 |
| 신규 파일(310) 중 IR_DIFF | 0 |
| **순효과 (개선−회귀)** | **+49** |

→ 채택 게이트 충족: **순효과 +49 > 0, 악화 0**.

잔존 10건 = F3 잔여(다중필드 복합슬롯) + shapeComment + ruby 엣지 — 본 타스크 범위 외(별건).

## 2. Hangul 페이지 오라클 (시각 붕괴 보조 검증)

개선 49건 중 8건 표본(seed=1), 한글 편집기 PageCount 비교:

```
8건 / OK=8 COLLAPSE=0 기타=0 (붕괴율 0%)   전부 pg 1→1
```

→ ColumnDef 복원이 페이지 레이아웃을 붕괴시키지 않음. 오히려 컨트롤 인덱스 시프트가
사라져 셀 char_shape 오매핑(숨은 바코드 가시화) 하위 증상도 동시 해소.

## 3. 회귀 가드 영속화

- 단위: `task1584_body_first_para_two_columndefs_roundtrip` (Stage 1).
- 통합: 대표 실문서 `36382399` 를 `samples/hwpx/opengov/` 고정 말뭉치에 편입,
  `tests/fixtures/opengov_snapshot.tsv` 에 PASS 등록. snapshot 테스트 통과.

## 4. 결론

본문 인라인 ColumnDef 드롭 결함 해소. 실문서 IR_DIFF 59→10(공통 기준 −49, 회귀 0).
무손실 PASS율 추가 상승. 채택.
