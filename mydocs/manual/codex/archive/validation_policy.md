---
kind: snapshot
status: historical
canonical: mydocs/manual/codex/README.md
last_verified: 2026-07-17
---

# Validation Policy

## Hancom Is The Compatibility Gate

HWP 저장 호환성 작업에서 rhwp 자기 검증은 충분하지 않다.

다음은 보조 자료다:

- rhwp 파서 roundtrip
- rhwp-studio 렌더링
- byte diff
- page count
- PDF 출력
- 외부 변환기 결과
- 한글뷰어 결과
- macOS 인쇄 결과

최종 호환성 판단은 작업지시자의 한컴 에디터 검증을 따른다.

권위 있는 기준:

- 한컴 2010 편집기 출력
- 한컴 2022 편집기 출력
- 작업지시자가 제공한 한컴 2020 변환 정답 파일

## PDF Is Not Authoritative

PDF는 환경 의존성이 크다.

영향 요인:

- 한컴 버전
- OS
- 폰트 설치 상태
- 프린터/PDF 드라이버
- 렌더링 환경

따라서 PDF 일치는 1차 정답으로 삼지 않는다.

## Visual Validation Matters

페이지 수가 같아도 시각적으로 틀릴 수 있다.

특히 다음은 page count나 byte diff만으로 놓치기 쉽다:

- 셀 내부 그림 위치
- 표 배치
- 쪽 배경
- 페이지 경계와 마진
- 선 두께와 테두리
- 문단 배경 무늬
- 이미지 clipping/clamp

작업지시자 시각 검증이 핵심 게이트다.

## HWP Damage Class

한컴 에디터의 "파일 손상" 판정은 rhwp가 읽을 수 있다는 사실로 반박할 수 없다.

대표 원인 후보:

- record payload 길이 불일치
- HWP5 필수 확장부 누락
- `control_mask`와 실제 controls 불일치
- `char_count`와 `PARA_TEXT` code unit 불일치
- `LIST_HEADER` 확장부 누락
- table record 말미 필드 누락
- DocInfo ID 참조 불일치
- BinData ID/index 매핑 불일치
- HWPX 의미값을 HWP5 ordinal로 잘못 매핑

## LINE_SEG Note

한컴은 `LINE_SEG`가 비어 있거나 기본값이어도 자체 재계산할 수 있다.

따라서 `LINE_SEG`는 중요한 힌트이지만, 파일 손상 원인을 무조건 `LINE_SEG`에서 찾으면 안 된다.
