import { serve } from "https://deno.land/std@0.184.0/http/mod.ts";
import { contentType } from "https://deno.land/std@0.184.0/media_types/mod.ts";
import { handler } from "../build/with_deno.js";

const STATIC_PATH = "/static";
const PUBLIC_DIR = "../build/with_deno/public";

async function handleRequest(request: Request): Promise<Response> {
  const { pathname } = new URL(request.url);

  if (pathname.startsWith(STATIC_PATH)) {
    const path = pathname.slice(STATIC_PATH.length);
    const ext = pathname.slice(pathname.lastIndexOf("."));
    const filePath = `${PUBLIC_DIR}/${path}`;
    console.log("Serving file: " + filePath);

    const file = await Deno.readFile(filePath);

    return new Response(file, {
      headers: {
        "content-type": contentType(ext) ?? "application/octet-stream",
      },
    });
  }

  return handler(request);
}

// deno-lint-ignore no-explicit-any
function handleError(error: any): Response {
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
  port: 5000,
  hostname: "127.0.0.1",
  onError: handleError,
  onListen: ({ hostname, port }) => {
    console.log(`⚡ Server started at: "http://${hostname}:${port}"`);
  },
});
