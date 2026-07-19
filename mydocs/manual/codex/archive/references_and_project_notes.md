---
kind: snapshot
status: historical
canonical: mydocs/manual/codex/README.md
last_verified: 2026-07-17
---

# References And Project Notes

## Project Identity

rhwp는 Rust로 작성된 HWP 문서 뷰어/에디터다.

넓은 정체성:

- 한국형 DTP 엔진
- 워드프로세서
- 한컴 웹기안기의 오픈소스 대안
- HWP/HWPX/HWP3를 공통 `Document` IR로 변환

## Parser Architecture

공통 IR:

```text
src/model/document.rs
```

포맷별 파서:

```text
src/parser/hwpx/
src/parser/hwp5/
src/parser/hwp3/
```

HWP3 전용 로직은 `src/parser/hwp3/` 안에 둔다. renderer/layout/document_core에 HWP3 전용 분기를 넣지 않는다.

## Output Folder Rule

새 출력 코드는 `output/` 바로 아래에 흩뿌리지 않는다.

용도별 하위 폴더:

- `output/re/`: 재현 검증
- `output/svg/`: SVG 출력
- `output/debug/`: 디버그 출력
- `output/poc/`: POC 산출물

## Font Reference

외부 TTF 폰트 경로:

```text
/home/edward/mygithub/ttfs
```

폰트 추가 시 `resolve_metric_alias` 계층과 `font_metrics_data` 계층을 함께 맞춰야 한다.

## Renderer Notes

이미지 렌더링 경로는 renderer별로 별도 함수가 있을 수 있다.

예:

- `svg.rs`
- `web_canvas.rs`
- paint/json 계열

시각 결함을 고칠 때 한 경로만 수정하고 다른 경로를 놓치지 않는다.

## Equation Note

한컴 수식 컨트롤은 항상 `treat_as_char`로 본다.

핵심 경로는 paragraph layout의 인라인 배치다.

## Release Notes

릴리즈 작업은 별도 절차가 필요하다.

- main 동기화 확인
- 관련 manual 정독
- 체크리스트와 1:1 대조
- AMO 제출 시 `data_collection_permissions`, `gecko_android`, platform suffix, source zip 등을 확인

현재 Task #854와 직접 관련된 작업은 아니다.

## External Public Docs

외부 공개 문서 작성 시 자기검열 체크리스트를 적용한다.

주의:

- 특정 회사명 비교
- 최상급 주장
- 개인정보
- 공공기관 오인 가능성
- 과장 표현

## Wording

기술 문서, 코드 주석, 보고서에서 "산수" 대신 "계산"을 사용한다.
