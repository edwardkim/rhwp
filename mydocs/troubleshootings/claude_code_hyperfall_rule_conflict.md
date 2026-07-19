---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# 트러블슈팅: 클로드 코드 기본 동작과 하이퍼-워터폴 규칙 충돌

> **작성일**: 2026-04-10
> **관련 타스크**: #62 재오픈

---

## 문제 현상

클로드 코드는 기본적으로 빠른 실행과 자율성을 지향하도록 훈련되어 있다.
하이퍼-워터폴 방법론은 단계별 승인, 문서화 위치 규칙, 소스 수정 전 승인 요구 등 엄격한 절차를 요구한다.
이 두 가지가 충돌할 때 클로드 코드가 CLAUDE.md 규칙을 따르지 않는 경우가 발생한다.

### 발생 사례 (2026-04-10, #62 재오픈)

1. **최종 보고서 위치 오류**: 최종 완료보고서를 `mydocs/working/`에 생성함 → 올바른 위치는 `mydocs/report/`
2. **소스 수정 전 승인 생략**: 회귀 발생 시 승인 없이 소스를 수정하려는 경향
3. **컨텍스트 압축 후 규칙 망각**: 세션이 길어져 컨텍스트가 압축되면 CLAUDE.md 세부 규칙이 기본 행동 패턴에 밀림

### 발생 사례 (2026-04-11, #106)

4. **보고서 커밋 누락 후 merge**: 단계별 완료보고서(`_stage1.md`), 최종 결과보고서(`_report.md`), 오늘할일(`orders/`) 갱신을 타스크 브랜치에서 커밋하지 않고 merge → `devel` 브랜치에서 뒤늦게 커밋하는 비정상 흐름 발생. CLAUDE.md에 커밋 시점 규칙이 명시되지 않은 것이 원인.

---

## 원인

- 클로드 코드는 업데이트마다 기본 동작 방식이 변경됨
- 하이퍼-워터폴 규칙은 CLAUDE.md에 명시되어 있지만, 컨텍스트 압축 후 새 세션에서는 MEMORY.md 인덱스만 로드되고 세부 규칙은 직접 읽어야 함
- 클로드 코드의 훈련 방향(빠른 실행)이 하이퍼-워터폴(단계별 승인)과 근본적으로 충돌

---

## 해결 방안

### 1. MEMORY.md에 규칙 위반 사례 즉시 기록

규칙 위반이 발생할 때마다 `memory/` 폴더에 피드백 메모리를 추가하고 MEMORY.md 인덱스에 등록한다.
MEMORY.md는 매 세션마다 자동으로 로드되므로, 반복 위반을 줄이는 가장 효과적인 수단이다.

```
memory/feedback_report_location.md   — 최종 보고서는 mydocs/report/
memory/feedback_process_must_follow.md — 이슈→브랜치→할일→계획서→구현 순서
memory/feedback_no_close_without_approval.md — 이슈 클로즈는 승인 필수
```

### 2. 문서 생성 전 CLAUDE.md 확인 습관

문서를 생성하기 전에 반드시 CLAUDE.md의 문서 폴더 구조와 파일명 규칙을 확인한다.

```
mydocs/
  orders/    — 오늘 할일 (yyyymmdd.md)
  plans/     — 수행계획서, 구현계획서
  working/   — 단계별 완료보고서 (_stage{N}.md)
  report/    — 최종 완료보고서 (_report.md)  ← 혼동 주의
  feedback/  — 피드백
  tech/      — 기술 사항
  manual/    — 매뉴얼/가이드
  troubleshootings/ — 트러블슈팅
```

### 3. 소스 수정 전 반드시 승인 요청

회귀나 버그 발견 시 즉시 수정하지 않고 작업지시자에게 승인을 요청한다.
특히 회귀 발생 시 "어느 파일에서 회귀가 나는지 먼저 확인하고, 수정 방향을 보고한 후 승인을 받는다."

### 4. 컨텍스트 압축 후 재확인

세션이 길어져 컨텍스트가 압축된 경우, 작업 재개 시 CLAUDE.md와 관련 계획서를 다시 읽어 규칙을 확인한다.

---

## 컨트리뷰터를 위한 체크리스트

클로드 코드로 이 프로젝트에 기여할 때 다음을 확인한다:

- [ ] 작업 시작 전 CLAUDE.md 전체 읽기
- [ ] 타스크 브랜치 생성 전 GitHub Issue 등록 (`gh issue create`)
- [ ] 수행계획서 → 구현계획서 → 단계별 구현 → 완료보고서 순서 준수
- [ ] 각 단계 완료 후 반드시 승인 요청 (승인 없이 다음 단계 진행 금지)
- [ ] 소스 수정 전 승인 요청
- [ ] 문서 위치: 단계별 보고서 → `mydocs/working/`, 최종 보고서 → `mydocs/report/`
- [ ] 이슈 클로즈는 작업지시자 승인 후에만 수행

---

## 관련 메모리

- [최종 보고서 위치 규칙](../manual/memory/feedback_report_location.md)
- [타스크 프로세스 반드시 준수](../manual/memory/feedback_process_must_follow.md)
- [이슈 클로즈는 작업지시자 승인 필수](../manual/memory/feedback_no_close_without_approval.md)
