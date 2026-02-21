import { test, expect } from "../fixtures";
import { TestId } from "../helpers/testIds";
import {
  generateExecutionId,
  generateObservationId,
  ObservationBuilder,
} from "observation-tools-client";

test("Server homepage loads", async ({ page, server }) => {
  await page.goto(server.baseUrl);
  await expect(page.locator("h1")).toBeVisible();
  await expect(
    page.getByTestId(TestId.NavBarLogo),
    "Logo didn't load, are static files working?",
  ).toBeVisible();
});

test("Create execution", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "test-execution";
  const exe = client.beginExecution(executionName);
  const executionId = exe.idString;

  await page.goto(server.baseUrl);
  await page.getByTestId(TestId.NavExecutionsList).click();
  await page.getByTestId(TestId.ExecutionLink).filter({ hasText: executionName }).first().click();

  await expect(page.getByTestId(TestId.ExecutionDetailTitle)).toContainText(executionName);
  await expect(page.getByTestId(TestId.ExecutionDetailId)).toContainText(executionId);
});

test("Create execution with observation and verify data", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "execution-with-observation";
  const exe = client.beginExecution(executionName);
  const observationName = "test-observation";
  const observationPayload = { message: "Hello, World!", count: 42, nested: { value: true } };
  const observationGroups = ["api", "test"];
  const sourceFile = "test.ts";
  const sourceLine = 123;
  const observationMetadata = [
    ["environment", "testing"],
    ["version", "1.0.0"],
    ["user", "test-user"],
  ];
  let builder = new ObservationBuilder(observationName);
  for (const label of observationGroups) {
    builder = builder.group(label);
  }
  for (const [key, value] of observationMetadata) {
    builder = builder.metadata(key, value);
  }
  const observationHandle = builder
    .source(sourceFile, sourceLine)
    .jsonPayload(JSON.stringify(observationPayload))
    .send(exe)
    .handle();
  const observationId = observationHandle.id;

  await page.goto(server.baseUrl);
  await page.getByTestId(TestId.NavExecutionsList).click();
  await page.getByTestId(TestId.ExecutionLink).filter({ hasText: executionName }).first().click();
  expect(page.url()).toBe(exe.url);
  await page
    .getByTestId(TestId.ObservationListItemLink)
    .filter({ hasText: observationName })
    .click();

  await expect(page.getByTestId(TestId.ObservationId)).toContainText(observationId);
  await expect(page.getByTestId(TestId.ObservationPayload)).toBeVisible();
  await expect(page.getByTestId(TestId.ObservationPayload)).toContainText("Hello, World!");
  await expect(page.getByTestId(TestId.ObservationPayload)).toContainText("42");
  for (const label of observationGroups) {
    await expect(page.getByTestId(TestId.ObservationGroups)).toContainText(label);
  }
  await expect(page.getByTestId(TestId.ObservationSourceFile)).toContainText(sourceFile);
  await expect(page.getByTestId(TestId.ObservationSourceLine)).toContainText(sourceLine.toString());

  // Verify metadata is rendered
  await expect(page.getByTestId(TestId.ObservationMetadataHeader)).toBeVisible();
  await expect(page.getByTestId(TestId.ObservationMetadata)).toBeVisible();
  for (const [key, value] of observationMetadata) {
    await expect(page.getByTestId(TestId.ObservationMetadata)).toContainText(key);
    await expect(page.getByTestId(TestId.ObservationMetadata)).toContainText(value);
  }
});

test("Execution list pagination with 357 executions", async ({ page, server }) => {
  const client = server.createClient();
  const totalExecutions = 357;
  const pageSize = 100;
  for (let i = 0; i < totalExecutions; i++) {
    client.beginExecution(`execution-${i.toString().padStart(3, "0")}`);
  }

  async function expectExecutionLinkVisible(executionIndex: number) {
    const executionName = `execution-${executionIndex.toString().padStart(3, "0")}`;
    await expect(
      page.getByTestId(TestId.ExecutionLink).filter({ hasText: executionName }),
    ).toBeVisible();
  }

  await page.goto(server.baseUrl);
  await page.getByTestId(TestId.NavExecutionsList).click();
  const prevButton = page.getByTestId(TestId.PaginationPrev);
  await expect(prevButton).toBeDisabled();

  // Check that the new executions  are on the first 3 pages
  await expectExecutionLinkVisible(totalExecutions - 1);
  const nextButton = page.getByTestId(TestId.PaginationNext);
  await nextButton.click();
  await expectExecutionLinkVisible(totalExecutions - pageSize - 1);
  await nextButton.click();
  await expectExecutionLinkVisible(totalExecutions - pageSize * 2 - 1);

  // Navigate to the end and back
  // We allow testing against an external server, which may have other executions present so we don't assume a fixed number of pages
  while (await nextButton.isEnabled()) {
    await nextButton.click();
  }
  await expect(prevButton).toBeEnabled();
  while (await prevButton.isEnabled()) {
    await prevButton.click();
  }

  await expectExecutionLinkVisible(totalExecutions - 1);
});

