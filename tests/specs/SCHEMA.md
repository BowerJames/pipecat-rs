## Pipecat-RS Test DSL — v0.1 Schema

Behavior-first DSL for black-box testing of streaming pipelines. External services are simulated; tests assert observable messages/traces, not implementation details. Inspired by Pipecat’s message orchestration model [pipecat-ai/pipecat](https://github.com/pipecat-ai/pipecat).

### Top-level
- **version**: integer. DSL schema version.
- **name**: string. Human-readable scenario name.
- **fail_after**: duration (e.g., `100ms`, `2s`). Hard timeout for the scenario.
- **time_epsilon**: optional duration (default `5ms`). Timing tolerance for comparisons.
- **default_within**: optional duration. Default assertion window for `await` if not provided.

### Durations
- Strings with units: `Nms`, `Ns`. Examples: `10ms`, `1s`, `1500ms`.

### Pipeline
Array of nodes forming a linear pipeline. By default, messages flow `downstream` from the first node to the last; `upstream` is the reverse direction.

```yaml
pipeline:
  - { id: input,  kind: transport@simulated@input }
  - { id: echo,   kind: echo }
  - { id: output, kind: transport@simulated@output }
```

Node fields:
- **id**: string. Unique node identifier (referenced by steps via `node`).
- **kind**: string. Implementation or simulator type (e.g., `echo`, `transport@simulated@input`).
- **config**: optional object. Node-specific configuration.

### Script
Ordered list of steps. v0.1 supports a linear timeline (no parallel blocks yet).

Each step is one of:
- **await**: Assert that a matching message appears within a time window.
- **send**: Inject a message at a node after some virtual time elapses.

#### Common fields
- **node**: string. Node id from `pipeline`.
- **direction**: `downstream` | `upstream`.
  - `downstream` means along pipeline order (towards later nodes).
  - `upstream` means opposite pipeline order (towards earlier nodes).
- **pattern**: Message selector.
  - **type**: string. Message type (e.g., `text_input`, `text_output`).
  - **body**: object. Message payload subset to match against.

Matching rules:
- Type must match exactly.
- Body is a partial, structural match: all specified fields must be present and equal in the actual message. Unspecified fields are ignored in v0.1.
- Strings match literally in v0.1. Regex/globs may be added in a later version.

#### await step
Asserts that a message matching `pattern` is observed on the specified `direction` at `node` within a time window.

Fields:
- **within**: duration. Deadline relative to the step’s start time (see scheduling).

Example:
```yaml
- op: await
  node: output
  direction: downstream
  pattern:
    type: text_output
    body: { text: "Hello, world!" }
  within: 60ms
```

#### send step
Schedules injection of a message at `node` in a given pipeline `direction` after some virtual time.

Fields:
- **after**: duration. Relative delay from the current virtual time to schedule the send.

Example:
```yaml
- op: send
  node: input
  direction: downstream
  after: 10ms
  pattern:
    type: text_input
    body: { text: "Hello, world!" }
```

### Virtual time, scheduling, and determinism
- Virtual clock starts at `t=0`.
- `after` delays in `send` are applied relative to the current virtual time at the moment the step is processed.
- `within` windows for `await` are measured from the step’s start time.
- At any timestamp, processing order is deterministic:
  1. All due `send` injections are emitted into the pipeline.
  2. The pipeline processes resulting messages synchronously to produce observable outputs on each node/direction.
  3. Pending `await` steps are evaluated against newly observed messages; matching ones complete.
- If an `await` does not match before its `within` window closes, it fails.
- The entire scenario fails if not finished before `fail_after`.

### Failure reporting (contract)
On failure, runners should emit a structured error including:
- **step_index**: integer (0-based in `script`).
- **step**: the step that failed.
- **reason**: one of `timeout`, `mismatch`, `unexpected`, `internal_error`.
- **observed**: optional array of messages seen in the relevant window.
- **expected**: the `pattern` that was awaited.

Example (JSON):
```json
{
  "step_index": 3,
  "reason": "timeout",
  "expected": { "type": "text_output", "body": { "text": "Hello, world!" } },
  "observed": [ { "type": "text_output", "body": { "text": "Hi" }, "t": "55ms" } ]
}
```

### Reserved keys (v0.1)
Top-level: `version`, `name`, `fail_after`, `time_epsilon`, `default_within`, `pipeline`, `script`.
Pipeline node: `id`, `kind`, `config`.
Script step: `op`, `await`, `send`, `node`, `direction`, `after`, `within`, `pattern`, `type`, `body`.

### Minimal valid example
```yaml
version: 1
name: echo
fail_after: 100ms

pipeline:
  - { id: input,  kind: transport@simulated@input }
  - { id: echo,   kind: echo }
  - { id: output, kind: transport@simulated@output }

script:
  - op: await
    node: input
    direction: downstream
    pattern:
      type: text_input
      body: { text: "Hello, world!" }
    within: 50ms

  - op: send
    node: input
    direction: downstream
    after: 10ms
    pattern:
      type: text_input
      body: { text: "Hello, world!" }

  - op: await
    node: echo
    direction: downstream
    pattern:
      type: text_input
      body: { text: "Hello, world!" }
    within: 50ms

  - op: send
    node: echo
    direction: downstream
    after: 10ms
    pattern:
      type: text_output
      body: { text: "Hello, world!" }

  - op: await
    node: output
    direction: downstream
    pattern:
      type: text_output
      body: { text: "Hello, world!" }
    within: 60ms
```

### Future extensions (non-normative)
- Concurrency primitives: `par`, `race`, `until` blocks.
- Golden trace capture/diff for regression.
- Rich matchers: regex/glob, numeric ranges, partial arrays, token streams.
- Backpressure and pacing controls in simulators.
- Graph pipelines with explicit edges (vs linear order only).


