const {
  buildClientSchema,
  getIntrospectionQuery,
  lexicographicSortSchema,
  printSchema,
} = require("graphql");

const endpoint = process.argv[2];
const attempts = 30;
const delayMs = 2000;
const timeoutMs = 10000;

async function introspect() {
  const response = await fetch(endpoint, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ query: getIntrospectionQuery() }),
    // Bounded, because a graph-node that accepts the connection and then
    // stalls would hang this await and the retry below would never run. The
    // signal covers reading the body, not just the headers.
    signal: AbortSignal.timeout(timeoutMs),
  });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  const body = await response.json();
  if (body.errors) {
    throw new Error(JSON.stringify(body.errors));
  }
  return body.data;
}

async function main() {
  if (!endpoint) {
    throw new Error("usage: print-api-schema.js <graphql endpoint>");
  }
  let last;
  for (let attempt = 1; attempt <= attempts; attempt++) {
    try {
      const introspection = await introspect();
      // Sorted, because the order introspection reports types and fields in is
      // graph-node's own and is not what the snapshot is asserting.
      process.stdout.write(
        printSchema(lexicographicSortSchema(buildClientSchema(introspection))),
      );
      return;
    } catch (error) {
      last = error;
      console.error(
        `${endpoint}: attempt ${attempt}/${attempts}: ${error.message}`,
      );
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }
  }
  throw last;
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
