# Task M100 #4338 Stage 1 — R38 설치 채널 매니페스트 (scoop·homebrew·winget)

- 이슈: [#4338](https://github.com/edwardkim/rhwp/issues/4338)
- 기준 브랜치: `upstream/devel` · 작업 브랜치: `task_m100_4338`
- 작성일: 2026-08-09 KST · 상태: 문서·매니페스트 전용, 검증 완료

## 산출물

- `contrib/packaging/scoop/rhwp.json` — v0.8.2 실물 해시·`extract_dir: rhwp`
  (아카이브 구조 실측 반영)·autoupdate 절. 머지 즉시 raw URL 설치 가능.
- `contrib/packaging/homebrew/rhwp.rb` — 3자산(mac arm/intel·linux). 탭 경로는
  메인테이너 결정 사항으로 가이드 §3 에 두 안 제시.
- `contrib/packaging/winget/` 3종 — portable zip, `rhwp\rhwp.exe` → `rhwp` 별칭.
- `tools/update_channel_manifests.py` — SHA256SUMS 로 5개 매니페스트 일괄
  갱신 + `--check` 멱등 검증.
- `mydocs/manual/channel_manifests_guide.md` — 채널별 설치·제출·갱신 절차와
  **미검증 요건 정직 목록**(§5).

## 검증

- 갱신 스크립트 멱등 실증: `update_channel_manifests.py 0.8.2 SHA256SUMS.txt
  --check` → "5개 매니페스트가 v0.8.2 기준 최신" (실물 릴리스 자산 해시 대조).
- scoop JSON 파싱 유효성 확인. 문서 메타데이터·상대 링크 검사 이상 없음.
- 코드·워크플로 무변경 — Cargo·WASM·시각 검증 해당 없음. 채널 심사 세부 요건은
  제출 시점 검증(가이드 §5 명시).
