# 판단 트리 — 세션인가 무상태인가

```
문서 작업인가?
├─ 아니오 → 이 스킬 범위 밖 (기여는 rhwp-contributor)
└─ 예
   ├─ 폴더/목록인가? → 무상태 배치 (hwp_scan / hwp_batch*)
   ├─ 세션에 짝이 없는 동사인가? (pdf, redact, run, ir-diff, convert)
   │     → 무상태만. 이름을 만들지 말 것
   ├─ 호출이 정확히 1회인가? → 무상태
   ├─ 같은 문서를 2회 이상 파싱하게 되는가?
   │     ├─ 예, 파일이 크거나 편집 루프다 → hwp_open 세션
   │     └─ 예외적으로 파일 두 개를 비교 → hwp_ir_diff (무상태)
   └─ --workspace 코퍼스인가? → hwp_ws_list → hwp_ws_open
```

## 편집 루프 (세션)

```
hwp_open
  → (조회) fields/tables/search/structure
  → (누적) fill_fields / replace_text / set_cell
  → (눈검증) render_page(changedPages)
  → (기록) save verify=true
  → (더 있으면) 조회부터 반복 — 핸들은 그대로
  → hwp_close
```

## 단건 채움 (무상태)

```
hwp_fields → hwp_fill_fields → hwp_export_svg 또는 hwp_verify
```

여러 편집을 한 파일에 원자적으로 묶으면 `hwp_run_plan` (세션이 아님).

## 실패 시

막히면 리소스 `rhwp://docs/agent_troubleshooting_guide.md` §14.
도구가 생각나지 않으면 `rhwp capabilities --mcp` 와 `tools/list`.
