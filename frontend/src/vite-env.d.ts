/// <reference types="vite/client" />

declare module "@fontsource-variable/*";

interface ImportMetaEnv {
  VITE_BASE_API_URL: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
