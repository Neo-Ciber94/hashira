import { serve } from "https://deno.land/std@0.184.0/http/mod.ts";
import { contentType } from "https://deno.land/std@0.184.0/media_types/mod.ts";
import { handler, set_envs } from "../build/{{crate_name}}.js";
import * as denoPath from "https://deno.land/std@0.183.0/path/mod.ts";
import * as denoFs from "https://deno.land/std@0.183.0/fs/mod.ts";
import {
  Status,
  STATUS_TEXT,
} from "https://deno.land/std@0.184.0/http/http_status.ts";

const PORT = Deno.env.get("HASHIRA_PORT") || 5000;
const HOST = Deno.env.get("HASHIRA_HOST") || "127.0.0.1";
const STATIC_PATH = Deno.env.get("HASHIRA_STATIC_DIR") || "/static";
const PUBLIC_DIR = denoPath.join(Deno.cwd(), "public");

// TODO: We are currently setting the rust wasm this way,
// due `std::env` had no access to the variables
const envs = Deno.env.toObject();
set_envs(envs);

async function handleRequest(request: Request): Promise<Response> {
  try {
    const { pathname } = new URL(request.url);
    if (pathname.startsWith(STATIC_PATH) || pathname === "/favicon.ico") {
      return await serveStaticFile(request);
    }

    return handler(request);
  } catch (err) {
    return handleError(err);
  }
}

async function serveStaticFile(request: Request): Promise<Response> {
  const { pathname } = new URL(request.url);
  const path = pathname.startsWith(STATIC_PATH)
    ? pathname.slice(STATIC_PATH.length)
    : pathname;
  const ext = denoPath.extname(pathname);
  const filePath = `${PUBLIC_DIR}/${path}`;

  if (!(await denoFs.exists(filePath))) {
    console.warn(`⚠️  File not found: ${filePath}`);
    return new Response("Not found", {
      status: 404,
    });
  }

  const fileInfo = await Deno.stat(filePath);
  const lastModified = (fileInfo.mtime ?? new Date())?.toUTCString();
  const headers = new Headers({
    "Content-Type": contentType(ext) ?? "application/octet-stream",
    "Last-Modified": lastModified,
  });

  const requestHeaders = new Headers(request.headers);
  const ifModifiedSince = requestHeaders.get("If-Modified-Since");

  if (ifModifiedSince && ifModifiedSince === lastModified) {
    return new Response(null, { status: 304, headers });
  }

  const file = await Deno.readFile(filePath);
  console.log("📂  Serving file: " + filePath);
  return new Response(file, { headers });
}

// deno-lint-ignore no-explicit-any
function handleError(error: any): Response {
  console.log(`📛  Something went wrong: ${error}`);
  // prettier-ignore
  const errorMessage = error.message || error.description || "Something went wrong";
  const status = Number(error.statusCode || error.status || error.code || 500);
  const statusText = STATUS_TEXT[status as Status] ?? "Error";

  const html = `
  <html>
    <head>
      <title>${status} | ${statusText}</title>
      <style>
        body {
          display: flex;
          justify-content: center;
          align-items: center;
          min-height: 100vh;
          overflow: hidden;
        }
        h1 {
          font-size: 3rem;
          text-align: center;
          font-family: monospace;
          overflow-wrap: break-word;
          max-width: 90vw;
        }
      </style>
    </head>
    <body>
      <h1>${errorMessage} | ${status}</h1>
    </body>
  </html>
`;

  return new Response(html, {
    status: Number.isNaN(status) ? 500 : status,
    headers: {
      "content-type": "text/html",
    },
  });
}

await serve(handleRequest, {
  port: Number(PORT),
  hostname: HOST,
  onError: handleError,
  onListen: ({ hostname, port }) => {
    console.log(`⚡ Server started at: \`http://${hostname}:${port}\``);
  },
});
