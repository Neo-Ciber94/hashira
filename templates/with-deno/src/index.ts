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

await serve(handleRequest, {
  port: 5000,
  hostname: "127.0.0.1",
});
