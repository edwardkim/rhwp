# Task M100 #4340 Stage 1 — U1 1단계: rhwp.integrations (LangChain·LlamaIndex)

- 이슈: [#4340](https://github.com/edwardkim/rhwp/issues/4340)
- 기준: `upstream/devel` · 브랜치 `task_m100_4340` · 2026-08-09 KST · 구현·검증 완료

## 산출물
- `src/rhwp/integrations/{__init__,langchain,llama_index}.py` — 쪽 단위 Document
  로더/리더. 선택 의존성 원칙: 모듈 임포트는 프레임워크 무요구, 사용 시점 지연
  임포트 + 미설치 시 pip 힌트 ImportError. 본체 "런타임 의존성 0" 유지.
- `tests/test_integrations.py` 4본 — 무의존 임포트·미설치 힌트(모킹)·실물 17쪽
  계약(langchain)·llama(importorskip, CI 스킵).
- python README 에 사용 절 추가.

## 실측·판단 기록
- export-text 봉투 `pages[]` 실측: 전체 내보내기 `page` 0-기반(0..16), `-p N`
  지정 시 1-기반 에코 — **봉투 내부 비일관 관찰**. 로더는 인용용 쪽 번호를
  배열 위치 기반 1-기반으로 결정론 부여(코드 주석·README 명시). 상류 봉투
  정합 여부 판단은 메인테이너 몫으로 PR 본문에 관찰만 남김.
- metadata 계약: source/format/page/total_pages (+llama extra_info 병합).

## 검증
- `pytest` 전체: **297 passed, 1 skipped**(llama 미설치 스킵) — 신규 4본 포함,
  기존 294 무회귀. langchain-core 는 로컬 설치로 실통합 검증.
- 프레임워크 미설치 CI 에서는 통합 2본이 명시 스킵되고 나머지 green 이어야
  한다(무의존 임포트 테스트가 그 계약을 고정).
