# Task M100 #1315 — 1단계 완료 보고서

## 단계 목표

`hwpx-roundtrip` CLI 명령 추가 + `samples/hwpx/` 전수 인벤토리 측정 (serializer 본체 무수정).

## 구현 내역

| 파일 | 변경 |
|------|------|
| `src/diagnostics/hwpx_roundtrip_batch.rs` | 신규 — 단일/배치 roundtrip 실행, TSV 출력, 단위 테스트 8건 |
| `src/diagnostics/mod.rs` | 모듈 등록 1줄 |
| `src/main.rs` | `hwpx-roundtrip` dispatch 1줄 + 도움말 |
| `src/serializer/hwpx/roundtrip.rs` | `diff_documents()` `pub` 전환 (비교 로직 무수정) |

명령 사용법:

```bash
rhwp hwpx-roundtrip sample.hwpx -o output/poc/task1315/          # 단일
rhwp hwpx-roundtrip --batch samples/hwpx -o output/poc/task1315/ # 전수
```

- 파일별 측정: parse / serialize / 재parse 성공 여부, IrDiff 건수, 소요 시간
- 산출물: `{out}/{상대경로 stem}.rt.hwpx` + `{out}/inventory.tsv`
- 실패 존재 시 종료 코드 1 (회귀 검출용)

## 전수 측정 결과 (54개 파일)

| 상태 | 건수 | 비고 |
|------|------|------|
| PASS (IrDiff 0) | **52** | ref/ 4건 포함 |
| PARSE_FAIL | 1 | `hwpx-01.hwpx` |
| SERIALIZE_FAIL | 1 | `exam_social.hwpx` |
| IR_DIFF / REPARSE_FAIL | 0 | — |

### 실패 2건 분석

1. **`hwpx-01.hwpx` — PARSE_FAIL (샘플 자체 문제)**
   - 오류: `ZIP 오류: Could not find EOCD`
   - 원인: 파일 매직이 `d0cf 11e0`(OLE CFB) — **HWP5 파일이 `.hwpx` 확장자로 잘못 저장된 샘플**. `file` 판정도 "Hancom HWP v5.0". 짝 파일 `samples/hwpx/hancom-hwp/hwpx-01.hwp`와 동일 계열로 추정.
   - 분류: serializer 결함 아님 → 3단계 등급화에서 대상 제외(샘플 정비는 별도 판단 요청)

2. **`exam_social.hwpx` — SERIALIZE_FAIL**
   - 오류: `미등록 ID 참조 발견: borderFillIDRef: [31]`
   - 원본 header.xml: `<hh:borderFills itemCnt="31">`, id 1~31 정상 존재
   - 원인 추정: parser→IR→serializer 과정에서 borderFill ID 31 참조가 serializer 등록 집합에서 빠짐 (ID 매핑 경계 문제로 추정). serializer의 참조 무결성 가드는 정상 작동
   - 분류: 3단계에서 xfail 처리 + 사유 코드 기록. 수정은 본 타스크 범위 외 (별도 이슈 등록 후보)
   - 참고: 동일 문서의 1페이지 절단본 `exam_social-p1.hwpx`는 PASS

### 성능 참고

- 최장 처리: `exam_kor.hwpx` 861ms, `aift.hwpx` 348ms (release 빌드) — 전수 1회 약 2.5초
- 대형 보도자료류(2024~2025) 전부 PASS, 36~51ms

## 검증 결과

- `cargo test --lib` — 통과, 1652 passed / 0 failed / 6 ignored (신규 8건 포함)
- `cargo fmt --check` — 통과
- `cargo clippy --lib -- -D warnings` — 통과
- 전수 배치 실행 완주 — 패닉 없음

## 다음 단계

2단계 — 패키지 구조 검증기(`package_check.rs`) + 2-round 안정성 검사 추가, 인벤토리 컬럼 확장.
