# Issue #8863 재현 입력

- 출처: https://github.com/NomaDamas/k-skill/blob/main/corporate-registration-consulting/templates/attachment-hwp/articles-of-incorporation.hwp
- 보고: https://github.com/edwardkim/rhwp/issues/8863
- 원본 바이트를 변경하지 않은 공개 정관 양식 HWP5 파일입니다.
- SHA-256: `244f5eddd3bda1b200ad28c2a4bd1295182949c502255821d418f6605dc8cb52`
- 구역 0 문단 4는 같은 시작 위치 0에 글자 모양 7, 6, 12, 6을 저장합니다. 조회에서 마지막 모양 6이 유효하며, 위치 11의 두 글자를 삭제해도 이 문단의 남은 서식은 유지되어야 합니다.
- 검사 범위: 실제 편집 API와 HWP/HWPX 저장 왕복의 글자·굵기·색상 보존. 화면 시각 일치율을 대신하지 않습니다.

- 원 저장소 MIT 라이선스 고지를 `LICENSE`에 보존합니다: https://github.com/NomaDamas/k-skill/blob/main/LICENSE
