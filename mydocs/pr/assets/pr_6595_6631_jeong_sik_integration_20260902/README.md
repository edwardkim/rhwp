# jeong-sik #6595--#6631 통합 후보 시각 증적

각 PNG는 왼쪽 한컴 기준 PDF와 오른쪽 현 후보 `rhwp export-svg`를 Chrome으로 래스터화한 직접 비교다.
전체 페이지의 pixel diff는 글꼴 및 텍스트 shaping 차이를 포함하므로 합격 임계값이 아니다. 각 review record가
명시한 결함 범위와 전용 회귀를 함께 확인하는 증적으로만 사용한다. 생성 명령과 SHA-256은 `manifest.tsv`에 고정했다.
