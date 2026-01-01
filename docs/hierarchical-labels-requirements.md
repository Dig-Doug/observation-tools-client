# Requirements: Hierarchical Labels for Observation Grouping

## 1. Overview

### 1.1 Summary

This document specifies the requirements for organizing observations hierarchically using label-based grouping. The
feature adds a tree-view UI, a new backend API endpoint, and tracing integration for automatic label generation.

### 1.2 Goals

- Enable users to organize related observations into hierarchical groups
- Provide a tree-view UI for navigating grouped observations
- Integrate with the Rust `tracing` crate for automatic span-based labeling
- Maintain backward compatibility with existing label functionality

### 1.3 Scope

This feature is MVP-complete and includes:
- New grouped tree view in the Web UI
- New `/executions/{id}/tree` API endpoint
- Rust tracing layer for automatic label generation
- Minimal TypeScript client updates

---

## 2. Label System

### 2.1 Label Format

Labels use "/" as a hierarchy separator, similar to file paths:

```
{component}/{subcomponent}/{...}/{leaf}
```

**Examples:**
- `http/request/headers` - HTTP request headers
- `http/request/body` - HTTP request body
- `algorithm/quicksort/partition/0` - First partition step
- `process_request/{span_id}/validate` - Tracing-generated label

### 2.2 Multiple Labels

- Observations MAY have multiple labels
- When an observation has multiple labels, it appears in ALL corresponding groups in the tree view
- Each appearance links to the same detail panel (no group-specific context)

### 2.3 Label Validation

- **No validation is performed** on label strings
- Labels are accepted as-is; users are responsible for consistent formatting
- Empty string labels: observation appears in "Root" section
- Whitespace-only labels: treated as a regular group (whitespace is the group name)

### 2.4 Label Depth

- Maximum label depth is **configurable** via server configuration
- When a label exceeds the configured maximum depth:
  - A **warning is logged** on the server
  - The observation is **stored with the full label** (not truncated or rejected)
- Default maximum depth: TBD (recommend 10-20 levels)

---

## 3. User Interface

### 3.1 Grouped View Tab

Add a third tab alongside existing views:

```
[log] [payload] [grouped]
```

### 3.2 Tree Structure

The grouped view displays observations in a collapsible tree structure:

```
Root (2)
  ├─ unlabeled obs 1     text/plain        10:30:00
  └─ unlabeled obs 2     application/json  10:30:01

▼ http (3)
  ├─ ▼ request (2)
  │    ├─ headers        text/plain        10:30:01
  │    └─ body           application/json  10:30:01
  └─ response            application/json  10:30:02

▼ process_request/{span_id} (0)  [grayed - empty group]
  └─ ▼ validate (1)
       └─ input          application/json  10:30:03

▶ algorithm (100+)      [collapsed - pagination available]
```

### 3.3 Tree Behavior

| Behavior | Specification |
|----------|---------------|
| Hierarchy depth | Full hierarchy - nested collapsible tree matching full label path depth |
| Default expand state | **Smart expand**: expand levels if total visible items stays under 50 |
| Group ordering | **Creation order** - by first observation/span creation timestamp |
| Header click | Clicking anywhere on group header toggles expand/collapse |
| Empty groups | Displayed with visual distinction (grayed out/dimmed styling) |
| Root section | Observations without labels appear in "Root" section at top |

### 3.4 Observation Row Display

Each observation row displays:
- Label leaf name (last path component)
- Content type (MIME type)
- Timestamp
- **Truncated payload preview** (first N characters/lines)

### 3.5 Pagination

- Groups with many observations show first **100 items**
- "Load more" button appears after initial items
- Pagination is **per-group** (each group has independent pagination state)

### 3.6 View Synchronization

- Selection **syncs between log view and grouped view**
- Selecting an observation in grouped view highlights/scrolls to it in log view
- Selecting an observation in log view highlights it in grouped view (if visible)

### 3.7 State Persistence

- **No URL persistence** - view state resets on page refresh
- Expanded/collapsed state and pagination position are session-only

### 3.8 Detail Panel

- Clicking an observation opens the detail side panel (same as other views)
- When observation appears in multiple groups, all appearances open the same detail panel

### 3.9 Accessibility

- **Deferred** to future iteration
- MVP uses standard click/tap interactions only

### 3.10 Search/Filter

- **No search or filter** functionality in grouped view for MVP
- Users should use existing log view search to find specific observations

---

## 4. API Specification

### 4.1 New Endpoint

```
GET /api/exe/{execution_id}/tree
```

Returns a paginated tree structure of observations grouped by label hierarchy.

### 4.2 Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| execution_id | string (path) | Yes | Execution ID |

**Note:** No filter or sort parameters for MVP.

### 4.3 Response Schema

