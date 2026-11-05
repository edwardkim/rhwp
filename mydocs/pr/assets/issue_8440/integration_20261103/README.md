# PR #8443 편집 후 재현 입력

원본 `samples/issue6123/3112461_railway_emc_criteria.hwpx`에 아래 명령을 적용한
`pr8443-emc-edit.hwpx`를 보존한다.

```bash
rhwp edit delete-text samples/issue6123/3112461_railway_emc_criteria.hwpx \
  --section 0 --para 114 --offset 0 --count 1 \
  -o mydocs/pr/assets/issue_8440/integration_20261103/pr8443-emc-edit.hwpx
```

출력 SHA-256은 `8009a8f11dd1f53545ed457fdee47326310caf26c56f2eda9bde7d226d7ab9e9`이며,
원 PR의 결정적 편집본 SHA와 같다. 기준은 같은 편집본을 한컴 2024 Print로 출력한
`pdf/issue-8440/emc_edit-2024.pdf`(SHA-256
`054f6a2be2dd1b08cb6ac98c72ba0563391c41d1f7b312f91b132a3fbfa17968`, 14쪽)다.
2026-11-03 검증 당시 제품 CLI는 통합 코드
`0884eafc7b2c34fae20a765109ff314514175e4b`에서 빌드했다.
