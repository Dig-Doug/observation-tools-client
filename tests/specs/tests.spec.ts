import { test, expect } from "@playwright/test";
import { spawnServer } from "../helpers/server";
import { TestId } from "../helpers/testIds";

test("Server homepage loads", async ({ page }) => {
  const server = await spawnServer();
  await page.goto(server.baseUrl);
  await expect(page.locator("h1")).toBeVisible();
  await server.stop();
});

test("Create execution", async ({ page }) => {
  const server = await spawnServer();
  const client = server.createClient();
  const executionName = "test-execution";
  const exe = client.beginExecution(executionName);
  const executionId = exe.idString;

  await page.goto(server.baseUrl);
  await page.getByTestId(TestId.NavExecutionsList).click();

  // Wait for and verify the execution appears in the list
  const executionLink = page.getByTestId(TestId.ExecutionLink).filter({ hasText: executionName });
  await expect(executionLink).toBeVisible({ timeout: 5000 });

  // Click on the execution to view details
  await executionLink.click();

  // Verify we're on the execution detail page
  await expect(page.getByTestId(TestId.ExecutionDetailTitle)).toContainText(executionName);

  // Verify the execution ID is displayed on the page
  await expect(page.getByTestId(TestId.ExecutionDetailId)).toContainText(executionId);

  await server.stop();
});
