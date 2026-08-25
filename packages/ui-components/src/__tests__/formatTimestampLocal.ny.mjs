/**
 * Child script for formatTimestampSecondsAsLocal local-time coverage.
 * Run with TZ=America/New_York so Date local getters are non-UTC.
 *
 * Mirrors packages/ui-components/src/lib/services/time.ts formatting path.
 */
import dayjs from "dayjs";
import bigIntSupport from "dayjs/plugin/bigIntSupport.js";
import localizedFormat from "dayjs/plugin/localizedFormat.js";
import utc from "dayjs/plugin/utc.js";

dayjs.extend(bigIntSupport);
dayjs.extend(localizedFormat);
dayjs.extend(utc);

function formatTimestampSecondsAsLocal(timestampSeconds, useLocalTime = false) {
  const date = dayjs(timestampSeconds * BigInt("1000"));
  return (useLocalTime ? date : date.utc()).format("L LT");
}

process.stdout.write(formatTimestampSecondsAsLocal(BigInt("1672531200"), true));