test("Observation list pagination with 396 observations", async ({ page, server }) => {
  const client = server.createClient();
  const exe = client.beginExecution("execution-with-many-observations");
  const totalObservations = 396;
  for (let i = 0; i < totalObservations; i++) {
    new ObservationBuilder(`observation-${i.toString().padStart(3, "0")}`)
      .jsonPayload(JSON.stringify({ index: i, data: `test-data-${i}` }))
      .send(exe);
  }

  await page.goto(exe.url);
  await expect(page.getByTestId(TestId.ObservationListItem).first()).toBeVisible();
  // Wait for all observations to be received by the server (auto-refresh will pick them up)
  await expect(page.getByTestId(TestId.PaginationInfo)).toContainText(`of ${totalObservations}`, {
    timeout: 30000,
  });
  const observationItems = page.getByTestId(TestId.ObservationListItem);
  const count = await observationItems.count();
  expect(count).toBeGreaterThan(10);
  // Page 1
  const prevButton = page.getByTestId(TestId.PaginationPrev);
  const nextButton = page.getByTestId(TestId.PaginationNext);
  await expect(prevButton).toBeDisabled();
  await expect(nextButton).toBeEnabled();
  // Page 2
  await nextButton.click();
  await expect(prevButton).toBeEnabled();
  await expect(nextButton).toBeEnabled();
  // Page 3
  await nextButton.click();
  // Page 4
  await nextButton.click();
  await expect(nextButton).toBeDisabled();
  await expect(prevButton).toBeEnabled();

  // Navigate back to page 1
  await prevButton.click();
  await prevButton.click();
  await prevButton.click();
  await expect(prevButton).toBeDisabled();
});

test("Execution list auto-refresh", async ({ page, server }) => {
  const client = server.createClient();

  await page.goto(server.baseUrl);
  await page.getByTestId(TestId.NavExecutionsList).click();
  await expect(page.getByTestId(TestId.ExecutionsListEmpty)).toBeVisible();

  const executionName = "auto-refresh-test-execution";
  client.beginExecution(executionName);
  const executionLink = page.getByTestId(TestId.ExecutionLink).filter({ hasText: executionName });
  await expect(executionLink).toBeVisible();
  await expect(page.getByTestId(TestId.ExecutionsListEmpty)).not.toBeVisible();
});

test("Large payload is uploaded as blob", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "execution-with-large-payload";
  const exe = client.beginExecution(executionName);
  const observationName = "large-observation";

  // Create a payload larger than 64KB (the blob threshold)
  // The payload must be valid JSON, so we create an object with a large string field
  const largeData = "x".repeat(70000);
  const largePayload = { data: largeData, size: largeData.length };
  const observationId = new ObservationBuilder(observationName)
    .group("test")
    .group("large-payload")
    .jsonPayload(JSON.stringify(largePayload))
    .send(exe)
    .handle().id;

  // Wait a moment for the blob upload to complete
  await new Promise((resolve) => setTimeout(resolve, 1000));

  // Navigate to the observation page
  await page.goto(server.baseUrl);
  await page.getByTestId(TestId.NavExecutionsList).click();
  await page.getByTestId(TestId.ExecutionLink).filter({ hasText: executionName }).first().click();
  await page
    .getByTestId(TestId.ObservationListItemLink)
    .filter({ hasText: observationName })
    .click();

  // Verify the observation details are visible
  await expect(page.getByTestId(TestId.ObservationId)).toContainText(observationId);

  // The payload should be retrieved from blob storage and displayed
  // Even though it was stored as a blob, the UI should still show it
  await expect(page.getByTestId(TestId.ObservationPayload)).toBeVisible();
});

