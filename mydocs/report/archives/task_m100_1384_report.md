# Task M100 #1384 최종 보고서 — borderFillIDRef 등록 축 정정 (#1381 통합)

- 이슈: #1384 "HWPX serializer: borderFillIDRef 미등록 참조로 SERIALIZE_FAIL — parser→serializer ID 매핑 경계" (#1381 흡수)
- 마일스톤: M100 (v1.0.0), #1315 하위
- 브랜치: `local/task1384`
- 작성일: 2026-06-14

## 1. 이슈 통합 + 결함 본체

### 1.1 #1381 통합

#1381(exam_social 1건, 6/11 오전)과 #1384(같은 결함 4샘플 묶음, 6/11 저녁)는 동일
결함. baseline xfail 4건 사유가 전부 #1384 귀속. 작업지시자 결정(2026-06-14)으로
#1384로 통합 처리, #1381 흡수·클로즈.

### 1.2 결함 — borderFill 등록 축 off-by-one

| 경로 | 축 | 위치 |
|------|-----|------|
| 방출 | `idx + 1` (1-based) | `header.rs:300` |
| 참조 (charPr/paraPr borderFillIDRef) | 1-based 원본 보존 | parser·serializer 무변환 |
| 인라인 등록 (표/셀) | IR 값 1-based | `context.rs:135~140` (정상) |
| **doc_info 등록** | **`idx`** (0-based) ← 결함 | `context.rs:117` |

doc_info 등록만 0-based라 마지막 id(exam_social 31)가 등록 범위(0~30) 밖 →
`assert_all_refs_resolved` SERIALIZE_FAIL.

**왜 일부만 실패했나**: charShape는 원본 id가 0-based(`idx` 등록과 일치)라 통과,
borderFill만 1-based라 어긋남 — 같은 header.xml 안에서 리소스별 id 기준이 달랐다.
표면화는 doc_info charPr/paraPr의 borderFillIDRef가 인라인에 없는 id를 참조할
때만(4샘플).

## 2. 단계 요약

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 결함 축 확정 + 회귀 안전성 전수 + 1줄 가설 검증(4샘플 PASS+회귀 0) | `54c6ad8a` |
| 2 | context.rs 등록 1-based + baseline XFAIL 4건 제거 + 단위 테스트 | `96c59198` |
| 3 | 매뉴얼 + CI + 최종 보고서 | (본 커밋) |

수정 파일: `src/serializer/hwpx/context.rs`(1줄), `tests/hwpx_roundtrip_baseline.rs`
(XFAIL 정리) — serializer ID 등록 1지점.

## 3. 검증

### 3.1 전수 배치 (`output/poc/task1384/`)

| 항목 | 종전 | 수정 후 |
|------|------|--------|
| **SERIALIZE_FAIL** | 4 | **0** |
| **PASS** | 49 | **53** |
| IR_DIFF / ROUND2_DIFF | 0 | 0 (회귀 없음) |
| PARSE_FAIL | 1 (제외 hwpx-01) | 1 (동일) |

### 3.2 baseline 승격

- `cargo test --test hwpx_roundtrip_baseline` 4 passed — XFAIL **빈 배열(B=0)**.
- **samples/hwpx 54건 중 53건 A등급** (제외 1건 hwpx-01은 비-HWPX). baseline 첫 xfail 0.
- `ORACLE_UNFIT`(exam_kor 등 복합 실문서)은 A등급 승격 후에도 시각 oracle 부적합으로 유지.

### 3.3 CI급 검증 (release-test 프로필)

- `cargo test --profile release-test --tests` — **2318 passed, 0 failed** (기존 2317 + 신규 1: context borderFill 1-based 등록 가드)
- `cargo fmt --check` 통과, clippy 경고 0

## 4. 신규 발견 — numbering 동형 잠재 결함 (별도 이슈 제안)

`write_numbering`(header.rs:788)도 `id + 1`(1-based) 방출인데 등록은 `idx`(0-based)
— borderFill과 동형 불일치. **단 numbering은 `reference()` 호출이 없어**
(`unresolved` 항상 빈집합) SERIALIZE_FAIL을 안 낸다. 표면화 안 됨이라 본 타스크
범위 밖. 동일 패턴이므로 **별도 이슈 등록 완료 (#1409)** (선제 정합 — numberingIDRef 비교가
게이트에 추가되면 표면화될 수 있음).

## 5. 잔존 한계 (기지 이슈)

| 한계 | 이슈 |
|------|------|
| 표 pageBreak 일괄 TABLE 방출 | #1393 |
| hp:pic 크기 요소 IR 미반영 | #1389 |
| 열거 속성 표면 표기 정합 검사 | #1402 |
| newNum 슬롯 위치 + 143E RT 페이지 수 | #1407 |
| numbering 등록 축 잠재 불일치 | #1409 (4절) |

## 6. 산출물

- 계획서: `mydocs/plans/task_m100_1384{,_impl}.md`
- 단계별 보고서: `mydocs/working/task_m100_1384_stage2.md`
- 매뉴얼 갱신: `mydocs/manual/hwpx_roundtrip_baseline.md` (등급 현황 A=53/B=0 + #1384 해소)
- 검증 산출물: `output/poc/task1384/`

## 7. 이슈 처리

- 승인·검증 완료 후 **#1384 + #1381 동시 close** (중복 정리, 상호 참조 코멘트).
