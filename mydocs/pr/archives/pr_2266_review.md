# PR #2266 검토 — PDF 텍스트 임베드 옵트아웃 (planet6897, closes #2264)

- 검토일: 2026-07-14 / base: devel / 5파일 +313/−1 / CI 11 green / BEHIND
- 요지: 프로파일링으로 PDF 메모리 지배항 = `svg2pdf` 텍스트 임베드(폰트
  서브셋) 확정 → `PdfExportOptions.embed_text`(기본 true = 종전 동작) +
  CLI `--text-as-paths` 추가. 폐기 접근(시스템 폰트 가지치기)의 반증 기록 동반.

## 구조 검토

- 소스 변경은 옵션 필드 + `ConversionOptions.embed_text` 전달 + CLI 플래그
  — 최소·격리적. **svg2pdf 0.13 기본값이 embed_text=true 임을 크레이트
  소스로 확인** → 기본 경로는 구조체 수준에서 종전과 동일.
- 트레이드오프(텍스트 선택·검색 상실, 파일 크기 증가) 문서화 충실.

## 검증 (로컬 재실증)

| 게이트 | 결과 |
|--------|------|
| 전수 `--tests --no-fail-fast` / fmt / clippy | **3,157/0** / 통과 / 0 |
| `--text-as-paths` RSS (treatise 7쪽) | **124.3 → 78.0 MB** (−46MB 재현) |
| 산출 PDF 유효성 | 두 모드 모두 7쪽 유효 PDF (paths 모드 0.77→7.4MB — 명시된 트레이드오프) |
| 기본 경로 불변 | devel↔PR PDF 바이트 차이는 **같은 바이너리 2회 실행에서도 재현되는 기존 비결정**(폰트 리소스 /foN 채번)으로 판별 — PR 무관 |

## 검증 중 발견 (PR 무관, 이슈 등록 후보)

1. **export-pdf 간헐 행**: devel/PR 양쪽에서 동일 명령이 수 분 무응답
   (재시도 시 정상 수 초). WSL 시스템 폰트 스캔(/mnt/c 9p) 의심.
2. **PDF 폰트 리소스 채번 비결정**: 같은 바이너리 2회 실행에서 /foN 번호가
   달라 회귀 diff 를 방해. 채번 결정화(정렬) 후보.

## 판단

**approve → merge 수용 권고** (BEHIND — merged tree 선검증 + admin merge).
merge 시 #2264 는 devel push 워크플로로 close.