test("Navigate to execution page before execution exists, then create it", async ({
  page,
  server,
}) => {
  const executionId = generateExecutionId();
  await page.goto(`${server.baseUrl}/exe/${executionId}`);
  await expect(page.getByText("Waiting for execution...")).toBeVisible();
  await expect(page.getByText(executionId)).toBeVisible();

  const client = server.createClient();
  const executionName = "pre-navigation-test-execution";
  const exe = client.beginExecutionWithId(executionId, executionName);
  expect(exe.idString).toBe(executionId);
  await expect(page.getByTestId(TestId.ExecutionDetailTitle)).toContainText(executionName);
  await expect(page.getByTestId(TestId.ExecutionDetailId)).toContainText(executionId);
});

test("Navigate to observation page before observation exists, then create it", async ({
  page,
  server,
}) => {
  const client = server.createClient();
  const executionName = "pre-navigation-observation-test";
  const exe = client.beginExecution(executionName);
  const executionId = exe.idString;

  const observationId = generateObservationId();
  await page.goto(`${server.baseUrl}/exe/${executionId}/obs/${observationId}`);
  await expect(page.getByText("Waiting for observation...")).toBeVisible();
  await expect(page.getByText(observationId)).toBeVisible();

  const observationName = "pre-navigation-test-observation";
  const observationPayload = { message: "Created after navigation" };
  const createdId = new ObservationBuilder(observationName)
    .withId(observationId)
    .jsonPayload(JSON.stringify(observationPayload))
    .send(exe)
    .handle().id;
  expect(createdId).toBe(observationId);

  await expect(page.getByTestId(TestId.ObservationId)).toContainText(observationId);
  await expect(page.getByTestId(TestId.ObservationPayload)).toContainText(
    "Created after navigation",
  );
});

