---
name: hwp2hwpx Java 라이브러리 (매핑 권위 자료)
description: HWP↔HWPX 변환 작업 시 매핑 명세 권위 자료. Apache 2.0, hwplib/hwpxlib 저자 (neolord0/dogfoot)
type: reference
originSessionId: 67d1cb8f-86d4-4672-b831-a8d028a1cfcf
---
`/home/edward/vsworks/hwp2hwpx` — HWP→HWPX Java 변환 라이브러리.

**Why use**: HWP↔HWPX 의 표/셀/문단/lineseg/CommonObjAttr 가 enum/필드 단위 1:1 매핑임을 직접 확인 가능. hwp2hwpx 의 `ForCell.java` (cell.hasMargin ↔ apply_inner_margin), `ForShapeObject.java` (CommonObjAttr 비트 매핑), `ForPara.java` (lineseg vpos 1:1) 가 우리 어댑터 명세의 권위 있는 레퍼런스.

**How to apply**:
- HWPX↔HWP 변환 영역 신규 작업 시 본 라이브러리의 해당 변환 코드를 먼저 읽기
- 단, **렌더링은 미고려** (작업지시자 지적, 2026-04-19) — 페이지네이터 휴리스틱 / typeset 영향 / 한컴 호환은 본 라이브러리에서 잡지 못함. 우리 rhwp 페이지네이터·한컴 검증과 별개로 처리해야 함.
- Java + hwplib/hwpxlib 객체 모델 의존이라 직접 코드 포팅 불가. **매핑 명세서로만 사용**.

라이선스 Apache 2.0 — 명세 인용 가능. 코드 직접 포팅 시 라이선스 표기 필요.
