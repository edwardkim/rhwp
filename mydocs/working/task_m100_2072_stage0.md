# Task M100 #2072 Stage 0 - 문서 기준선과 링크 검사

## 목표

문서 위치를 바꾸기 전에 `mydocs/manual`과 `mydocs/tech`의 현행 구조를 측정하고,
내부 Markdown 상대 링크가 파일 이동 뒤에도 검증 가능한 상태인지 확인한다.

## 이 단계의 범위

- `scripts/check_markdown_links.py`를 추가한다.
- 기본 검사 대상은 `mydocs/manual`, `mydocs/tech`의 Markdown 파일이다.
- 외부 URL, `mailto:`, 문서 내부 앵커는 파일 존재 검사 대상에서 제외한다.
- fenced code block 안의 예시 링크는 검사하지 않는다.
- 문서 이동, redirect stub, 본문 중복 제거, CI hard gate 추가는 다음 단계로 미룬다.

## 분류 원칙

문서의 역할과 생명주기는 하나의 상태값으로 섞지 않는다.

- `kind`: `canonical`, `guide`, `reference`, `investigation`, `decision`, `snapshot`, `memory`
- `status`: `active`, `historical`, `superseded`
- `canonical`: 해당 문서가 참조해야 하는 권위 문서 경로
- `last_verified`: 사실성 또는 절차 현행성을 마지막으로 확인한 날짜

메타 블록은 Stage 1의 문서 지도와 Stage 2의 클러스터별 현행성 감사에서 도입 여부를 확정한다.
이 단계에서는 기존 문서에 일괄 추가하지 않는다.

## 현행 구조 관찰

- 2026-07-16 기준 검사 대상은 `manual` 129개, `tech` 118개로 총 247개 Markdown 파일이다.
- 최초 검사에서 13건을 발견했다. 12건은 archive 이동 또는 같은 디렉터리 경로 중복으로 생긴
  경로 오류였고, 1건은 저장소에 없는 HWP 5.0 원본 스펙 파일 참조였다. Stage 0에서 현행 보존
  경로 또는 권위 문서 링크로 정정한 뒤 clean 기준선을 만든다.
- 저장소 루트에는 프로젝트 전용 `AGENTS.md`가 없고, 현재 작업 규칙은 사용자 전역 부트로더와
  `CLAUDE.md`, `mydocs/manual/codex/` 문서에 나뉘어 있다.
- `manual`은 수행 방법, `tech`는 기술 사실과 설계 근거라는 경계를 우선 유지한다.
- `task_m100_*`라는 파일명만으로 이동 대상을 정하지 않는다. 장기 계약·기준선·설계 결정과
  일회성 조사 기록은 내용으로 분류한다.
- redirect stub은 외부 이력에서 자주 참조되는 문서만 allowlist로 유지하고, 저장소 내부 링크는
  이동 PR에서 새 경로로 직접 갱신한다.

## 다음 단계

1. 검사 결과를 기준선으로 기록한다.
2. 파일 이동 없이 `mydocs/manual/README.md`, `mydocs/tech/README.md`에 문서 지도와 권위 문서 표를 추가한다.
3. 클러스터별 현행성 감사를 마친 뒤에만 이동 전용 변경을 별도 stage로 진행한다.

## 검증 결과

```bash
python3 scripts/check_markdown_links.py
```

- 검사 문서: 247개
- 깨진 내부 Markdown 상대 링크: 0건
- 외부 URL과 문서 내부 앵커는 의도대로 검사 대상에서 제외했다.
