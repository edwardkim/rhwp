# Stage 3 완료 보고서 — Task M100 #416

## 자동 검증 결과

| 항목 | 명령 | 결과 |
|------|------|------|
| 전체 lib test | `cargo test --lib` | ✅ **1023 passed**, 0 failed (1016 → +7) |
| svg_snapshot | `cargo test --test svg_snapshot` | ✅ **6/6 passed** (다른 샘플 무회귀) |
| clippy | `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |
| WASM 빌드 | `docker compose --env-file .env.docker run --rm wasm` | ✅ 1m 20s, 4,100,250 bytes (-4,672 bytes) |
| 단위 테스트 (renderer::layout::utils::) | `cargo test --lib renderer::layout::utils::` | ✅ 7/7 passed |

## 시각 검증 결과

`samples/hwpspec.hwp` 1 페이지:
- ✅ 페이지 표지 이미지 정상 (16×13 → 793×1121, BIN000C.png 1137 bytes)
- ✅ 작업지시자 시각 판정 통과

## 트러블슈팅 문서 갱신

`mydocs/troubleshootings/bin_data_id_index_mapping.md` 회귀 이력 + 추가 교훈 보강:

- **2026-04-20 회귀 origin** — Task #195 fallback 도입 시점, 결함 코드, 영향, 메인테이너 검토 누락 정황
- **2026-04-28 재정정** — 본 task 의 정정 코드 + 단위 테스트 7 개
- **추가 교훈** — 가드/fallback 추가 시 트러블슈팅 정독, PR 검토 절차 강화, 정정과 함께 단위 테스트 필수

## 시각 검증 중 발견된 별개 결함

작업지시자가 `samples/hwpspec.hwp` **20 페이지** 시각 확인 중 **이미지 이중 출력** 결함 발견:
- 같은 이미지가 y 좌표 약 2.67px 차이로 두 번 그려짐 (3 쌍, 총 6 개 image 요소)
- **devel 시점 (Task #416 정정 전)** 에도 4 개 image 가 있었음 — 본 task 의 정정과 별개로 이미 존재하던 회귀
- 본 task 의 정정으로 4 → 6 으로 늘어난 것은 부수효과로 추정 (정확 매칭 후 일부 안 그려지던 이미지가 추가로 그려짐)
- 별도 task 로 처리 — Task #416 은 본 결함 (페이지 표지) 만으로 마무리

## 다음 단계

- 최종 보고서 작성
- 이중 출력 결함 별도 이슈 등록

## 산출물

- 변경 파일: 없음 (Stage 3 는 검증만)
- 트러블슈팅 갱신: `mydocs/troubleshootings/bin_data_id_index_mapping.md`
- WASM 빌드 결과: `pkg/rhwp.js`, `pkg/rhwp_bg.wasm` (4,100,250 bytes)
- 본 보고서: `mydocs/working/task_m100_416_stage3.md`
