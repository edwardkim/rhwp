# 시각 글꼴 공급 도구 계약

현재 source의 `test_visual_sweep_png`, `test_visual_sweep_native_webfonts`, `test_visual_sweep_handbook` 72건과 `font_rule_projection_gen` 16건을 실행해 모두 통과했다. `FontFamilyTrailingSpaceTests`는 실제 이름의 앞뒤 공백 정규화·기존 캐시 무효화·가운데 공백 및 PostScript 이름 보존을 검사한다. 최신 base와 후보의 시각 비교에는 같은 정규화된 측정 harness를 사용했으므로 이 별도 계약으로 도구 자체의 수정도 검증했다.

실행 환경은 `PYTHONPATH=scripts/tests:.`, 저장소 venv Python과 Node다. 최초 실행은 테스트 모듈의 형제 import 경로를 누락해 ImportError로 실패했고, 소스 변경 없이 실행 경로를 바로잡아 72건을 다시 실행했다. 이 최초 실패를 제품 회귀로 처리하거나 성공 로그로 덮어쓰지 않았다.

실행 로그의 해시·종료 코드·실제 결과는 [원장](validation.tsv)에 기록했다. 예상된 손상 글꼴 음성 입력의 경고와 실제 실패를 구분했다.
