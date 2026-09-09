# task_m100_2551 처리결과 보고서 — dump-pages 인자 검증을 형제 명령과 정합

- **이슈**: [#2551](https://github.com/edwardkim/rhwp/issues/2551)
- **브랜치**: `task/m100-2551-dump-pages-args` (base `devel` @ `3c54abfd`)
- **범위**: `src/main.rs` `dump_pages` 인자 파싱 3개 결함
- **분류**: 결함 수정 (무성 오류 → 명시적 오류)

## 1. 문제

`dump-pages` 만 형제 명령들과 다르게 인자 오류를 조용히 삼켰다. 세 가지 모두 무성이라
스크립트/CI 에서 잘못된 호출을 감지할 수 없었다.

`src/main.rs:2436-2452` (수정 전)

```rust
"--page" | "-p" => {
    if i + 1 < args.len() { target_page = args[i + 1].parse().ok(); i += 2; }
    else { i += 1; }
}
_ => { i += 1; }   // 알 수 없는 옵션, 메시지 없음
```

| # | 결함 | 종전 동작 |
|---|---|---|
| 1 | 파싱 실패를 `.ok()` 로 삼킴 | `-p abc` → `None` → **문서 전체 덤프** |
| 2 | 범위 검사 없음 | `-p 999` → 빈 출력 → "쪽 없는 문서" 처럼 보임 |
| 3 | 미지 옵션 무시 | `--respect-vpos-resets`(오타) 조용히 버려짐 |

## 2. 분석 — 새 규약이 아니라 기존 선례로의 정합

형제 명령은 같은 자리를 이미 올바르게 처리한다. 본 수정은 **그 패턴을 그대로 미러링**한 것이며
새로운 규약을 도입하지 않는다.

| 항목 | 선례 |
|---|---|
| 파싱 실패 오류 | `export_svg` `main.rs:337-350`, `export_png` `:995-1008`, `export_text` `:1622-1636` |
| 범위 검사 | `export_svg` `main.rs:520-527` |
| 미지 옵션 경고 | `export-svg` / `export-render-tree` |

`-p` 가 0-based 라는 기존 계약(`cli_commands.md:24`)은 유지했고, 범위 메시지도 선례와 동일하게
`(0~N-1)` 형식으로 맞췄다.

## 3. 변경

`src/main.rs` — `dump_pages` 한 함수만 수정.

1. `-p` 파싱을 `match ... { Ok/Err }` 로 바꾸고 실패 시 `eprintln!` + 조기 반환
2. `-p` 뒤 인자 누락 시에도 오류 반환
3. 문서 로드 후 `target_page >= page_count` 범위 검사 추가(`saturating_sub` 로 0쪽 문서 방어)
4. 미지 옵션 catch-all 에 `알 수 없는 옵션: {}` 경고 추가

## 4. 검증

`cargo build --bin rhwp` 통과 후 **실제 바이너리로 행위 검증**했다(정적 확인이 아님).

| 케이스 | 결과 |
|---|---|
| `dump-pages basic-table-01.hwpx -p abc` | `오류: 페이지 번호가 올바르지 않습니다.` + **출력 0줄** (종전: 전체 덤프) |
| `dump-pages … -p 999` | `오류: 페이지 번호가 범위를 벗어났습니다 (0~0)` (종전: 무성 빈 출력) |
| `dump-pages … --respect-vpos-resets` | `알 수 없는 옵션: --respect-vpos-resets` (종전: 무성) |
| `dump-pages … -p 0` (정상) | `문서 로드: … (1페이지)` — **회귀 없음** |

### 미실행 항목 (투명 고지)

- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`): 저장소 규약상
  작업지시자 별도 승인 사항이라 실행하지 않았다.
- 자동화 통합 테스트(`assert_cmd` 형태)는 추가하지 않았다. 현재 저장소에 CLI 종료코드/표준오류를
  단언하는 테스트 하네스가 없어, 하네스 도입은 범위를 넘는다고 판단했다. 위 행위 검증 결과로
  대신했으며, 하네스 추가를 원하시면 별도로 진행하겠다.

## 5. 잔여 — 같은 스윕의 별건 (본 PR 미포함)

범위를 섞지 않기 위해 분리했다. 필요하시면 각각 이슈로 등록하겠다.

1. **대부분의 명령이 치명적 오류 뒤에도 종료 코드 0** — `export-pdf` 만 `process::exit` 사용
   (`main.rs:17-22`). 매뉴얼 §3 의 종료 코드 계약과 불일치하며, `export-svg`/`export-png` 는
   쪽 렌더 실패를 경고만 하고 `내보내기 완료: N개` 를 출력한다.
2. **`export-svg --font-path` 가 파싱만 되고 무시됨** — `--profile` 경로(`main.rs:537-543`)와
   기본 경로에서 `font_paths` 가 버려진다.
3. **단일 페이지 출력 파일명 규칙 불일치** — `export-png` 는 선택 쪽 수, `export-svg`/
   `export-text` 는 문서 쪽 수를 기준으로 삼아 같은 인자에 다른 파일명이 나온다.
4. **매뉴얼 누락** — `measure-width`, `core-pages`, `export-pdf --backend/--raster-dpi` 가
   `cli_commands.md` 에 없다(문서가 "39개 전수 등재" 라고 명시).
