# Stage 3 사후 감사 보고 — Task M100 #3789: CI false-positive 경계 제거

- **일자**: 2026-08-27 KST
- **브랜치**: `task_m100_3789-render-boundary`
- **구현 commit**: `514ff74bc`
- **이슈**: [#3789](https://github.com/edwardkim/rhwp/issues/3789)
- **보고 작성 commit**: `3c509c7d1`
- **문서 성격**: Stage 4 검증 뒤 실제 결과를 대사해 작성한 사후 보고

## 구현 결과

Render Diff workflow, trusted classifier와 policy mirror에서 `src/main.rs`를 제거하고 실제 direct render
consumer인 `src/cli/commands/caption_validation.rs`를 같은 경계에 추가했다. classifier schema version은
4에서 5로 올리고 root/caption/structure 대표 fixture를 추가했다.

| 변경 경로 | Rust | Render Diff | Native Skia |
| --- | ---: | ---: | ---: |
| `src/main.rs` | true | false | false |
| `src/cli/commands/caption_validation.rs` | true | true | true |
| `src/cli/outputs/mod.rs` | true | true | true |
| `src/cli/outputs/pdf.rs` | true | true | true |
| `src/cli/outputs/raster.rs` | true | false | true |
| `src/cli/outputs/vector.rs` | true | false | false |
| `src/cli/queries/structure.rs` | true | false | false |

## 검증

- classifier·policy Node 계약: 67/67 통과
- CI workflow Python 계약: 68/68 통과
- `actionlint .github/workflows/render-diff.yml`: 통과
- #5776 output-adapter fixture와 `render=true -> workflow 추적` 불변식: 통과

## 종료 판단

root-only 변경의 Render Diff false positive를 제거하면서 실제 caption/PDF/shared renderer consumer의
positive mapping을 유지했고 세 source 목록이 같은 경계를 표현했다. 그러나 이 결과를 Stage 3 보고로
작성해 작업지시자 승인을 받은 뒤 Stage 4에 진입하지는 않았다. 전체 검증 뒤 이 문서를 `3c509c7d1`에서
사후 작성했다.
