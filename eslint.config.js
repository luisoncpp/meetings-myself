import js from '@eslint/js';
import ts from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';

export default [
  {
    ignores: [
      'dist/',
      'target/',
      'node_modules/',
      'coverage/',
      '.svelte-check/',
      '.agents/',
      '.fallow/',
      'src-tauri/',
      'crates/',
      'launcher/',
      'docs/',
    ],
  },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parserOptions: { parser: ts.parser },
      globals: {
        window: 'readonly',
        document: 'readonly',
        KeyboardEvent: 'readonly',
      },
    },
  },
  {
    files: ['**/*.svelte.ts'],
    languageOptions: {
      parserOptions: { parser: ts.parser },
    },
  },
  {
    rules: {
      // GUIDELINES.md: no function takes more than 3 parameters.
      'max-params': ['error', 3],
      // GUIDELINES.md: no source file exceeds 200 lines.
      'max-lines': ['error', { max: 200, skipBlankLines: true, skipComments: true }],
      // GUIDELINES.md: no function exceeds 30 lines.
      'max-lines-per-function': ['error', { max: 30, skipBlankLines: true, skipComments: true }],
    },
  },
];
