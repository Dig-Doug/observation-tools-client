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
      key: "value"
    },
    emptyArray: [],
    emptyObject: {},
    longStringShouldWrap: "a".repeat(500),
    nullValue: null,
    malicious: "<script>alert(\"XSS\")</script>",
    imgXss: "<img src=\"x\" onerror=\"alert('XSS')\">"
  });

  const handle = new ObservationBuilder(observationName).jsonPayload(jsonContent).send(exe);

  await page.goto(handle.handle().url);
  const payloadElement = page.getByTestId(TestId.ObservationPayload);
  await expect(payloadElement).toBeVisible();
  await expect(payloadElement).toHaveScreenshot();
  await expect(payloadElement.locator("script")).toHaveCount(0);
  await expect(payloadElement).toContainText("<script>");
});

test("JSON objects are collapsible", async ({ page, server }) => {
  const client = server.createClient();
  const executionName = "execution-with-collapsible-json";
  const exe = client.beginExecution(executionName);

  const observationName = "collapsible-json-observation";
  const jsonContent = JSON.stringify({
    outer: {
      inner: "value"
    }
  });

  const handle = new ObservationBuilder(observationName).jsonPayload(jsonContent).send(exe);

  await page.goto(handle.handle().url);
  const payloadElement = page.getByTestId(TestId.ObservationPayload);

  await expect(payloadElement).toBeVisible();
  const details = page.getByTestId(TestId.JsonCollapsibleArea).first();
  await expect(details).toHaveAttribute("open", "");
  const collapseToggle = details.getByTestId(TestId.JsonCollapseToggle).first();
  await collapseToggle.click();
  await expect(details).not.toHaveAttribute("open", "");
  await collapseToggle.click();
  await expect(details).toHaveAttribute("open", "");
});
