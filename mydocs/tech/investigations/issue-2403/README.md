---
kind: investigation
status: active
canonical: mydocs/tech/investigations/issue-2403/README.md
last_verified: 2026-07-19
---

# #2403 Stage 1 — Provenance/Profile advisory baseline

- 기준 commit: 생성 시점 devel merge 상태 (커밋 메시지 참조) / 빌드:
  `cargo build --profile release-test`
- 생성·대조: `./scripts/advisory_snapshot.sh <dir>` → 단계 게이트마다 재생성 후
  `diff -r` (advisory — 무변동 기대, Phase P 규정 `plans/refactoring_plan_2026.md` §3·§7)

| 자산 | 파일 | 내용 |
|------|------|------|
| public Rust API 표면 | `advisory/api_surface.txt` (3,034줄) | src pub 선언 정규화 시그니처 목록 (grep 기반 결정적 추출) |
| CLI output 계약 | `advisory/cli_output.txt` | info·dump-pages × HWP5/HWP3/HWPX 대표 3샘플 |
| WASM/render-tree JSON 계약 | `advisory/render_tree_sha256.txt` | export-render-tree p0 구조 해시 3건 |

재현성: 동일 커밋에서 2회 생성 `diff -r` 바이트 동일 검증 완료 (2026-07-19).

## 1단계 재고정 (2026-07-19)

- api_surface 정규화에서 **줄번호 제외**로 스크립트 보정 (무관 필드 추가로 전
  항목이 밀리는 노이즈 — 1단계 실측) 후 baseline 재고정.
- 1단계 의도 delta (검토 완료, 추가 7건뿐 — 이동/제거 0):
  `model/provenance.rs` 신설(SourceFormat/SourceProvenance/
  LayoutCompatibilityProfile + 질의 2), `Document::layout_profile`,
  `pub mod provenance`. CLI output·render-tree 해시는 **무변동**.

## 2단계 재고정 (2026-07-19)

- 의도 delta (검토 완료): profile 질의 2 추가(hwp3_native_layout/
  hwp5_origin_hwpx), LayoutEngine setter 교체(set_hwp3_variant+set_hwpx_source
  → set_layout_profile). CLI output·render-tree 해시 **무변동**, 연결맵 414쪽
  유지, 전체 스위트 284 바이너리 0 실패.
- 계량: 소스분기 직접 참조 renderer 코어(typeset/layout/table·paragraph·shape
  _layout) 77→8 (잔존 = 값 전달 자유 함수 파라미터 2쌍 + 이력 주석 4).
  src 전체 176→105.

## 3단계 (2026-07-19)

- document_core 필드 읽기 40곳 → profile/provenance 질의 (가변 차용 겹침 31곳
  호이스팅, 루프 내 1곳 루프 밖 이동) + 자유 함수 3곳. **document_core 필드
  읽기 0 달성.**
- 규약 명문화: parser_architecture.md "소스 출처와 레이아웃 호환 정책" 절 +
  CONTRIBUTING 코드 스타일 항목.
- 게이트: 전체 스위트 0 실패, clippy 0, CLI·render-tree 무변동, **API 표면
  delta 0**, 연결맵 414 유지.
- 최종 계량: 소스분기 계열 참조 **176 → 87** — 잔존 전부 원점(파서 확정
  지점 18)·모델 shim 선언·테스트 초기화·값 전달 파라미터·주석.
