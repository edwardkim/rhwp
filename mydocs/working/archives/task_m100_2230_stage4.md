# 4단계 완료보고서 — Task M100 #2230: 저장 왕복 + 게이트

- 이슈: #2230 / 구현계획서: `task_m100_2230_impl.md` / 브랜치: `local/task2230`
- 작성일: 2026-07-12

## 수행 내용

### 저장 왕복 표적 테스트 (+2건 = 총 6건)

`tests/issue_2230_placeholder_selection.rs`:

- `assign_then_hwpx_roundtrip_preserves_image`: 심볼 placeholder 에 그림
  지정 → `export_hwpx_native` → 재파싱 → missing 마커 부재 + image 컨트롤
  2건 유지 (HWPX BinData 매니페스트 왕복).
- `assign_then_hwp_roundtrip_preserves_image`: 동일 시나리오를
  `export_hwp_with_adapter`(HWP 5.0 CFB) 로 검증 — BIN 스트림 왕복.

두 왕복 모두 첫 실행에서 통과 — `register_embedded_bin_data` 가
insert_picture_native 의 검증된 규칙(storage id 최댓값+1)을 그대로
공유하는 설계 효과.

## 게이트 총괄

| 게이트 | 결과 |
|--------|------|
| `cargo fmt --all -- --check` | 통과 |
| clippy (release-test, all-targets) | 경고 0 |
| `cargo test --profile release-test --tests --no-fail-fast` | **3058 passed / 0 failed** (golden svg_snapshot 포함) |
| OVR 5샘플 (±2px, 샘플별 분리 폴더 `output/poc/issue2230/`) | **회귀 0건** (exam_science / issue1835 / issue1853 / pr-1674 / rowbreak) |
| studio `tsc --noEmit` / `npm run build` / `npm test` | 클린 / 성공 / 206/0 (3단계) |
| WASM 빌드 (docker) | 성공 (3m 55s, `pkg/rhwp_bg.wasm` 갱신 — `assignPictureImage` 심볼 rhwp.js/.d.ts 포함 확인) |

## 실사용 판정 요청 (작업지시자)

검증 문서: `samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물
화재발생 알림(화재번호 2026-177).hwpx`

1. studio 로 열기 → 1페이지 상단 결재 표 우측 "심볼" 칸의 placeholder
   (점선 테두리 + 그림-없음 아이콘) **클릭** → 선택 테두리+핸들 표시 확인.
2. **더블클릭** → 파일 선택 대화상자 → 이미지 선택 → placeholder 자리에
   그림 표시 확인 (개체 틀 크기 유지).
3. Ctrl+Z (undo) → placeholder 복귀 / Ctrl+Y (redo) → 그림 복귀 확인.
4. 저장(HWPX/HWP) → 재열기 → 그림 유지 확인.

> dev 서버 사용 시 재기동 + hard reload 필수 (구 번들 위음성 방지).
