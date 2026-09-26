'use strict';
const fs = require('node:fs');
const { classifyChromeExtension } = require('./ci-impact-classifier.cjs');
module.exports = { classifyChromeExtension };
if (require.main === module) {
  let output;
  try { output = classifyChromeExtension(JSON.parse(fs.readFileSync(process.argv[2], 'utf8'))); }
  catch { output = { chrome_extension_e2e_required: 'true', chrome_extension_e2e_reason: 'classification-error' }; }
  console.log(JSON.stringify(output));
  if (process.env.GITHUB_OUTPUT) fs.appendFileSync(process.env.GITHUB_OUTPUT,
    Object.entries(output).map(([key, value]) => `${key}=${value.replace(/[\r\n]/g, ' ')}\n`).join(''));
}