```json
{
  "tree": {
    "groups": [
      {
        "name": "root",
        "path": [],
        "is_empty": false,
        "observation_count": 2,
        "observations": [
          {
            "id": "obs-id-1",
            "name": "unlabeled obs",
            "mime_type": "text/plain",
            "created_at": "2024-01-15T10:30:00Z",
            "preview": "First 100 chars of payload...",
            "labels": []
          }
        ],
        "has_more_observations": false,
        "next_cursor": null,
        "children": []
      },
      {
        "name": "http",
        "path": ["http"],
        "is_empty": false,
        "observation_count": 3,
        "observations": [],
        "has_more_observations": false,
        "next_cursor": null,
        "children": [
          {
            "name": "request",
            "path": ["http", "request"],
            "is_empty": false,
            "observation_count": 2,
            "observations": [
              {
                "id": "obs-id-2",
                "name": "headers",
                "mime_type": "text/plain",
                "created_at": "2024-01-15T10:30:01Z",
                "preview": "Content-Type: application/json...",
                "labels": ["http/request/headers"]
              }
            ],
            "has_more_observations": false,
            "next_cursor": null,
            "children": []
          }
        ]
      },
      {
        "name": "process_request",
        "path": ["process_request"],
        "is_empty": true,
        "observation_count": 0,
        "observations": [],
        "has_more_observations": false,
        "next_cursor": null,
        "children": [
          {
            "name": "abc123",
            "path": ["process_request", "abc123"],
            "is_empty": false,
            "observation_count": 1,
            "observations": [...],
            "has_more_observations": false,
            "next_cursor": null,
            "children": []
          }
        ]
      }
    ],
    "total_observations": 6,
    "total_groups": 5
  }
}
```

### 4.4 Pagination - Load More Endpoint

```
GET /api/exe/{execution_id}/tree/group?path={encoded_path}&cursor={cursor}
```

Fetches additional observations for a specific group.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| execution_id | string (path) | Yes | Execution ID |
| path | string (query) | Yes | URL-encoded label path (e.g., "http/request") |
| cursor | string (query) | Yes | Pagination cursor from previous response |

**Response:**
```json
{
  "observations": [...],
  "has_more_observations": true,
  "next_cursor": "cursor-token"
}
```

### 4.5 Observation Metadata in Tree Response

The tree response returns **metadata only** - no inline payloads:
- `id` - Observation ID
- `name` - Observation name
- `mime_type` - Payload MIME type
- `created_at` - Creation timestamp
- `preview` - Truncated preview of payload (first 100-200 chars)
- `labels` - Full labels array

Full payload is fetched separately via existing endpoint:
```
GET /api/exe/{execution_id}/obs/{observation_id}/content
```

### 4.6 Response Schema Definitions

Add to OpenAPI spec:

```yaml
TreeResponse:
  type: object
  required: [tree]
  properties:
    tree:
      $ref: '#/components/schemas/ObservationTree'

ObservationTree:
  type: object
  required: [groups, total_observations, total_groups]
  properties:
    groups:
      type: array
      items:
        $ref: '#/components/schemas/TreeGroup'
    total_observations:
      type: integer
      minimum: 0
    total_groups:
      type: integer
      minimum: 0

TreeGroup:
  type: object
  required: [name, path, is_empty, observation_count, observations, has_more_observations, children]
  properties:
    name:
      type: string
      description: Group name (last component of path, or "root" for unlabeled)
    path:
      type: array
      items:
        type: string
      description: Full path components for this group
    is_empty:
      type: boolean
      description: True if group has no observations (only child groups)
    observation_count:
      type: integer
      minimum: 0
      description: Total observations in this group (not including children)
    observations:
      type: array
      items:
        $ref: '#/components/schemas/TreeObservation'
      description: First page of observations (up to 100)
    has_more_observations:
      type: boolean
      description: True if more observations available via pagination
    next_cursor:
      type: string
      nullable: true
      description: Cursor for fetching next page, null if no more
    children:
      type: array
      items:
        $ref: '#/components/schemas/TreeGroup'
      description: Child groups

TreeObservation:
  type: object
  required: [id, name, mime_type, created_at, preview, labels]
  properties:
    id:
      $ref: '#/components/schemas/ObservationId'
    name:
      type: string
    mime_type:
      type: string
    created_at:
      type: string
      format: date-time
    preview:
      type: string
      description: Truncated payload preview
    labels:
      type: array
      items:
        type: string

GroupPaginationResponse:
  type: object
  required: [observations, has_more_observations]
  properties:
    observations:
      type: array
      items:
        $ref: '#/components/schemas/TreeObservation'
    has_more_observations:
      type: boolean
    next_cursor:
      type: string
      nullable: true
```

---

## 5. Tracing Integration

### 5.1 Overview

The Rust client provides a `tracing` layer that automatically generates hierarchical labels from span hierarchy.

### 5.2 Label Generation

- Span hierarchy is encoded in the label path using **span IDs** for uniqueness
- Format: `{span_name}/{span_id}/{child_span_name}/{child_span_id}/...`

**Example:**
```rust
#[instrument]
fn process_request() {  // span_id = "abc123"
    #[instrument]
    fn validate() {     // span_id = "def456"
        observe!("input", data);
        // Generated label: "process_request/abc123/validate/def456"
    }
}
```

### 5.3 Span Capture Behavior

- **All spans are captured** as groups in the tree, even if they contain no observations
- Empty span groups appear in the tree with `is_empty: true` and grayed-out styling
- This provides context about the execution structure even when observations are sparse

