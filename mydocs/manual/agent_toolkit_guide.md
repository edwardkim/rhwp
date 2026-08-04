---
kind: guide
status: active
canonical: tools/agent-toolkit/README.md
last_verified: 2026-08-04
---

# 에이전트 자동화 도구킷 가이드

## 개요

**RHWP Agent Toolkit**은 [에이전트 실무 대체 예제집](agent_task_playbook.md)의 플레이북 패턴을 자동화하는 Python 기반 도구 모음입니다.

### 위치

```
rhwp/tools/agent-toolkit/
├── lib/              # 공통 라이브러리
├── workflows/        # 워크플로우 스크립트 
├── tests/            # 테스트
└── examples/         # 사용 예제
```

## 워크플로우

각 워크플로우는 플레이북의 "에이전트 시퀀스"를 자동화합니다:

### 1. form_filling.py - 서식 자동 작성
- **플레이북**: 예제 1 (누름틀 메일머지)
- **자동화**: `rhwp fields` → `edit fill-fields` → 재독 검증

### 2. table_harvest.py - 표 데이터 수확
- **플레이북**: 예제 2 (HWP 표 → CSV)
- **자동화**: `rhwp export-tables` → CSV 변환

### 3. archive_search.py - 아카이브 검색
- **플레이북**: 예제 3 (검색 → 페이지 렌더)
- **자동화**: `rhwp search` → 매치 페이지만 렌더

### 4. bulk_sweep.py - 대량 문서 스윕
- **플레이북**: 예제 4 (메타/본문/구조 일괄 처리)
- **자동화**: `rhwp batch info/export-text/export-structure`

### 5. distribution_verify.py - 배포본 동일성 검증
- **플레이북**: 예제 8 (변조·판본 확인)
- **자동화**: `rhwp render-diff` + SVG 바이트 비교

## 핵심 개념

### 기계 검증 (눈 검증 → 계약 검증)

- **종료 코드**: 0(성공), 1(실패), 3(검증실패), 4(페이지불일치)
- **재독 대조**: 쓴 것을 다시 읽어 값·구조를 프로그램으로 비교
- **자기서술**: 실행 전 명령 가용성 확인

### 오류 격리 및 재시도

배치 작업에서 일부 실패 시 성공 레코드는 보존하고 실패분만 재시도 가능.

## 사용 예제

```bash
# 서식 자동 작성
python3 tools/agent-toolkit/workflows/form_filling.py \
    template.hwp data.json -o output.hwp

# 표 데이터 수확
python3 tools/agent-toolkit/workflows/table_harvest.py \
    document.hwp -o tables/ -f csv

# 대량 문서 스윕
python3 tools/agent-toolkit/workflows/bulk_sweep.py \
    docs/ -o results/ --min-pages 10
```

## 관련 문서

- [상세 문서](../../tools/agent-toolkit/README.md)
- [에이전트 실무 대체 예제집](agent_task_playbook.md)
- [CLI 명령어 매뉴얼](cli_commands.md)

---

**Last Updated**: 2026-08-04
