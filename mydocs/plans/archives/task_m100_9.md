# Task #9: tac-img-02.hwpx 19쪽 pagination overflow 수정

## 수행 목표

19쪽에서 tac 표 이후 컨텐츠가 본문 영역을 넘어 배치되는 pagination 버그를 수정한다.

## 현상

- `pi=293` 표(tac=true)부터 body 하단(1046.9px)을 23~90px 초과
- typeset vs paginator 페이지 수 불일치: 67 vs 69 (2쪽 차이)

## 원인 분석

Task #8 (고정값 줄간격 TAC 표 병행 배치, 커밋 `7165229`)에서 추가된 `fixed_overlay_remaining` 로직이 직접 원인으로 추정:

```rust
// engine.rs — Task #8에서 추가
if fixed_overlay_remaining > 0.0 && !has_table {
    if is_fixed {
        st.current_height -= consumed;  // ← 높이를 빼서 겹침 처리
    }
}
```

음수 `line_spacing` TAC 표 이후 Fixed 줄간격 문단의 높이를 `current_height`에서 차감하는데, 이 차감이 과도하게 적용되어 페이지 19에서 `current_height`가 실제보다 낮아짐. 결과적으로 `pi=293` 표 도달 시 "공간이 남아있다"고 잘못 판정하여 페이지 넘김 실패.

## 구현 단계

### 1단계: 재현 및 정확한 원인 특정

- pi=290 표 배치 후 `current_height` vs `available_height` 추적
- pi=293 도달 시 flush 판정이 왜 실패하는지 로그로 확인
- `tac_table_count_for_flush` 조건 또는 높이 캡핑이 원인인지 확정

### 2단계: 수정 및 검증

- 원인에 따라 flush 조건 또는 높이 계산 수정
- tac-img-02.hwpx 19쪽 overflow 해소 확인
- 기존 테스트(`cargo test`) 통과 확인
- 다른 샘플 문서 회귀 확인

### 3단계: 완료 보고

- 최종 결과 보고서 작성

## 승인 요청

위 수행계획서를 검토 후 승인 부탁드립니다.
