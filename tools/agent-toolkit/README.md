# RHWP Agent Toolkit

플레이북의 일반적인 패턴을 자동화하는 에이전트 도구킷입니다.

## 개요

이 도구킷은 에이전트 실무 대체 예제집의 에이전트 시퀀스를 자동화하는 Python 스크립트/라이브러리입니다.

### 구조

```
tools/agent-toolkit/
├── lib/              # 공통 라이브러리
├── workflows/        # 워크플로우 스크립트
├── tests/            # 테스트
└── examples/         # 사용 예제
```

## 워크플로우

1. **form_filling.py** - 서식 자동 작성 (누름틀 메일머지)
2. **table_harvest.py** - 표 데이터 수확 (HWP → CSV)
3. **archive_search.py** - 아카이브 검색 + 페이지 렌더
4. **bulk_sweep.py** - 대량 문서 스윕 (메타/본문/구조)
5. **distribution_verify.py** - 배포본 동일성 검증

## 사용법

각 워크플로우는 독립 실행 가능한 Python 스크립트입니다:

```bash
python3 workflows/form_filling.py template.hwp data.json -o output.hwp
```

자세한 사용법은 각 워크플로우 스크립트의 `--help`를 참조하세요.

## 관련 문서

- [에이전트 실무 대체 예제집](../../mydocs/manual/agent_task_playbook.md)
- [에이전트 도구킷 가이드](../../mydocs/manual/agent_toolkit_guide.md)
