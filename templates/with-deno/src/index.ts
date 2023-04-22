import { serve } from "https://deno.land/std@0.184.0/http/mod.ts";
import { contentType } from "https://deno.land/std@0.184.0/media_types/mod.ts";
import { handler, set_envs } from "../build/with_deno.js";
import * as denoPath from "https://deno.land/std@0.183.0/path/mod.ts";
import * as denoFs from "https://deno.land/std@0.183.0/fs/mod.ts";

// FIXME: Theses values should come from environment variables
const PORT = 5000;
const HOST = "127.0.0.1";
const STATIC_PATH = "/static";
const PUBLIC_DIR = denoPath.join(Deno.cwd(), "public");

// Set envs
const envs = Deno.env.toObject();
set_envs(envs);

async function handleRequest(request: Request): Promise<Response> {
  try {
    const { pathname } = new URL(request.url);

    if (pathname.startsWith(STATIC_PATH)) {
      return await serveStaticFile(request);
    }

    return handler(request);
  } catch (err) {
    return handleError(err);
  }
}

async function serveStaticFile(request: Request): Promise<Response> {
  const { pathname } = new URL(request.url);
  const path = pathname.slice(STATIC_PATH.length);
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
  console.log(`📛  Something went wrong: ${error}`)
  // prettier-ignore
  const errorMessage = error.message || error.description || "Something went wrong";
  const status = Number(error.statusCode || error.status || error.code || 500);

  const html = `
      <html>
        <head>
          <title>Error</title>
          <style>
            body {
              display: flex;
              justify-content: center;
              align-items: center;
              height: 100vh;
            }
            h1 {
              font-size: 3rem;
              text-align: center;
            }
          </style>
        </head>
        <body>
          <h1>${errorMessage}</h1>
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
  port: PORT,
  hostname: HOST,
  onError: handleError,
  onListen: ({ hostname, port }) => {
    console.log(`⚡ Server started at: \`http://${hostname}:${port}\``);
  },
});
