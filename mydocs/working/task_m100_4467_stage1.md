# #4467 Stage 1 완료 — 프로젝트 로드맵 canonical 분리

- **Issue**: [#4467](https://github.com/edwardkim/rhwp/issues/4467)
- **브랜치**: `task/4467-project-roadmap`
- **기준**: `upstream/devel` `629cd33db`
- **계획**: [task_m100_4467.md](../plans/task_m100_4467.md)
- **완료일**: 2026-08-10 KST

## 1. 결과

루트 `ROADMAP.md`를 프로젝트 전체 제품 방향과 버전 단계의 canonical로 신설했다. 초기
`0.5 → 1.0 → 2.0 → 3.0` 방향을 보존하면서 다음 관리 계약을 추가했다.

- 증거 등급: `[완료]`, `[실측]`, `[문서]`, `[이슈]`, `[가설]`
- 제품 단계 공통 항목: 한 줄, 지금, 범위·비범위, 착수 게이트, DoD, 의존
- 횡단 제품 트랙 P1~P7: 포맷·IR, 조판·충실도, 편집·저장, 제품·플랫폼, AI·자동화,
  기여·거버넌스, 접근성·공공 자산
- 제품 방향·milestone·기술 계약·작업 절차·release 기록 사이의 권위 경계
- 로드맵 자체가 구현·PR·merge 승인이 아니며 수치와 작업 목록을 중복 관리하지 않는 갱신 규칙

## 2. README 분리

- `README.md`는 비전, 현재 `v0.8.2 → v1.0` 단계와 `ROADMAP.md` 진입점만 남겼다.
- `README_EN.md`도 중복 단계 표를 제거하고 같은 한국어 canonical을 가리킨다. 별도 영문 로드맵을
  만들지 않아 장기 목표를 이중 관리하지 않는다.
- 현재 구현 기능을 보여 주는 `v0.5.0 ~ v0.8.x` 이정표는 README에 유지하고 제목을 “현재 이정표”로
  명확히 했다.

## 3. #3907과의 관계

#2659·#3608·#3880·#3907과 로컬 `agent_roadmap/` 문서 세트는 프로젝트 로드맵 P5
`AI·자동화`의 하위 기술 지도로 배치했다. 하위 지도는 R1~R100의 기술 상세를 계속 소유하지만 다음을
대체하지 않는다.

- 프로젝트 전체 제품 우선순위
- GitHub v1.0.0 milestone의 완료 판단
- 개별 구현·PR·merge 승인 절차

`llms.txt`, `mydocs/README.md`, `mydocs/tech/README.md`, `agent_roadmap/README.md`에 이 관계를
상호 연결했다.

## 4. 검증

```text
python3 scripts/check_markdown_links.py --changed-from upstream/devel --forbid-redirect-references
검사 문서: 530개 / 변경 파일: 10개 / redirect stub: 30개
내부 Markdown 상대 링크: 이상 없음

python3 scripts/check_document_metadata.py
메타데이터 검사 문서: 521개
문서 메타데이터: 이상 없음

git diff --check
통과
```

문서와 정보구조만 변경하므로 Rust·Studio·WASM·시각 테스트는 수행하지 않았다.

## 5. 남은 절차

- 작업지시자 문서 판정
- 승인 후 원격 branch push
- 별도 승인 후 Open PR 생성과 문서 PR 절차
