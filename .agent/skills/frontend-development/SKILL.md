---
description: How to add new frontend pages, components, and API calls
---

# Frontend Development

## Tech Stack

- React 19 + TypeScript
- Vite 7 (dev server on port 5173)
- Vanilla CSS (no CSS framework)
- API base URL: `http://localhost:3001/api`

## File Structure

```
frontend/src/
├── main.tsx          # Entry point
├── App.tsx           # Main layout (sidebar + content)
├── App.css           # Global app styles
├── index.css         # CSS reset & variables
├── types.ts          # Shared TypeScript types
├── vite-env.d.ts     # Vite type declarations
├── api/
│   └── client.ts     # API client (fetch wrapper)
├── pages/
│   ├── ConnectionPage.tsx/.css
│   └── TablePage.tsx/.css
└── components/
    ├── DataTable.tsx/.css
    └── DynamicForm.tsx/.css
```

## Adding a New API Call

### 1. Add type in `types.ts`

```typescript
export interface MyEntity {
  id: string;
  name: string;
}
```

### 2. Add API method in `api/client.ts`

```typescript
export const api = {
  // ...existing methods
  myNewCall: (data: MyRequest): Promise<MyEntity> =>
    request<MyEntity>("/my-endpoint", {
      method: "POST",
      body: JSON.stringify(data),
    }),
};
```

## Adding a New Page

1. Create `pages/MyPage.tsx` and `pages/MyPage.css`
2. Import and render in `App.tsx` with appropriate conditional rendering
3. Add navigation trigger in the sidebar

## API Client Pattern

All API calls use the `request<T>()` helper which:

- Adds `Content-Type: application/json`
- Throws on non-OK responses with error message from response body
- Returns `null` for 204 No Content responses

## CSS Variables

Key CSS variables defined in `index.css`:

- `--accent` — Primary accent color
- Use existing patterns in `App.css` for new components
