# 보정 소스의 필수 편람 Native 383쪽 게이트

source `37e5bba3007614d9304c3dbf863503a8c7340405`, 실제 제품 SHA-256 `359eb1788753114c9799fb03b1d66678ea91b58802aa85f18c7fda54aa6890f0`.

**383쪽 통과, 전쪽 최저 raw 99.96%, 기준 표시값 하락 0쪽, webfont fallback 0건, exit 0**.

96 DPI·Native Skia·Print profile·compat 2024·색상 경계 보정 전 2px raw다. [전체 TSV](handbook-native-silhouette.tsv)를 제출한다.

TSV SHA-256 `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308`.
원 기준 `mydocs/pr/assets/issue_8153/final-50cdf-validation-20261030/handbook-hwpx-native.tsv`, SHA-256 `d5006cd9110c1247282aa5627f1728d9c751c3212925204582b9b40f39900308`.

| 대상 | 실제 파일 SHA-256 |
| --- | --- |
| HWPX `samples/2025 행정업무운영 편람(최종).hwpx` | `c6dd7e847a99f219681afc5a29c80a9665c04df9cda4d820a3350d739664fdf6` |
| 독립 한컴 Print PDF `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf` | `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e` |

## 디스크 부족과 복구의 구분

첫 실행은 전체 구간을 측정한 뒤 결과 집계·보고서 저장에서 `No space left on device`로 종료 코드 1이었다. 이를 성공으로 집계하지 않았다. 별도 doctest도 동시 링크 중 Bus error로 종료 코드 101이었다.

Cargo·rustc·rustdoc·wasm 작업 0개와 캐시의 열린 파일 참조 0개를 확인한 뒤 이 검토가 만든 종료된 증분 캐시 4개를 정리했다. target 전체·deps·fingerprint·실행 파일·진행 중 시각 산출물을 유지했다. 약 7.3GB의 여유 공간을 확보했다.

편람은 동일 입력·PDF·제품·글꼴 조건의 완료 구간에 canonical `render_silhouette_resumable`의 `resume=True`를 적용해 복구했다. metric·허용치·원 기준·제품은 바꾸지 않았다. 원래 `required_handbook_gate`와 `handbook.compare`를 실행해 전쪽 집계·판정·기존 품질 게이트를 완료했다. 별도 재렌더링을 완료했다고 표시하지 않는다.

실제 복구 시간 63.754초, 복구 로그 SHA-256 `d2ce967dea34dcb8f56a867d0a5c78f8b7a3eea76283663053842599616a24eb`.

복구 실행:

```bash
venv/bin/python output/pr-review/planet6897-20261103/resume_vertical_handbook.py
```

doctest는 source·기대값을 바꾸지 않고 `--test-threads 1`로 동시 링크만 제한해 재실행했다. Cargo 빌드 잡은 8개 그대로다. **8 PASS / 0 FAIL / 3 IGNORE, exit 0**을 확인했다. 이는 전체 Rust nextest나 Native Skia의 완료를 대신하지 않는다.

doctest 재검증 로그 SHA-256 `1b79411ef948d8cceb1aecfdfc626e88cd1e287bd4da8df719c8e336e14c311a`.

재검증 명령:

```bash
bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_vertical_worktree.sh \
  cargo test --locked --doc --target-dir /home/tsjang/rhwp/target/pr-review \
  -- --test-threads 1
```

작업 디렉터리는 보정 worktree다. 출력 증거의 시스템 글꼴 환경과 Rust의 공개 기본 글꼴 5개 환경을 구분한다. 전체 회귀는 기존 12스레드 설정으로 이어 실행한다.
