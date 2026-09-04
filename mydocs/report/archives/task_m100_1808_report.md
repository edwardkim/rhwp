# 최종 결과 보고서 — Task M100 #1808

## 이슈

HWPX→HWP 변환 시 표 셀 field_name 소실 (누름틀/필드 충실도)

## 결론

셀 LIST_HEADER 추가 바이트(raw_list_extra)의 한컴 필드 셀 계약 레이아웃을 원본
HWP5 대조(admrul_0039/0045, 필드 3개 샘플 정합)로 확정하고, 직렬화기가 모델
`field_name` 을 이 레이아웃으로 기록하도록 수정했다. HWPX 출처 셀 필드가 HWP 저장
후에도 보존된다.

## 한컴 계약 (원본 대조로 확정)

```
[0..4]   width (u32 LE)
[4..8]   ff 1b 02 01 (필드 속성 마커)
[8..12]  00 ×4
[12..15] 40 01 00
[15..17] name_len (u16 LE)
[17..]   UTF-16LE 필드 이름
[+8]     00 ×8            (총 25 + 2n 바이트)
```

필드 없는 셀 = width(4)+0×9 = 13바이트 (기존 default 동일).

## 수정 내역

| 파일 | 내용 |
|------|------|
| `src/parser/control.rs` | `parse_cell_field_name` pub(crate) 승격 (직렬화 대칭 검사용) |
| `src/serializer/control.rs` | `serialize_cell`: raw 인코딩과 모델 field_name 일치 시 원본 보존, 불일치 시 `build_cell_list_extra` 로 계약 레이아웃 재구성 (선두 4바이트 폭 참조 보존) |

회귀 테스트: `test_cell_field_name_extra_roundtrip` (serializer/control/tests.rs)

## 검증 결과

1. **seoul_0765**: HWPX→HWP 변환 후 셀 필드 40개 목록 **완전 일치** (수정 전: 전부 소실)
2. **네이티브 라운드트립**: admrul_0039 HWP5→HWP5 필드 80개 완전 일치
   (raw 일치 경로는 원본 바이트 보존 — 네이티브 무영향)
3. **기하 회귀**: seoul_0765 render-diff --via hwp PASS 0.00px 유지
4. **cargo test --release** 전수: 통과 (유일 실패 = #1775 기대 실패), 신규 테스트 포함