### 5.4 Span ID Format

- Uses tracing's native span ID (typically a u64 formatted as hex)
- Example: `process_request/a1b2c3d4`

### 5.5 Combining User Labels with Span Labels

When a user adds explicit labels to an observation within a span context:
- The span path becomes the **prefix**
- User label is appended
- Observation appears in **both** the span-derived group AND the user-specified label group

**Example:**
```rust
#[instrument]
fn process_request() {
    observe!("data", value).label("custom/category");
    // Labels: ["process_request/abc123", "custom/category"]
    // Appears in both groups in tree view
}
```

### 5.6 Tracing Layer Configuration

```rust
use observation_tools::tracing::ObservationLayer;

let layer = ObservationLayer::new(client)
    .with_span_labels(true);  // Enable span-based label generation

tracing_subscriber::registry()
    .with(layer)
    .init();
```

---

## 6. Client API

### 6.1 Rust Client

**Label method (string path):**
```rust
let obs = client.observe("request data", payload)
    .label("http/request/body")
    .send();
```

**Multiple labels:**
```rust
let obs = client.observe("request data", payload)
    .label("http/request/body")
    .label("debug/verbose")
    .send();
```

No builder pattern for constructing paths - users provide the full path string.

### 6.2 TypeScript Client

**Minimal implementation for MVP:**
```typescript
const obs = client.observe("request data", payload)
  .label("http/request/body");
```

Only the basic `label(path: string)` method is required.
No tracing integration for TypeScript in MVP.

---

## 7. Server Configuration

### 7.1 New Configuration Options

```toml
[labels]
# Maximum allowed depth for label hierarchies
# Labels exceeding this depth trigger a warning but are stored as-is
max_depth = 20

# Maximum characters for payload preview in tree response
preview_length = 200
```

---

## 8. Data Model

### 8.1 Schema Changes

**No schema changes required.** The existing `labels: Vec<String>` field on `Observation` is sufficient.

### 8.2 Indexing Considerations

For efficient tree construction, consider adding:
- Index on `labels` array field for prefix queries
- Index on `created_at` for ordering within groups

---

## 9. Error Handling

### 9.1 API Errors

| Scenario | HTTP Status | Error Response |
|----------|-------------|----------------|
| Execution not found | 404 | `{"error": "Execution not found"}` |
| Invalid cursor | 400 | `{"error": "Invalid pagination cursor"}` |
| Invalid path encoding | 400 | `{"error": "Invalid path parameter"}` |

### 9.2 Client Errors

- Label method accepts any string; no client-side validation
- Tracing layer logs warnings for unusually deep spans but does not fail

---

## 10. Performance Considerations

### 10.1 Tree Construction

- Tree is constructed server-side from stored observations
- No specific performance requirements defined
- Graceful degradation expected for very large datasets

### 10.2 Pagination

- Initial tree response includes first 100 observations per group
- Per-group pagination prevents loading entire large groups upfront

### 10.3 Caching

- Tree structure may be cached per execution (implementation detail)
- Cache invalidation required when observations are added

---

## 11. Future Considerations (Out of Scope)

The following are explicitly NOT part of this MVP:

- Search/filter in grouped view
- URL state persistence for sharing
- Accessibility (ARIA tree roles, keyboard navigation)
- TypeScript tracing integration
- Drag-and-drop reordering of groups
- Custom group ordering
- Group metadata/annotations
- Bulk operations on groups
- Export grouped view

---

## 12. Open Questions Resolved

| Question | Resolution |
|----------|------------|
| Multiple requests to same service | Use span ID in label path for uniqueness |
| Span hierarchy + user labels | Span path as prefix, user labels additive |
| Large groups handling | Pagination at 100 items with "Load more" |
| Model spans from tracing | Encoded in label convention with span IDs |

---

## 13. Acceptance Criteria

### 13.1 UI

- [ ] Grouped tab appears alongside log and payload tabs
- [ ] Tree displays hierarchical structure based on label paths
- [ ] Groups expand/collapse on header click
- [ ] Empty groups display with grayed-out styling
- [ ] Observations without labels appear in "Root" section
- [ ] Smart expand collapses deeper levels when >50 items visible
- [ ] Pagination works per-group with "Load more" after 100 items
- [ ] Selection syncs between log view and grouped view
- [ ] Observation rows show name, mime type, timestamp, and preview

### 13.2 API

- [ ] `GET /api/exe/{id}/tree` returns paginated tree structure
- [ ] `GET /api/exe/{id}/tree/group` loads additional observations
- [ ] Observations appear in multiple groups when they have multiple labels
- [ ] Empty span groups included in tree with `is_empty: true`
- [ ] Tree ordered by creation timestamp

### 13.3 Tracing Integration

- [ ] `ObservationLayer` generates labels from span hierarchy
- [ ] Span IDs included in label path for uniqueness
- [ ] All spans captured (including empty ones)
- [ ] User labels combine with span-derived labels

### 13.4 Client

- [ ] Rust client `.label(path)` method works with "/" paths
- [ ] TypeScript client `.label(path)` method available
- [ ] Multiple labels can be added to single observation
