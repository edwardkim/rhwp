# 00 — 시각 회귀 판단 트리

이 장은 에이전트가 **어느 명령을 먼저 칠지**만 고른다. 사다리는 강제 순회가
아니다. 질문이 이미 답이면 멈춘다.

gym 경로가 아니다. 새 CLI 도 없다. 아래 상자는
`mydocs/manual/cli_commands.md` 와 레시피 06, 그리고
`src/diagnostics/render_geom_diff.rs` 가 이미 고정한 명령이다.

```
render-diff <파일> [--via hwpx|hwp] [-p N] [--max-disp PX] [--json]
  │
  ├─ 포맷 왕복만 묻는가
  │     --via hwpx (기본) 또는 --via hwp
  │     PASS → 끝 (F01)
  │
render-diff <A> <B> [-p N] [--max-disp PX] [--json]
  │
  ├─ A 와 B 가 같은 경로인가
  │     항상 PASS 여야 한다 (F02). 아니면 도구 비결정성
  │
  ├─ PASS / WARN_TEXTRUN → 끝 (F01 / F12)
  ├─ STRUCT_MISMATCH → 노드 경로를 읽는다 (F03/F04)
  ├─ PAGE_MISMATCH → dump-pages (F05)
  ├─ OVER → worst_page (F06)
  └─ LOAD_FAIL → info (F07)

render-diff --batch <폴더> [-o 출력] [--via hwpx] [--json]
  │     산출: geom_inventory.tsv
  │     요약 줄만 보지 말고 행별 status (F10)

ir-diff <A> <B> --json
  │     0=동일 / 3=차이(데이터) / 1=로드 / 2=사용법 (F08)

thumbnail <파일>          저장 시점 PrvImage. 재렌더 아님 (F09)
export-png <파일> [-p N]  현재 IR 재렌더. 눈 검증 기준
```

## 축을 고르는 한 줄

| 관찰 | 축 |
| --- | --- |
| 포맷 왕복이 레이아웃을 깨나 | `render-diff <파일> --via hwpx` |
| 편집 전후 | `render-diff <전> <후>` |
| 폴더 CI | `render-diff --batch` |
| IR 구조(텍스트·표 필드) | `ir-diff --json` |
| 색·폰트 래스터 | `export-png` (render-diff 가 아님) |
| 저장 미리보기 | `thumbnail` |

## 명령 상자 (발명 금지)

살아 있는 동사는 이 넷이다.

1. `render-diff`
2. `ir-diff`
3. `thumbnail`
4. `export-png`

후속(이미 있는 명령, 이 스킬이 발명하지 않음): `export-svg --debug-overlay`,
`export-render-tree`, `dump-pages`, `info`.

없는 것: 픽셀 비교 전용 하위명령, 레이아웃 별칭, 스크린샷 비교 동사,
gym 렌더 러너. 오타 난 하위명령은 exit 2.

코어 재사용:

- 기하 = 기존 `diagnostics::render_geom_diff`
- IR = 기존 `ir-diff`
- 미리보기 = 기존 `thumbnail` (PrvImage)
- 재렌더 = 기존 `export-png` (native-skia)

## 원본 불변

비교 명령은 입력을 덮어쓰지 않는다. `--batch -o` 는 측정 TSV 만 쓴다.
편집은 다른 스킬(`rhwp-form-fill` / `rhwp-safe-edit`)이 `-o` 로 산출을
분리한 뒤에야 이 스킬이 전후를 잰다.

## 에이전트가 하지 말 것

- STRUCT 빨간불을 경로도 안 읽고 롤백
- thumbnail 을 채운 화면으로 제출
- A==A 실패를 문서 탓으로 돌림
- `--max-disp` 로 STRUCT 를 숨기려 함
- gym/ 아래에 과제를 만들기
