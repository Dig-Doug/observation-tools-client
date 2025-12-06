import { expect, test } from "../fixtures";
import { ObservationBuilder } from "observation-tools-client";
import { TestId } from "../helpers/testIds";

test("JSON observation is rendered with syntax highlighting", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "execution-with-json";
  const exe = client.beginExecution(executionName);

  const observationName = "json-observation";
  const jsonContent = JSON.stringify({
    name: "test",
    count: 42,
    active: true,
    items: ["a", "b", "c"],
    nested: {
      key: "value",
    },
  });

  const handle = new ObservationBuilder(observationName).jsonPayload(jsonContent).send(exe);

  await page.goto(handle.handle().url);
  const payloadElement = page.getByTestId(TestId.ObservationPayload);
  await expect(payloadElement).toBeVisible();

  // Check for syntax highlighting classes - verify they exist
  const jsonStrings = payloadElement.locator(".json-string");
  expect(await jsonStrings.count()).toBeGreaterThanOrEqual(5);

  // Verify specific values appear
  await expect(payloadElement).toContainText('"test"');
  await expect(payloadElement).toContainText("42");
  await expect(payloadElement).toContainText("true");

  // Verify keys have correct class
  const jsonKeys = payloadElement.locator(".json-key");
  expect(await jsonKeys.count()).toBeGreaterThanOrEqual(1);
});

test("JSON objects are collapsible", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "execution-with-collapsible-json";
  const exe = client.beginExecution(executionName);

  const observationName = "collapsible-json-observation";
  const jsonContent = JSON.stringify({
    outer: {
      inner: "value",
    },
  });

  const handle = new ObservationBuilder(observationName).jsonPayload(jsonContent).send(exe);

  await page.goto(handle.handle().url);
  const payloadElement = page.getByTestId(TestId.ObservationPayload);
  await expect(payloadElement).toBeVisible();

  // Check that details elements exist and are expanded by default
  const details = payloadElement.locator("> details");
  await expect(details).toHaveAttribute("open", "");

  // Click to collapse the root object using the direct child summary
  await details.locator("> summary").click();
  await expect(details).not.toHaveAttribute("open", "");
});

test("JSON arrays display item count", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "execution-with-json-array";
  const exe = client.beginExecution(executionName);

  const observationName = "json-array-observation";
  const jsonContent = JSON.stringify([1, 2, 3, 4, 5]);

  const handle = new ObservationBuilder(observationName).jsonPayload(jsonContent).send(exe);

  await page.goto(handle.handle().url);
  const payloadElement = page.getByTestId(TestId.ObservationPayload);
  await expect(payloadElement).toBeVisible();

  // Check for item count in preview
  await expect(payloadElement.locator(".json-preview")).toContainText("5 items");
});

test("JSON strings wrap long content", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "execution-with-long-json-string";
  const exe = client.beginExecution(executionName);

  const observationName = "long-string-json-observation";
  const longString = "a".repeat(500);
  const jsonContent = JSON.stringify({ longValue: longString });

  const handle = new ObservationBuilder(observationName).jsonPayload(jsonContent).send(exe);

  await page.goto(handle.handle().url);
  const payloadElement = page.getByTestId(TestId.ObservationPayload);
  await expect(payloadElement).toBeVisible();

  // Verify the long string is present
  await expect(payloadElement.locator(".json-string").last()).toContainText(longString);
});

test("JSON observation sanitizes malicious content", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "execution-with-malicious-json";
  const exe = client.beginExecution(executionName);

  const observationName = "malicious-json-observation";
  const maliciousJson = JSON.stringify({
    safe: "normal value",
    malicious: '<script>alert("XSS")</script>',
    imgXss: '<img src="x" onerror="alert(\'XSS\')">',
  });

  const handle = new ObservationBuilder(observationName).jsonPayload(maliciousJson).send(exe);

  await page.goto(handle.handle().url);
  const payloadElement = page.getByTestId(TestId.ObservationPayload);
  await expect(payloadElement).toBeVisible();

  // Verify safe content is rendered
  await expect(payloadElement).toContainText("normal value");

  // Verify script tags are not rendered as HTML (i.e., escaped/shown as text)
  await expect(payloadElement.locator("script")).toHaveCount(0);

  // The malicious strings should appear as text content (escaped)
  await expect(payloadElement).toContainText("<script>");
});

test("Empty JSON object and array are rendered", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "execution-with-empty-json";
  const exe = client.beginExecution(executionName);

  const observationName = "empty-json-observation";
  const jsonContent = JSON.stringify({
    emptyObject: {},
    emptyArray: [],
  });

  const handle = new ObservationBuilder(observationName).jsonPayload(jsonContent).send(exe);

  await page.goto(handle.handle().url);
  const payloadElement = page.getByTestId(TestId.ObservationPayload);
  await expect(payloadElement).toBeVisible();

  // Check that empty structures are rendered as text
  await expect(payloadElement).toContainText("{}");
  await expect(payloadElement).toContainText("[]");
});

test("JSON null value is rendered", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "execution-with-null-json";
  const exe = client.beginExecution(executionName);

  const observationName = "null-json-observation";
  const jsonContent = JSON.stringify({ value: null });

  const handle = new ObservationBuilder(observationName).jsonPayload(jsonContent).send(exe);

  await page.goto(handle.handle().url);
  const payloadElement = page.getByTestId(TestId.ObservationPayload);
  await expect(payloadElement).toBeVisible();

  await expect(payloadElement.locator(".json-null")).toContainText("null");
});
