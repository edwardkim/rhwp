# Task #1552 Stage 2 완료보고서 — C2 BinData 스트림 보존 검사

## 목표
F1(이미지 드롭) 게이트화 — 저장 시 BinData 그림 스트림 소실을 검출.

## 변경 사항 (`hwp5_roundtrip_batch.rs`)
- `bindata_fingerprint(bytes) -> Option<BTreeMap<u64,usize>>`:
  `CfbReader::open` → `list_bin_data` → 각 스트림 `read_bin_data`(raw) → `decompress_stream` 시도, 실패 시 raw.
  **decompressed 내용**의 해시 멀티셋. 이름이 아닌 내용 기준(재명명·재압축 무관). CFB 아니면 `None`.
- `bindata_lost(orig, rt)`: orig 멀티셋에서 rt 가 못 덮은 항목 수(gained 무시).
- `RoundtripRow`에 `bindata_total`/`bindata_lost` 추가, 상태 `BINDATA_LOSS`(IR_DIFF와 ROUND2 사이, 하드 실패).
- TSV 2컬럼 추가(`bindata_total`,`bindata_lost`), 콘솔 `bin_lost=n/total`, summary 1행.

## 검증 (스모크)
| 파일 | 결과 | 판정 |
|------|------|------|
| interview | `BINDATA_LOSS bin_lost=1/3` | ✅ 정탐 (olefile 일치) |
| Worldcup_FIFA2010_32 | `BINDATA_LOSS bin_lost=13/47` | ✅ 정탐 (olefile 정확 일치) |
| exam_kor | `PASS` | ✅ 무손실 재압축 오탐 없음 |
| business_overview | `PASS` | ✅ |
| **KTX** | `PASS` | ✅ (아래 정밀화 참조) |

- 단위 테스트: `cargo test --lib hwp5_roundtrip` **12 passed** (신규 `bindata_lost_counts_only_missing`, `bindata_fingerprint_preserved_on_roundtrip`).
- `cargo build --release` 성공.

## ⚠️ 정밀화 발견 — F1/F2 손실 원인 분리

선행 조사(`output/poc/fidelity/`)는 `convert` CLI로 저장본을 만들었는데, `convert`는
`serialize_document` 전에 **`convert_to_editable_native()`를 호출**한다. 본 게이트는
`serialize_document`(= `export_hwp_native`, 진짜 "저장하기")를 **직접** 호출한다.

KTX 교차 비교(olefile + 한글 PageCount):

| 경로 | KTX BinData | KTX 페이지(한글) |
|------|-------------|------------------|
| 원본 | 3 | 27 |
| `serialize_document`(본 게이트) | **3 보존** (재압축만) | **27 = 27 보존** |
| `convert`(convert_to_editable 경유) | **0 (전 소실)** | **27→1 붕괴** |

**결론**:
- **F1(이미지 드롭)은 두 원인** — ① `serialize_document` 자체(interview 1/3·Worldcup 13/47, **본 게이트가 정탐**) ② `convert_to_editable` 경로 추가 손실(KTX 3/3).
- **F2(KTX 페이지 붕괴)는 `serialize_document`가 아니라 `convert_to_editable` 경로 버그**. 네이티브 저장은 KTX를 완전 보존.
- 따라서 KTX `PASS`는 **정확한 판정**. 게이트가 손실 원인을 분리해냈다(게이트 신설의 핵심 가치).
- 선행 조사 보고서(`output/poc/fidelity/report.md`)의 KTX·convert 기반 수치는 Stage 4 최종보고에서 정정한다.
- **후속 이슈 후보 갱신**: F1(serialize_document BinData 드롭)과 **F2'(convert_to_editable 이미지·페이지 손실)**를 분리 등록.

## 다음 단계
Stage 3 — C3 페이지수 복원 + C4 CFB 구조 검사.
