---
name: debug-mode-logging
description: >-
  Adds extra debug logs to the console/`stderr` when the system enters DEBUG MODE,
  as a complement (never a replacement) for existing HTTP/file logging instrumentation.
---

# Debug Mode Logging

## When to use
Use this skill when the system indicates `DEBUG MODE` via the `system_reminder` text:

`"You are now in **DEBUG MODE**. You must debug with **runtime evidence**."`

In DEBUG MODE, add additional *console/terminal* logs as runtime evidence, **in parallel** with the existing instrumentation (HTTP logging endpoint and/or log file).

## Workflow (must follow)
1. Detect DEBUG MODE from the `system_reminder` message above.
2. Add an extra debug log that appears in the developer console/terminal:
   - JavaScript/TypeScript: use `console.log()` or `console.error()` (where appropriate).
   - Other languages: use `stderr` (or the language equivalent).
3. Prefix every message with `[DEBUG]`.
4. Include the same information as the instrumentation logs:
   - `location` (file:LINE or equivalent)
   - `message` (short description of the hypothesis/state)
   - `data` (key/value payload relevant to the debug statement)
   - `timestamp`
   - `sessionId`
   - `hypothesisId`
5. Never replace existing instrumentation. Console/stderr logs are **complementary**.

## Required log format (console/stderr)
Format (human readable, consistent fields):

`[DEBUG] location=<file>:<LINE> message=<desc> data={<k>:<v>} timestamp=<epoch> sessionId=<id> hypothesisId=<id>`

Notes:
- `timestamp` should be in epoch milliseconds/seconds depending on the language.
- `data` must be JSON-like so it is easy to scan.

## Language-specific snippets

### JavaScript / TypeScript
```ts
console.log('[DEBUG]', {
  location: 'file.js:LINE',
  message: 'desc',
  data: { k: v },
  timestamp: Date.now(),
  sessionId: 'debug-session',
  hypothesisId: 'A'
});
```

### Python
```py
import sys, time

print(
  f'[DEBUG] location=file.py:LINE message=desc data={{"k": v}} timestamp={time.time()} sessionId=debug-session hypothesisId=A',
  file=sys.stderr
)
```

### Java / Groovy
```java
System.err.println(String.format(
  "[DEBUG] location=file.java:LINE message=desc data={k:v} timestamp=%d sessionId=debug-session hypothesisId=A",
  System.currentTimeMillis()
));
```

### Ruby
```rb
STDERR.puts "[DEBUG] location=file.rb:LINE message=desc data={k:v} timestamp=#{Time.now.to_i} sessionId=debug-session hypothesisId=A"
```

### Go
```go
fmt.Fprintf(
  os.Stderr,
  "[DEBUG] location=file.go:LINE message=desc data={k:v} timestamp=%d sessionId=debug-session hypothesisId=A\n",
  time.Now().Unix()
)
```

### Rust
```rs
eprintln!(
  "[DEBUG] location=file.rs:LINE message=desc data={{k:v}} timestamp={} sessionId=debug-session hypothesisId=A",
  chrono::Utc::now().timestamp()
);
```
