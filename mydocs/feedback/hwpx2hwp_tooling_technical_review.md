# HWPX→HWP lowering 장기 작업 — 도구 개발(Tooling) 기술 검토 피드백

검토일: 2026-05-12
대상: `mydocs/troubleshootings/hwpx2hwp-rule.md` §7~§10 + #944/#946 도구 인프라
관점: 최소 1년 장기 프로젝트의 contract 추출 도구 체계
검토자: 메인테이너 (Claude)

## 0. 요약

`hwpx2hwp-rule.md` 는 방법론(§1~§5)과 금지 규칙(§7)은 견고하나, **그 방법론을
실행할 도구가 거의 없다**. §8 inventory 13 컬럼 표준, §10 #944 3-way inventory
대조는 문서로만 존재하고 이를 생성하는 CLI/도구가 부재하다. 1000+ contract unit
(§2.3)을 사람이 수작업으로 비교하면 장기 운영이 불가능하다.

핵심 결론: **문서 규칙 대비 도구 격차가 가장 큰 리스크**다. 아래 도구를 단계적으로
구축하지 않으면 §2.3 "다시 틀리지 않게 지식을 남긴다" 원칙이 실행 불가다.

## 1. 현재 도구 인벤토리 (rhwp CLI v0.7.11)

| 도구 | 용도 | HWPX→HWP lowering 적합도 |
|------|------|--------------------------|
| `dump` | IR 조판부호 구조 (ParaShape/LINE_SEG/표 attr) | △ IR 레벨만, HWP5 record tuple 아님 |
| `ir-diff` | HWPX vs HWP **IR** 차이 | △ IR 차이만, record/control contract 아님 |
| `dump-pages` | 페이지 배치 결과 | ✗ lowering 무관 |
| `diag` | 진단 | △ 범용 |
| `tools/diagnostics/hwpx_hwp_ir_diff.rs` | HWPX↔HWP IR diff | △ IR 레벨 |
| `tools/diagnostics/hwpx_roundtrip.rs` | HWPX 라운드트립 | △ 자기 검증 (한컴 oracle 아님) |
| `tests/hwpx_to_hwp_adapter.rs` | 어댑터 통합 테스트 | △ 회복 측정, record 대조 아님 |
| `diff_hwpx_vs_serializer_assumptions` | 직렬화 가정 차이 | △ 가정 차이만 |

**결정적 부재**: HWP5 **record/control tuple 레벨** inventory + oracle 3-way 대조
도구. `hwpx2hwp-rule.md` §8/§10 이 요구하는 핵심 도구가 통째로 없다.

## 2. 부족한 도구 — 우선순위순

### 2.1 [P0] `hwp5-inventory` — HWP5 record/control 인벤토리 추출기

**가장 시급.** §8 13 컬럼 표준을 기계 생성해야 한다.

```text
rhwp hwp5-inventory <file.hwp> [--format csv|jsonl] [--section N]

출력 컬럼 (§8 표준):
  sample, section, record_index, level, tag, size, owner,
  key_payload(attr/count/id/tail hex), parent_scope
```

근거: §8 표는 "최소한 다음 컬럼을 남긴다" 라고 명시하나 생성 도구 부재 →
현재는 사람이 hex 덤프를 손으로 읽어야 함. 1000 unit 운영 불가능.

요구사항:
- HWP5 CFB → BodyText/DocInfo record stream 파싱 (parser 재사용)
- record tree level/parent scope 추적 (CTRL_HEADER→LIST_HEADER→PARA_HEADER 등)
- `key_payload` 는 tag별 핵심 필드 (attr/count/id/tail) 만 hex 추출
- HWP oracle / generated 양쪽 동일 포맷 출력 → diff 가능

### 2.2 [P0] `hwp5-inventory-diff` — oracle vs generated 3-way 대조기

§10 #944 "세 inventory를 대조" 를 자동화.

