# #8174 JPEG frame sampling compatibility

S06에서 독립 생성·검사한 52개 JPEG를 바이트 변경 없이 보존한다. RGB source pixel은
`(31*x % 256, 19*y % 256, 23*(x+y) % 256)`이고 quality는 90이다.
일반 36개는 Pillow JPEG writer의 baseline/progressive·subsampling 0/1/2,
mixed 16개는 `cjpeg -quality 90 -sample 1x2,2x2,2x2`에 progressive 유무를 적용했다.
크기와 정확한 source SHA-256은 `manifest.json`에 있다. 아래 재생성기는 codec 버전에 따라
압축 바이트가 달라질 수 있어 보존된 fixture를 덮어쓰지 않고 다른 출력 폴더를 요구한다.

독립 `djpeg`의 전체 RGB 비교는 T06 증적에 보존했다. 모든 보간/IDCT 픽셀이 동일하다고
주장하지 않는다. 회귀는 SOF 크기·독립 검정색 1×1 대조군·동일 계수의 두 scan encoding·
정확한 예산 경계/실패/복구와 실제 PDF의 그림/alpha·양쪽 저장을 검사한다.
렌더링 골든이나 좌표 기대값이 아니다. `pdf/issue8174/jpeg-compat/`의 17개 실제 PDF는
별도 Native/fresh WASM 전쪽 100%와 현재 확대 판독을 확인한 뒤 추가했다.

상세 source/제품·명령·전쪽 TSV와 직접 판독은
`mydocs/working/task_m100_8174_t06_completion.md`에 연결한다.
