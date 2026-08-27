# Stage 1 사후 감사 보고 — Task M100 #3789: 계약과 소유 경계 고정

- **일자**: 2026-08-27 KST
- **브랜치**: `task_m100_3789-render-boundary`
- **기준 commit**: `upstream/devel` `1b91c2025`
- **이슈**: [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **계획 commit**: `fcaff2afd`
- **보고 작성 commit**: `3c509c7d1`
- **문서 성격**: 최종 검증 뒤 실제 이력을 대사해 작성한 사후 보고

## 확인한 기준선

- `src/main.rs`는 2,101줄이며 `test_caption`이 `Document::render_page_svg`를 직접 호출했다.
- `structure_json_value`는 root에 있었고 `export-structure`, batch query, MCP structure 응답이 공유했다.
- `export_structure`는 SVG·render-tree 출력과 함께 `src/cli/outputs/vector.rs`에 있었다.
- Render Diff workflow와 두 CI policy source는 `src/main.rs` 전체를 렌더 경계로 분류했다.
- #5776이 추가한 PDF/shared/raster adapter positive mapping과 전수 inventory 계약은 현행 기준선이었다.

## 고정한 계약

caption 명령은 mutation 순서, page 순회, SVG 파일명, stdout/stderr와 exit code를 유지한다. structure
이동은 JSON schema·provenance를 유지하고 모든 소비자가 하나의 query authority를 사용한다. CI는 root
파일명이 아니라 실제 caption render source만 Render Diff positive로 분류하며 #5776 mapping을 보존한다.

## 종료 판단

수행계획과 구현계획은 `fcaff2afd`로 구현 전에 고정됐고 작업지시자의 `진행해줘` 승인 뒤 Stage 2에
진입했다. 계획 작성·승인 순서는 준수했지만, 이 Stage 1 보고 자체는 당시 작성·승인되지 않았다. Stage
2·3 구현과 Stage 4 검증이 끝난 뒤 `3c509c7d1`에서 다른 Stage 보고와 함께 사후 작성됐다.
