import { serve } from "https://deno.land/std@0.184.0/http/mod.ts";
import { handler } from "../build/with_deno.js";

await serve(handler);
