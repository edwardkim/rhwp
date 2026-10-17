# PDF 이미지 배치 요청 예산 (#7966)

- `exceeds-32.pdf`: 메인테이너 제공 원본 `exceeds_32.pdf`와 동일, 19쪽, 1,460,215 bytes.
  SHA-256 `cd8465adc8c96e7dd6dc2412b0d198ea0d54ae019824cad63443bc31139ea3d4`.
  6쪽의 Image XObject 18개가 35회 배치된다. SMask는 별도의 배치로 세지 않는다.
- `image-placement-controls.pdf`: `python3 pdf/issue7966/generate.py`로 재생성한다.
  3쪽은 각각 XObject·Form·inline 경로에서 같은 2×2 RGB 자원을 33회 배치한다.
  서로 다른 CTM·절반 clip·50% opacity를 포함하며 총 99개 표시를 보존해야 한다.

독립 paint 기준은 각 PDF 자체의 Poppler 출력이다. 입력 의미의 기준은 위 생성식과
원본 content의 Do/BI paint 순서이며, rhwp의 기존 배치 좌표나 해시를 기대값으로 삼지 않는다.
전쪽 Native/fresh Docker WASM TSV·producer·대표 PNG와 직접 판독은 공동 작업 결과에 연결한다.
한도·오류 복구 검사는 조판 일치의 대용이 아니다. PDF parser 전체 할당 증명은 미완료다.


- `affine-clip-controls.pdf`: `python3 pdf/issue7966/generate_affine.py`로 만든 3쪽.
  삼각형의 nonzero clip, 회전한 Form BBox와 상위 사각 clip, 반대 감기 방향의 even-odd
  clip과 음수 scale·50% opacity를 각각 포함한다. 저해상도 원색 그림으로 clip 경계가
  원본 bitmap의 큰 픽셀 블록으로 대체되지 않는지도 확인한다.

배치 수 보정 뒤 원본 7쪽의 Form 239에서 affine clip의 미지원이 추가로 드러났다.
Form 호출 CTM의 두 비대각 성분 `-1.88e-7`과 원래 BBox를 그대로 해석한다.
새 지원은 최대 32 꼭짓점의 직선 볼록 경계이며, 다른 곡선·오목·복합 경계는 미지원이다.
RGBA 저장 프레임과 matrix를 함께 생산하며, 원래 물리 경계를 bbox 하나로 대신하지 않는다.
