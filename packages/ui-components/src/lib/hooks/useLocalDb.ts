import { getContext } from "svelte";
import type { SQLiteWasmDatabase } from "@rainlanguage/sqlite-web";
import {
  DeploymentStepsError,
  DeploymentStepsErrorCode,
} from "../errors/DeploymentStepsError";

export const LOCAL_DB_CONTEXT_KEY = "local-db-context";

export function useLocalDb(): SQLiteWasmDatabase {
  const db = getContext<SQLiteWasmDatabase>(LOCAL_DB_CONTEXT_KEY);
  if (!db) {
    DeploymentStepsError.catch(
      null,
      DeploymentStepsErrorCode.NO_LOCAL_DB_PROVIDER,
    );
  }
  return db;
}
