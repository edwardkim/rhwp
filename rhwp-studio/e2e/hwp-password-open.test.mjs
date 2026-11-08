/**
 * Issue #3474/#3481/#5986 browser regression — HWP3/HWPX password open
 * and save-protection lifecycle.
 *
 * The fixture password is intentionally reconstructed from character codes so
 * test output, source grep, and reports never print it as a literal value.
 */
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {runIssue8160PasswordOpen, runPasswordOpenContract} from './issue_8160_password_open.test.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const FIXTURES = [
  {
    name: 'HWP3 압축 비밀번호 문서',
    path: path.join(REPO_ROOT, 'samples', 'HWP3-password-123456.hwp'),
    pageCount: 24,
    canvasAt144Dpi: { width: 1191, height: 1684 },
  },
  {
    name: 'HWPX ODF AES-256-CBC 문서',
    path: path.join(REPO_ROOT, 'samples', 'HWP5-password-123456.hwpx'),
    // #7445: 과거 23쪽 핀은 이미 Native/MCP에서 이관했다. 암호 계약만 보호한다.
  },
];

await runIssue8160PasswordOpen();
await runPasswordOpenContract('Issue #3474/#3481/#5986 HWP3/HWPX 암호 문서 열기와 보호 상태', FIXTURES);