```text
rhwp hwp5-inventory-diff \
  --oracle hancom_converted.hwp \
  --generated rhwp_generated.hwp \
  [--hwpx-source original.hwpx] \
  --out output/poc/<sample>/inventory_diff.csv

출력: §8 표 + mismatch(누락/추가/값다름/순서다름) + failure_class(A-F 자동 분류 힌트)
```

근거: §10 "세 inventory를 대조하여 lowering contract를 확정" — 현재 수작업.
mismatch 자동 분류 (§5 A-F) 가 핵심. "마지막 정상 출력 위치 직후 record"
(§4 파일 손상) 를 자동 표시하면 probe 시작점이 명확해진다.

### 2.3 [P1] `hwpx-control-inventory` — HWPX construct 인벤토리

§10 1번 "HWPX control inventory" 생성.

```text
rhwp hwpx-control-inventory <file.hwpx> [--format jsonl]

출력: HWPX construct id, XML path, 의미 분류, 대응 예상 HWP5 tag(추정/미정)
```

근거: 3-way 대조의 한 축. HWPX construct ↔ HWP5 tuple 대응을 추적하려면
HWPX 측 인벤토리도 동일 키로 정규화 필요.

### 2.4 [P1] `hwp5-probe-gen` — synthetic/probe HWP 생성기

§7 6번 "synthetic/probe HWP는 contract 확인용으로만 생성".

```text
rhwp hwp5-probe-gen --template <base.hwp> --mutate <contract_unit.toml>

contract_unit.toml: 단일 record/control tuple 만 변형 (1 unit 격리 검증)
```

근거: §2.3 "각 unit은 HWPX construct 하나와 HWP5 tuple 하나" — 단일 contract
격리 검증용 최소 probe. 현재는 전체 변환만 가능 → 단일 contract 분리 불가.

### 2.5 [P2] contract corpus 레지스트리 + 회귀 러너

§2.3 "모든 샘플은 regression corpus", §10 5번 "hwpx-h-01/02/03 회귀 부재".

```text
mydocs/tech/hwpx2hwp_contract_corpus/  (또는 tests/fixtures)
  <unit_id>/
    construct.txt / oracle_tuple.hex / generated_tuple.hex
    lowering_contract.md / contract_status.toml
rhwp hwp5-contract-check  → 전 unit 회귀 일괄 검증 (CI 통합)
```

근거: §2.3 "모든 실패는 contract 지식으로 남긴다", §10 "회귀를 만들지 않는다"
— 1000 unit 누적 시 회귀 자동 검증 없으면 §6 규칙들이 무한 재검증 대상이 됨
(이번 stage56/58 깨진 참조 사건이 그 전조).

## 3. 기존 도구 개선 필요

### 3.1 `ir-diff` 의 한계 명문화
`ir-diff` 는 **IR 레벨**만 비교한다. §2.4 컴파일러 비유로는 frontend/IR 일치만
확인 — target ABI(HWP5 record) 검증 불가. 도구 출력 헤더에 "이것은 IR 차이이며
HWP5 record contract 검증이 아님" 경고를 명시해야 §7 금지 규칙
("rhwp reload 성공만으로 판단 금지") 와 일관.

### 3.2 `tools/diagnostics/hwpx_roundtrip.rs` 위험성 표기
자기 라운드트립은 `feedback_self_verification_not_hancom` 위반 소지. 도구 주석에
"한컴 oracle 아님, 충분 조건 아님" 명시 + §7 금지 규칙 링크.

### 3.3 `output/poc/` 경로 정책 등록
§9 "시각 판정 파일은 항상 output/poc/ 아래" — `project_output_folder_structure`
메모리/CLAUDE.md output 폴더 표에 `output/poc/` 용도 미등록. 등록 필요
(inventory/diff 산출물도 동일 경로 규약 적용 권장).

## 4. 신규 추가 권장 — 장기 운영 인프라

