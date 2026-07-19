# #2353 3단계 완료보고 — 구현

- 설계안: `task_m100_2353_stage2.md` (승인됨, 폐기 3건 포함)

## 구현 내역

| 항목 | 내용 |
|------|------|
| `rhwp-studio/e2e/MANIFEST.md` | **72행 전수** (tracked 기준) — 분류·상태·용도·샘플·배선·비고. 머리에 규칙 요약(명명·폐기=삭제·로컬 전용 제외) |
| `scripts/check_e2e_manifest.py` | 양방향 대조 + 명명 정합(legacy-name 면제) + **배선 실재 검증** + 열거값 검증. 로컬 실행 원칙 |
| 개명 3건 | `debug-*.test.mjs` → `debug-*.mjs` (참조 0건 확인 후) |
| 폐기 3건 | `issue-2021-probe.mjs`·`task1315-load.check.mjs` 삭제(git rm), `kps-ai-host.test.mjs` 는 **untracked 로컬 파일**이라 로컬 삭제 |
| `e2e-cdp.md` | 목록 절(2.2)을 MANIFEST 참조로 대체 |
| npm script | `e2e:manifest-check` 등록 |

## 구현 중 발견

- **kps-ai 계열은 .gitignore 의도적 로컬 전용**(`kps-ai*.test.mjs`, 비공개
  샘플 계열) — 커밋된 적 없음. manifest 범위를 "git tracked" 로 확정하고
  머리말에 로컬 전용 제외 원칙 명기. 검사기도 git ls-files 기준.
- 검사기 첫 실행이 hold 진단 3건의 명명 위반을 잡음 → legacy-name 면제
  표기 (기존 파일 보수 원칙 그대로 작동함을 확인).

## 게이트

- `check_e2e_manifest` **이상 없음** (72/72), npm script 경유 동일
- `check_markdown_links` / `check_document_metadata` green
- 대표 e2e 3종(text-flow·undo-contracts·undo-object-selection) 0 FAIL —
  관리 변경이 실행에 무영향. 개명된 debug 스크립트 기동 확인

## 3단계 보완 (작업지시자 파싱 팁 반영)

- 파서를 느슨한 regex(중간 열 `.*` 흡수)에서 **7열 명시 파싱**으로 교체 —
  셀 strip() 후 원문 보존(`—`/`/` 기호 파괴 없음), 열 수 불일치 행은
  malformed FAIL (조용한 건너뜀 → 가짜 미등재 오류 방지)
- 결손 처리: 빈칸·`—` 를 플레이스홀더로 유연 인지 + **용도 결손 집계**
  (정보 출력, 비-FAIL) → 즉시 6건 검출·채움
- 전 열 파싱으로 열린 신규 검증: **배선 열 ↔ 실제 배선 양방향 교차 대조** —
  즉시 실불일치 1건 검출·정정 (run-render-diff: CI → npm+CI)
