# Task #824 Stage 3 (회귀) 보고서

**브랜치**: `local/task824`
**선행**: Stage 2 (GREEN) 완료
**목표**: 전체 cargo test + HWP3 sample 5개 SVG 출력 회귀 zero 검증

## 자동 검증

### cargo test (release)

```
$ cargo test --release
... (모든 test 슈트 실행)
passed=1353 failed=0
```

신규 테스트 포함 모든 1353 통과, 0 실패.

### issue_824 단독 재검증

```
$ cargo test --release --test issue_824
test issue_824_embedded_picture_no_external_path ... ok
test issue_824_external_picture_keeps_external_path ... ok
test result: ok. 2 passed; 0 failed
```

### clippy

```
$ cargo clippy --release -- -D warnings
Finished `release` profile [optimized] target(s) in 7.58s
```

lib + 우리 변경 부분 clippy clean.

> `cargo clippy --all-targets --release` 시 50개 error 가 보고되나 모두 본 변경 무관 사전 존재 (예: `let _ = doc.convert_to_editable_native();` 누락 등 다른 test 파일).

## SVG 출력 회귀 검증

baseline (Stage 2 적용 전) vs patched (Stage 2 적용 후) — 보유 HWP3 sample 5개 전체 page SVG 비교.

| Sample | Pages | Diff |
|---|---:|---:|
| hwp3-sample.hwp | 16 | 0 |
| hwp3-sample4.hwp | 36 | 0 |
| hwp3-sample5.hwp | 64 | 0 |
| hwp3-sample10.hwp | 763 | 0 |
| hwp3-sample11.hwp | 152 | 0 |
| **합계** | **1031** | **0** |

**해석**: 5개 sample × 1031 page **bit-identical**. 의도한 결과:
- `external_path` 메타필드 변경은 SVG 출력 layout/렌더링에 영향 없음
- placeholder 분기는 `web_canvas.rs` (Canvas) 측에만 존재, native SVG 출력 경로에 placeholder 코드 없음
- 따라서 native SVG 회귀 zero 가 정확한 결과

### 의도 효과 검증 위치

| 효과 | 검증 위치 | 단계 |
|---|---|---|
| IR `external_path` None 보장 | `cargo test issue_824` | Stage 2 ✅ |
| Canvas placeholder 미발현 | rhwp-studio 시각 검증 | Stage 4 |
| 그림 속성 dialog 한컴 정합 | rhwp-studio 시각 검증 | Stage 4 |

## 작업트리 정리

Stage 3 진행 중 stash pop 사고로 무관 stash (`pre-sync-cleanup`) 가 자동 적용되어 200+ 파일에 conflict marker 가 잠시 발생. `git reset --hard HEAD` 로 즉시 복구 (Stage 1+2 commit 안전). 잔여 untracked `_Conflict` 파일 5개 정리 완료.

## 다음 단계

Stage 4 (시각 검증 + 최종 보고서) — WASM 재빌드 + rhwp-studio 작업지시자 시각 판정:
- `samples/hwp3-sample11.hwp` — 그림 속성 dialog "파일 이름 빈 값 + 문서에 포함 체크" 한컴 정합
- `samples/hwp3-sample10.hwp` — 외부 file path 그림 정상 (회귀 가드)
