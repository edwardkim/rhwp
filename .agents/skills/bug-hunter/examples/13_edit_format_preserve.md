# 예제 — 편집 3종 형식 보존

이슈 #5324. playbook 카탈로그 · 격차 #3383. gym 아님.

## 정답지

입력이 HWPX 면 산출도 HWPX. fill-fields / replace-text / set-cell
모두. IR `--verify` 통과와 별개로 확장자가 바뀌면 격차다.

## 명령

```bash
rhwp edit fill-fields in.hwpx --data @row.json -o out.hwpx --json
rhwp info --json out.hwpx
rhwp export-hwpx out.hwpx chk.hwpx --verify --verify-pages
rhwp ir-diff in.hwpx out.hwpx --json
```

## 읽는 법

#3383 이 devel 에서 살아 있는지 파일:라인으로 확인 (F14).
살아 있으면 재현 명령을 이 템플릿으로 남긴다. 고치지 않는다 (F12).
