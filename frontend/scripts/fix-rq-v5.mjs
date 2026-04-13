import fs from 'fs';
import path from 'path';

const filePath = path.resolve('src/openapi-rq/queries/index.ts');

if (!fs.existsSync(filePath)) {
  console.error(`Error: File not found at ${filePath}`);
  process.exit(1);
}

let content = fs.readFileSync(filePath, 'utf-8');

console.log('Patching React Query hooks for TanStack Query v5 compatibility...');

// 1. Transform useQuery(key, fn, options) -> useQuery({ queryKey: key, queryFn: fn, ...options })
// Improved regex to handle commas in function arguments and multi-line definitions
content = content.replace(
  /useQuery\(\[([\s\S]+?)\], \(\) => ([\s\S]+?), options\)/g,
  'useQuery({ queryKey: [$1], queryFn: () => $2, ...options })'
);

// 2. Transform useMutation(fn, options) -> useMutation({ mutationFn: fn, ...options })
// Catching variants with different argument names or types
content = content.replace(
  /useMutation\(([^,]+), options\)/g,
  'useMutation({ mutationFn: $1, ...options })'
);

// 3. SPECIAL CASE: handle ones that might have missed the 'options' parameter or have different formatting
// e.g. useMutation(({ id, requestBody }) => AdminService.updateOrganization(id, requestBody), options)
content = content.replace(
  /useMutation\(\(\{\s*([^}]+)\s*\}\) => ([\s\S]+?), options\)/g,
  'useMutation({ mutationFn: ({ $1 }) => $2, ...options })'
);

fs.writeFileSync(filePath, content);
console.log('✅ Successfully patched src/openapi-rq/queries/index.ts');
