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

// Auto-shim request.ts because @hey-api/client-fetch breaks openapi-typescript-codegen's request.ts
import { existsSync, readFileSync, writeFileSync } from 'fs';
import { resolve } from 'path';

const requestTsPath = resolve('src/openapi-rq/requests/core/request.ts');
if (existsSync(requestTsPath)) {
  const content = readFileSync(requestTsPath, 'utf-8');
  if (content.trim() === '') {
    const shimCode = `import { client } from '../../modern/client.gen';

export const request = async (config, options) => {
    let url = options.url || '';
    if (options.path) {
        for (const [k, v] of Object.entries(options.path)) {
            url = url.replace('{' + k + '}', String(v));
        }
    }
    
    const res = await client.request({
        method: options.method,
        url: url,
        query: options.query,
        body: options.body,
        headers: options.headers,
        mediaType: options.mediaType,
    });
    
    if (res.error) {
        throw {
            body: res.error,
            status: res.response?.status,
            statusText: res.response?.statusText,
            name: 'ApiError',
        };
    }
    
    return res.data;
};
`;
    writeFileSync(requestTsPath, shimCode);
    console.log('✅ Shimmed src/openapi-rq/requests/core/request.ts for @hey-api compat');
  }
}
