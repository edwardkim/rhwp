# Task #824 Stage 1 (RED) 보고서

**브랜치**: `local/task824`
**선행**: 수행계획서 + 구현계획서 승인 완료
**목표**: 결함을 객관 테스트로 입증 (FAIL 확인)

## 산출물

### 1. 회귀 테스트 신규 작성

`tests/issue_824.rs` — 2개 테스트:

| 테스트 | 대상 | 단언 | 목적 |
|---|---|---|---|
| `issue_824_embedded_picture_no_external_path` | `samples/hwp3-sample11.hwp` 첫 그림 | `external_path == None` | 결함 RED |
| `issue_824_external_picture_keeps_external_path` | `samples/hwp3-sample10.hwp` 첫 그림 | `external_path.is_some()` | 회귀 가드 |

`first_picture_external_path()` helper — 문서의 첫 `Control::Picture` 의 `external_path` 추출.

### 2. fixture 추가

| 파일 | 크기 | 용도 |
|---|---:|---|
| `samples/hwp3-sample11.hwp` | 391 KB | 본 결함 재현 (HWP3 임베디드 그림) |
| `samples/hwp3-sample11-hwp5.hwp` | 587 KB | 한컴오피스 2022 변환본 (참고) |
| `samples/hwp3-sample11-hwpx.hwpx` | 548 KB | 한컴오피스 2022 변환본 (참고) |
| `pdf/hwp3-sample11-hwpx-2022.pdf` | 27 MB | 한컴오피스 2022 PDF 권위 자료 |

> PDF 27 MB 는 다른 hwp3 sample PDF (~1 MB) 대비 큰 편이지만 `.gitattributes` 의 50 MB LFS 임계 미만 — 일반 git 추적 적용.

## 검증 결과

```
$ cargo test --test issue_824
running 2 tests
test issue_824_embedded_picture_no_external_path ... FAILED
test issue_824_external_picture_keeps_external_path ... ok

failures:
---- issue_824_embedded_picture_no_external_path stdout ----
thread 'issue_824_embedded_picture_no_external_path' panicked at tests/issue_824.rs:36:5:
임베디드 그림(pic_type=2)은 external_path 가 None 이어야 함
(현행 결함: Some("E$$00000.jpg")). got: Some("E$$00000.jpg")

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured
```

**해석**:
- ✅ 결함 RED 정확히 잡힘 — `Some("E$$00000.jpg")` 출력 확인 (임베디드 그림 내부 참조명)
- ✅ 회귀 가드 PASS — sample10 의 외부 file path 그림은 정상으로 판정 (Stage 2 GREEN 후에도 유지되어야 함)

## 다음 단계

Stage 2 (GREEN) — `src/parser/hwp3/mod.rs:935-952` 의 `pic_type` 분기 수정 → embedded 테스트 PASS 전환.
