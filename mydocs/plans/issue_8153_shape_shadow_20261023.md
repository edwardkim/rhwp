# 이슈 #8153: 평행 도형 그림자의 공통 배치와 paint 보정

- 작업 브랜치: `task/8153-handbook-raw100`
- 선행 코드: `220c70a677781902282d5b5b36f259a5987d3b70`
- 원본: 기존 편람 HWP/HWPX와 각 383쪽의 독립 한컴 2024 Print PDF를 유지한다.
- 완료 목표: #8153의 전쪽 Native/fresh WASM raw 100%·정확한 미대응 0과 중요 경계 회귀를
  그대로 유지한다. 이 단계는 205쪽 그림자 결함의 보정이며 전체 목표 완료가 아니다.

## 선행 근거와 원인

공식 OWPML 모델의 `SHADOWTYPE`은 평행 네 방향을 `PARELLEL_LEFTTOP`부터
`PARELLEL_RIGHTBOTTOM`으로 구분한다. signed offset은 별도 필드다.
[고정한 공식 enum 정의](https://github.com/hancom-io/hwpx-owpml-model/blob/1453388472c703a4b299a0834f425cdac16644b9/OWPML/Class/enumdef.h#L374)와
[공식 도형 그림자 속성](https://github.com/hancom-io/hwpx-owpml-model/blob/1453388472c703a4b299a0834f425cdac16644b9/OWPML/Class/Para/ShadowType.h)을 대조했다.
현대 그림 효과와 글자 그림자는 이 타입과 구분한다.

현재 HWPX parser는 공식 평행 이름을 누락하며 serializer도 다른 이름을 기록한다.
Canvas는 preset을 무시한 raw offset과 임의 blur를 적용하고 Native는 도형 그림자를 생략한다.
parser만 수정하면 잘못된 Canvas 출력을 HWPX에도 추가하므로 공통 paint를 함께 보정한다.

기존 매핑은 `a2c3f81236b7a1c1fefa3c9dcb57fe617be6fc37`에서 추적했다. commit의 Task 번호를
GitHub 이슈 번호로 간주하지 않는다. 해당 매핑과 같은 원본에서 재현되는 결함을 각각 확인했다.

원본 205쪽의 13개 도형은 양쪽 PDF에 같은 오른쪽 아래 그림자가 있다. 별도 진단 입력의
15조건 및 네 방향·signed offset의 방향 반전·절반 크기·내부 글자를 포함한 30조건을 실제
한컴 2024 PrintMethod 0으로 출력했다. 기존 383쪽 PDF를 새 진단 PDF로 교체하지 않았다.
30조건 PDF SHA-256은 `44b8e5e76e26b8cef269e5c92c0b963658f0da5285290662751a598237e22961`이다.
수정 전 Native의 진단 raw는 15조건 89.26777%, 30조건 80.55896%다.
진단 입력은 아직 정식 회귀 fixture가 아니며 양쪽 시각 선행 조건 후 등록 여부를 판단한다.

## 공통 소비 경로

1. HWP/HWPX의 도형 타입·signed offset을 model에서 보존한다. HWPX 이름 매핑은 하나를 공유한다.
2. 독립 개체의 외곽 원점에서 본체 원점을 계산한 뒤 도형과 내부 텍스트의 실제 배치가 함께 소비한다.
   묶음 자식은 renderingInfo가 이미 정한 원점이므로 독립 개체의 원점 정규화를 중복 적용하지 않는다.
3. LayerBuilder가 평행 그림자를 일반 도형 실루엣과 이동 레이어로 내리고 본체의 legacy shadow를
   제거한다. 내부 텍스트·이미지를 그림자 자식으로 복제하지 않는다.
4. Native·Canvas·SVG/PDF의 공통 레이어 재생과 visual bounds가 같은 이동 행렬을 소비한다.
   backend마다 preset 계산이나 blur를 따로 추가하지 않는다.
5. 기울임·원근·확대/축소 계열은 평행 이동으로 대체하지 않는다. 원본 타입의 왕복 보존과 해당
   계열의 실제 재생 지원을 구분한다.

## 검증 계획과 남은 조건

- 수정 전/후 진단 PNG·TSV·문단/텍스트 소속과 동일 PDF의 실제 drawing 경계를 비교한다.
- 양쪽 원본 205쪽의 Native/fresh WASM을 독립 PDF와 직접 대조한다.
- 독립 개체의 네 방향·offset 방향 반전·크기와 그룹 원점의 적용/비적용을 각각 확인한다.
- 같은 문서를 사용하는 기존 검사와 실제 소속·내용 순서 검사를 확인한다. 정식 회귀는 해당 범위의
  Native/fresh WASM 90% 이상과 직접 판독 후 추가하며 완료 페이지의 ±2px 조건도 준수한다.
- 273쪽의 기존 이미지 회귀, 단일 선폭으로 근사한 그림 테두리와 전쪽 raw 100% 미완료는 별개로 남는다.
- 이 문서는 구현 후보의 계획이며, 아래 검증 결과 없이 통과·제출 가능으로 판정하지 않는다.
