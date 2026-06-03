# Task #195 단계 6 완료보고서 — BinData 스트림 해제 인프라

> 구현계획서: [task_195_impl.md](../plans/task_195_impl.md)
> 단계: 6 / 8

## 작업 결과

### 수정 파일

- `src/parser/mod.rs`
  - `load_bin_data_content`: Storage(OLE) 타입도 로드
  - `load_bin_data_content_lenient`: 동일
  - OLE 해제 후 선두 4-byte size prefix 자동 스킵 (CFB 매직 인접 확인)

### 확장된 로직

| 이전 | 이후 |
|------|------|
| Embedding(이미지)만 로드 | Embedding + Storage(OLE) 모두 로드 |
| 확장자 없으면 "dat" 폴백 | Storage 타입은 "OLE" 폴백 |
| raw 바이트 그대로 저장 | OLE는 4-byte size prefix 제거 후 CFB 매직(`D0CF11E0...`)부터 저장 |

### 검증 (통합)

1.hwp 로드 후 `bin_data_content` 내용:
```
id=1 ext=OLE data_len=384512 head=[d0 cf 11 e0 a1 b1 1a e1 ...]
id=2 ext=OLE data_len=344064 head=[d0 cf 11 e0 a1 b1 1a e1 ...]
```
해제 + prefix 스킵 성공. 내부 CFB 바로 접근 가능.

### 테스트 결과
- `cargo build --release` OK
- `cargo test --release --lib` 878 passed 0 failed (회귀 없음)

## 커밋 대상
- src/parser/mod.rs (Storage 로드 + prefix 스킵)
- mydocs/working/task_195_stage6.md