test.describe("Auto-refresh behavior", () => {
  test("Payload tab stays active during auto-refresh", async ({ page, server }) => {
    const client = server.createClient();
    const executionName = "payload-tab-refresh-test";
    const exe = client.beginExecution(executionName);

    // Create a payload observation so the payload tab has content
    await new ObservationBuilder("payload-obs")
      .jsonPayload(JSON.stringify({ data: "test-payload" }))
      .send(exe)
      .waitForUpload();

    // Navigate to execution detail
    await page.goto(exe.url);
    await expect(page.getByTestId(TestId.ExecutionDetailTitle)).toContainText(executionName);

    // Click the payload tab
    await page.getByTestId(TestId.ViewTabPayload).click();
    await expect(page.getByTestId(TestId.ViewTabPayload)).toHaveClass(/tab-active/);
    await expect(page.getByTestId(TestId.ViewTabLog)).not.toHaveClass(/tab-active/);

    // Wait for multiple auto-refresh cycles (refresh every 2s)
    await page.waitForTimeout(5000);

    // Verify payload tab is still active after auto-refresh
    await expect(page.getByTestId(TestId.ViewTabPayload)).toHaveClass(/tab-active/);
    await expect(page.getByTestId(TestId.ViewTabLog)).not.toHaveClass(/tab-active/);
  });

  for (const tab of [
    { name: "log", tabTestId: TestId.ViewTabLog },
    { name: "payload", tabTestId: TestId.ViewTabPayload },
  ]) {
    test(`Pagination stays on current page during auto-refresh (${tab.name} tab)`, async ({
      page,
      server,
    }) => {
      const client = server.createClient();
      const exe = client.beginExecution(`pagination-refresh-${tab.name}`);
      const totalObservations = 250;
      const uploads = [];
      for (let i = 0; i < totalObservations; i++) {
        uploads.push(
          new ObservationBuilder(`obs-${i.toString().padStart(3, "0")}`)
            .jsonPayload(JSON.stringify({ index: i }))
            .send(exe)
            .waitForUpload(),
        );
      }
      await Promise.all(uploads);

      // Navigate to execution — all observations should already be uploaded
      await page.goto(exe.url);
      await page.getByTestId(tab.tabTestId).click();
      await expect(page.getByTestId(TestId.PaginationInfo)).toContainText(
        `of ${totalObservations}`,
      );

      // Navigate to page 2
      await page.getByTestId(TestId.PaginationNext).click();
      await expect(page.getByTestId(TestId.PaginationInfo)).toContainText("page 2");
      await expect(page.getByTestId(TestId.PaginationPrev)).toBeEnabled();

      // Wait for multiple auto-refresh cycles
      await page.waitForTimeout(5000);

      // Verify still on page 2 after auto-refresh
      await expect(page.getByTestId(TestId.PaginationInfo)).toContainText("page 2");
      await expect(page.getByTestId(TestId.PaginationPrev)).toBeEnabled();
    });

    test(`Side panel stays open during auto-refresh (${tab.name} tab)`, async ({
      page,
      server,
    }) => {
      const client = server.createClient();
      const exe = client.beginExecution(`side-panel-refresh-${tab.name}`);

      // Create multiple observations
      const observation1Name = "first-observation";
      const observation1Payload = { message: "First observation data" };
      const observation1Id = new ObservationBuilder(observation1Name)
        .group("test")
        .source("test.ts", 10)
        .jsonPayload(JSON.stringify(observation1Payload))
        .send(exe)
        .handle().id;

      const observation2Name = "second-observation";
      const observation2Payload = { message: "Second observation data" };
      const observation2Id = new ObservationBuilder(observation2Name)
        .group("test")
        .source("test.ts", 20)
        .jsonPayload(JSON.stringify(observation2Payload))
        .send(exe)
        .handle().id;

      // Navigate to the execution detail page
      await page.goto(exe.url);
      await page.getByTestId(tab.tabTestId).click();
      await expect(page.getByTestId(TestId.ExecutionDetailTitle)).toContainText(
        `side-panel-refresh-${tab.name}`,
      );

      // Click on the first observation to open the side panel
      await page
        .getByTestId(TestId.ObservationListItemLink)
        .filter({ hasText: observation1Name })
        .click();

      // Verify the side panel is open with the first observation
      await expect(page.getByTestId(TestId.ObservationId)).toContainText(observation1Id);
      await expect(page.getByTestId(TestId.ObservationPayload)).toContainText(
        "First observation data",
      );

      // Wait 5 seconds to span multiple auto-refresh cycles (refresh happens every 2 seconds)
      await page.waitForTimeout(5000);

      // Verify the side panel is still open with the same observation
      await expect(page.getByTestId(TestId.ObservationId)).toContainText(observation1Id);
      await expect(page.getByTestId(TestId.ObservationPayload)).toContainText(
        "First observation data",
      );

      // Click on the second observation
      await page
        .getByTestId(TestId.ObservationListItemLink)
        .filter({ hasText: observation2Name })
        .click();

      // Verify the side panel shows the second observation
      await expect(page.getByTestId(TestId.ObservationId)).toContainText(observation2Id);
      await expect(page.getByTestId(TestId.ObservationPayload)).toContainText(
        "Second observation data",
      );

      // Wait another 5 seconds
      await page.waitForTimeout(5000);

      // Verify the side panel is still open with the second observation
      await expect(page.getByTestId(TestId.ObservationId)).toContainText(observation2Id);
      await expect(page.getByTestId(TestId.ObservationPayload)).toContainText(
        "Second observation data",
      );
    });

    test(`Clicking observation to open side panel preserves current page (${tab.name} tab)`, async ({
      page,
      server,
    }) => {
      const client = server.createClient();
      const exe = client.beginExecution(`pagination-side-panel-${tab.name}`);

      // Create enough observations for multiple pages (100 per page by default)
      const totalObservations = 150;
      for (let i = 0; i < totalObservations; i++) {
        new ObservationBuilder(`observation-${i.toString().padStart(3, "0")}`)
          .jsonPayload(JSON.stringify({ index: i }))
          .send(exe);
      }

      // Navigate to the execution detail page
      await page.goto(exe.url);
      await page.getByTestId(tab.tabTestId).click();
      await expect(page.getByTestId(TestId.ObservationListItem).first()).toBeVisible();

      // Wait for all observations to be received by the server
      await expect(page.getByTestId(TestId.PaginationInfo)).toContainText(
        `of ${totalObservations}`,
        {
          timeout: 30000,
        },
      );

      // Navigate to page 2
      const nextButton = page.getByTestId(TestId.PaginationNext);
      await nextButton.click();
      await expect(page.getByTestId(TestId.PaginationInfo)).toContainText("(page 2)");

      // Click on an observation to open the side panel
      const observationLink = page.getByTestId(TestId.ObservationListItemLink).first();
      await observationLink.click();

      // Verify the side panel opened
      await expect(page.getByTestId(TestId.ObservationId)).toBeVisible();

      // Verify we're still on page 2
      await expect(page.getByTestId(TestId.PaginationInfo)).toContainText("(page 2)");
    });
  }
});