### 4.1 contract unit DSL/스키마
`tools/rhwp-ingest/schema` 가 이미 존재 — 동일 패턴으로
`tools/hwpx2hwp/schema/contract_unit.schema.json` 정의. §2.3 8-필드
(construct/oracle tuple/generated tuple/lowering contract/violation/judgement/
rule/regression sample) 를 기계 검증 가능 스키마로.

### 4.2 "마지막 정상 출력 위치" 자동 탐지
§4 파일 손상 — "마지막 정상 출력 위치가 다음 probe 시작점". 한컴 판정은
수동이나, generated HWP 의 record stream 을 순회하며 **contract 위반 1st
occurrence** 를 자동 표시하는 도구가 probe 범위를 좁힌다.

### 4.3 진행 대시보드
1000 unit 중 satisfied/violated/unknown 카운트 + failure_class 분포 +
샘플별 커버리지. `scripts/dashboard.html` 패턴 재사용 가능.

## 5. 권장 구축 순서 (장기 로드맵)

```text
Phase 1 (필수 기반): hwp5-inventory + hwp5-inventory-diff (P0 2종)
  → §8/§10 자동화. 이것 없이는 1 unit 도 재현성 있게 진행 불가.

Phase 2 (대조 축 완성): hwpx-control-inventory + output/poc 정책 등록
  → 3-way 대조 가능.

Phase 3 (격리 검증): hwp5-probe-gen + contract_unit 스키마
  → 단일 contract unit 격리 + 검증.

Phase 4 (장기 운영): contract corpus 레지스트리 + 회귀 러너 + 대시보드
  → 1000 unit 누적 + 회귀 자동 차단 + 진행 가시화.
```

## 6. 핵심 리스크 (도구 부재 시)

| 리스크 | 근거 | 도구 부재 시 결과 |
|--------|------|------------------|
| contract 미검증 규칙 누적 | stage56/58 깨진 참조 사건 | §6 규칙이 무한 재검증 대상화 |
| 1 unit 수작업 비용 | §2.3 1000 unit | 장기 운영 비현실적 |
| 회귀 미탐지 | §10 5번 회귀 부재 조건 | hwpx-h-01 통과가 02/03 깨뜨림 (실제 stage58 패턴) |
| oracle/generated 비교 불가 | §7 금지 규칙 | "한컴 통과" 만 보고 contract 미설명 → 규칙 채택 |

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `project_hwpx_to_hwp_adapter_limit` | 도구 없이 어댑터만으론 한컴 호환 불가 — inventory 도구가 "완전 변환기" 의 전제 |
| `feedback_self_verification_not_hancom` | ir-diff/roundtrip 의 한계 명문화 필요 (§3.1/3.2) |
| `feedback_search_troubleshootings_first` | contract corpus 레지스트리가 이 룰의 자동화 형태 |
| `feedback_close_issue_verify_merged` | 회귀 러너 부재 → 근거 미검증 규칙 재발 (stage56/58 사건) |
| `feedback_small_batch_release_strategy` | contract unit 잘게 쪼개기(§2.3)는 이 룰과 정합 — 도구가 그 단위를 강제 |
| `project_output_folder_structure` | output/poc/ 경로 정책 미등록 (§3.3) |

## 8. 결론

- `hwpx2hwp-rule.md` 방법론은 견고하나 **실행 도구가 통째로 부재**가 최대 리스크.
- 최우선: **P0 2종 (hwp5-inventory + hwp5-inventory-diff)** — §8/§10 자동화.
  이것 없이는 §2.3 "1000 unit / 다시 틀리지 않게 지식 남김" 이 실행 불가.
- 기존 ir-diff/roundtrip 은 한계 명문화 (IR≠record contract).
- output/poc/ 경로를 폴더 정책 문서에 등록.
- 장기 로드맵 Phase 1~4 단계 구축 권장 — #946 tooling 작업의 골격으로 제안.

---

작성: 2026-05-12
