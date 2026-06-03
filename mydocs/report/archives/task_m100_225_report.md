---
타스크: #225 mydocs/manual/MEMORY.md 링크 누락 및 중복 파일
브랜치: local/task225
작성일: 2026-04-23
기여자 보고: @InsuJeong496
상태: 구현 완료
---

# 최종 결과 보고서

## 1. 배경

2026-04-20 @InsuJeong496 님이 신고: `mydocs/manual/` 아래에 `MEMORY.md` 파일이 두 개 존재하며, 상위 경로의 파일(`mydocs/manual/MEMORY.md`) 은 링크가 모두 깨져 있다.

## 2. 현황 분석

3개 MEMORY 비교:

| 파일 | 줄수 | 상태 |
|------|------|------|
| `~/.claude/.../memory/MEMORY.md` | 24 | AI 메모리 원본 (오늘 AMO 항목까지 갱신됨) |
| `mydocs/manual/MEMORY.md` | 22 | 중복 · 링크 깨짐 (상대 경로인데 같은 폴더에 파일 없음) |
| `mydocs/manual/memory/MEMORY.md` | 20 | 공개 사본 · 링크 정상, 그러나 구버전 |

최근 추가된 메모리 파일 2개 (`feedback_external_docs_self_censor`, `feedback_amo_submission_gotchas`) 는 공개 사본에 부재.

기존에 이미 있던 `feedback_release_sync_check.md` · `feedback_release_manual_required.md` 는 공개 사본에 파일 자체는 존재했으나 인덱스 (MEMORY.md) 의 링크는 누락된 상태였음 — @InsuJeong496 님이 언급한 "2줄 더 작성" 부분이 정확히 이 두 항목.

## 3. 수행 내역

### 3.1 중복 파일 삭제
- `mydocs/manual/MEMORY.md` (22줄, 깨진 링크) 삭제

### 3.2 공개 사본 인덱스 갱신
- `mydocs/manual/memory/MEMORY.md` 를 `~/.claude/.../memory/MEMORY.md` 최신본으로 덮어씀
- 20행 → 24행 (4 항목 증가)

### 3.3 누락 메모리 파일 추가
- `mydocs/manual/memory/feedback_external_docs_self_censor.md` 신규 (오늘 자기검열 체크리스트)
- `mydocs/manual/memory/feedback_amo_submission_gotchas.md` 신규 (오늘 AMO 제출 함정 기록)

기존에 있던 release_sync · release_manual 은 파일 자체 존재 확인 후 재복사 생략.

### 3.4 공개 적합성 확인
추가된 2개 파일 모두 외부 공개 적합:
- `external_docs_self_censor` — 프로젝트 투명성 향상 (자기검열 체크리스트 공개 자체가 긍정적)
- `amo_submission_gotchas` — 다른 오픈소스 Firefox 확장 개발자에게 가치 있는 정보

## 4. 검증

- [x] `mydocs/manual/MEMORY.md` 삭제 확인
- [x] `mydocs/manual/memory/MEMORY.md` 가 `~/.claude/.../memory/MEMORY.md` 와 diff 0
- [x] `mydocs/manual/memory/MEMORY.md` 24 항목
- [x] 모든 링크 대상 파일이 같은 폴더에 존재

## 5. 변경 파일

| 파일 | 변경 |
|------|------|
| `mydocs/manual/MEMORY.md` | 삭제 |
| `mydocs/manual/memory/MEMORY.md` | 20→24 행 갱신 |
| `mydocs/manual/memory/feedback_external_docs_self_censor.md` | 신규 |
| `mydocs/manual/memory/feedback_amo_submission_gotchas.md` | 신규 |
| `mydocs/plans/task_m100_225.md` | 신규 (수행계획서) |
| `mydocs/report/task_m100_225_report.md` | 신규 (본 문서) |

## 6. 후속 제안 (별도 이슈)

본 수정은 **일회성 동기화**. 근본적 drift 방지책이 필요:

- **문제**: AI 메모리 시스템(`~/.claude/.../memory/`) 과 공개 사본(`mydocs/manual/memory/`) 이 서로 다른 경로라 메모리 추가/갱신 시 두 곳을 동시에 수정해야 함. 이번처럼 drift 가 재발할 수 있음.
- **권고**: AI 메모리 등록/갱신 프로토콜에 **"공개 사본 동시 갱신"** 단계 포함.
- **처리**: 별도 이슈로 등록할지 작업지시자 결정 사항.

## 7. 기여자 감사

@InsuJeong496 님의 세심한 문서 감사로 이슈가 식별되었다. 한 번 읽고 지나칠 수도 있는 링크 끊김을 주의 깊게 발견해주심.

## 8. 절차

- [x] 수행계획서 작성
- [x] 작업지시자 승인
- [x] 구현 (파일 4건 · 디렉토리 1건 · 계획서 · 보고서)
- [ ] `local/devel` merge + `devel` push
- [ ] 이슈 #225 close + 감사 코멘트
