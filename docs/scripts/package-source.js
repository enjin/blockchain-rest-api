// Copyright (C) 2026 Enjin
// SPDX-License-Identifier: GPL-3.0-or-later
// Ship the corresponding documentation sources alongside the compiled bundle.
const { execFileSync } = require('node:child_process');
const path = require('node:path');
const root = path.resolve(__dirname, '../..');
execFileSync('tar', [
    '-czf', path.join(root, 'docs/dist/source.tar.gz'),
    '--exclude=docs/node_modules', '--exclude=docs/dist',
    '-C', root, 'docs', 'LICENSE', 'NOTICE',
], { stdio: 'inherit' });
