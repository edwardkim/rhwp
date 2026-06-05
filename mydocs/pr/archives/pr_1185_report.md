# PR #1185 처리 보고서 — Task #1143: external image bytes injection contract

- **작성일**: 2026-05-31
- **PR**: #1185 → **MERGED** (devel, 머지커밋 `171bc745`)
- **컨트리뷰터**: @postmelee (핵심 컨트리뷰터)
- **연결 이슈**: #1143 (관련, Parent #1141, Follow-up #1142/PR #1175) — 자동 클로즈 아님(Closes 미명시)
- **판단**: **머지** ✅

## 결정 사유

#1142(PR #1175, discovery contract)의 후속으로, discovery key 로 external image bytes 를
core document state 에 주입하는 contract 를 고정. 신규 opt-in API(`injectExternalImageByKey`)
추가 + 기존 basename API 하위호환 보존 + 동작 보존 리팩토링. 위험 낮음. 검증 통과.

## 변경 요약 (9 파일, +727 / −119)

| 파일 | 변경 |
|------|------|
| `src/model/document.rs` | helper 3개(`external_image_loaded`/`inject_external_image_data`/`update_external_image_display_path`), `Vec→BTreeMap` 중복 로드 방지 |
| `src/wasm_api.rs` | **신규 `injectExternalImageByKey(key,data,path)`**, basename API 하위호환, 캐시 무효화 |
| `src/renderer/canvaskit_policy.rs` | `image_has_replayable_payload` 신규, external_path 아닌 실제 payload 기준 direct replay 판단, `injectedImageData`/`missingImageData` 구분 |
| `document_core/commands/document.rs` + layout 4파일 | cache 무효화 + `external_path` → ImageNode 전달 (4 picture 경로 일관) |
| `tests/issue_1143.rs` (+515 신규) | 회귀 9건 (HWP3/HWP5/HWPX sample10 key injection roundtrip, 기존 fixture 재사용) |

## 검증 결과

| 단계 | 명령 | 결과 |
|------|------|------|
| merge | `git merge --no-ff` | ✅ CLEAN (충돌 0) |
| fmt | `cargo fmt --all --check` | ✅ clean |
| 대상 | `cargo test --test issue_1143` | ✅ 9 passed, 0 failed |
| 전체 | `cargo test --tests` | ✅ 98 스위트, 1836 passed, 0 failed |
| clippy(lib) | PR 변경 파일 clippy 항목 | ✅ 0 건 |
| clippy(--all-targets) | `cargo clippy --all-targets -- -D warnings` | ⚠️ EXIT 101 — **devel baseline 동일 실패, PR 무관** → 이슈 #1186 등록 |

## 처리 절차

1. PR 정보 확인 — MERGEABLE / **CLEAN** (head 최신). 4 stage 커밋 구조.
2. `pr_1185_review.md` 작성 → 승인.
3. 로컬 머지 시뮬레이션 브랜치에서 검증. clippy baseline 실패가 PR 무관임을 확인
   (devel 도 동일 EXIT 101, PR 변경 파일 clippy 0건).
4. 메인테이너 로컬 `--no-ff` 머지 + push (`f502c335..171bc745`). push 후 PR 자동 MERGED.
5. lint 정리 이슈 #1186 등록(작업지시자 지시). PR 댓글.

## 비고

- 검토 중 테스트 함수명 전각공백(U+3000) 의심이 있었으나 **거짓 경보**(출력 채널 누락 착시).
  실제 함수명(`issue_1143_key_injection_rejects_invalid_or_unknown_keys`)은 정상.
- #1143 은 'Closes' 미명시로 자동 클로즈 대상 아님 — Parent #1141 진행에 따라 클로즈 판단.
- @postmelee 의 이미지/PageLayerTree 영역 누적 기여(#1175/#1174/#1163/#1019)의 일부.
