# Task M100 #2072 Stage 25 - tech legacy 문서 분류

## 목표

`mydocs/tech`와 `mydocs/tech/webhwp`에서 front matter가 없는 기술 문서를 내용 기준으로 감사한다.
현재 스펙·아키텍처·설계 결정과 작성 당시 조사·비교·PoC 기록을 구분하고 canonical 관계를 명시한다.

## 원칙

- 현재 구현이 따라야 하는 포맷 사실과 계약은 `reference/active`로 분류한다.
- 현재 구조와 확장 원칙을 정하는 문서는 `decision/active` 또는 `canonical/active`로 분류한다.
- 특정 시점의 비교, 연구, PoC, 교체 검토는 현재 사실처럼 읽히지 않도록 `historical`로 분류한다.
- `webhwp` 묶음은 별도 문서 지도를 추가해 원본 분석 자료의 범위를 명시한다.
- 문서 이동은 내용상 소유 경계가 명확하고 링크 이득이 큰 경우에만 수행한다. 분류만으로 역할이 충분히
  드러나면 불필요한 경로 churn을 만들지 않는다.

## 검증 계획

- `mydocs/tech` 전체 front matter 누락 재검사
- tech 문서 지도와 `webhwp` 지도에서 주요 canonical·historical 진입점 확인
- 기본 링크·메타데이터·Python 구문·`actionlint`·`git diff --check`
- 제품 소스와 테스트 변경 여부 확인

## 감사 결과

### 이슈별 조사 분리

루트에 있던 특정 이슈의 조사·PoC 문서를 다음 묶음으로 이동했다.

| 이슈 묶음 | 이동한 문서 | 현행 권위 문서 |
| --- | ---: | --- |
| `issue-101` | 부분 표 흐름 1개 | `table_layout_rules.md` |
| `issue-112` | ThorVG PoC 1개 | `thorvg_decision.md` |
| `issue-124` | 캔버스·폰트 측정 3개 | `rendering_engine_design.md`, `font_fallback_strategy.md` |
| `issue-139` | 수식 조사 4개 | 현재 수식 구현과 테스트 |
| `issue-397` | 증분 레이아웃 조사 4개 | `rendering_engine_design.md` |

각 디렉터리에 README를 추가해 당시 조사 범위와 현행 권위 문서의 경계를 명시했다. 내부 문서가 이전
루트 경로를 계속 참조하지 않도록 이동과 같은 stage에서 링크를 새 경로로 갱신했다.

### 보관 문서 분리

현재 계약으로 오인될 수 있던 이전 roadmap, 제품 비전, 교체 전략, 구현 전 제안과 대체 설계 10개를
`tech/archive/`로 이동했다. 보관 문서는 `historical`로 분류하고, 현재 계약이 존재하는 경우 해당
canonical을 가리키며 나머지는 archive 지도를 따른다.

외부 이력에서 반복 참조될 가능성이 큰 `dev_roadmap.md`와 `direct_printing_guideline.md`만 짧은
redirect stub으로 남겼다. 나머지는 내부 링크를 직접 갱신해 redirect를 늘리지 않았다.

### webhwp 조사 묶음

루트의 webhwp 비교 문서 3개를 기존 `tech/webhwp/`로 이동하고, 디렉터리 README를 추가했다.
이 묶음의 10개 분석 문서는 2026-02 minified bundle을 관찰한 `investigation/historical` 자료이며,
현재 rhwp 렌더링·표·폰트 계약의 권위 문서가 아님을 명시했다.

### 루트 기술 문서 현행성

루트에 남긴 스펙·IR·렌더링·저장·포맷 참고 문서는 내용을 읽고 `reference`, `decision` 등으로
분류했다. 검증되지 않은 offset을 포함한 `hwp_ole_spec.md`는 `reference/historical`로 표시했다.
HWPX 외부 참조 문서의 개인 로컬 경로는 upstream 저장소 URL로 바꿨다.

분류 후 `mydocs/tech`의 Markdown은 총 170개이며 루트에는 53개가 남는다. `investigations` 91개,
`archive` 15개, `webhwp` 11개를 포함해 front matter가 없는 문서는 0개다.

## 검증 결과

- 이전 루트 경로 잔존: 0건 (`mydocs` 전체)
- 내부 Markdown 상대 링크: 383개 문서, 이상 없음
- 현행 메타데이터 검사: 242개 문서, 이상 없음
- `mydocs/manual`, `mydocs/tech`, `mydocs/troubleshootings` front matter 누락: 0건
- Python 검사 스크립트 구문: 이상 없음
- `actionlint`: 오류 없음
- `git diff --check`: 이상 없음
- 제품 소스·테스트 변경: 없음

메타데이터 검사 대상이 아직 파일별 하드코딩 목록이므로, 전체 디렉터리를 자동 수집하도록 바꾸는 작업은
다음 stage에서 별도 커밋한다.
