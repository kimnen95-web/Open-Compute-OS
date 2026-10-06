# Sessions

A session is the user's open work: which applications, which documents, where the cursor is, and what was not saved yet.

The phone image and the desktop image each have their own build of the app. Connecting hands the snapshot to the other build. The document and cursor stay. The CPU that runs the app changes.

## Snapshot

```json
{
  "session_id": "ses_demo",
  "owner_node_id": "node_…",
  "runtime_node_id": "node_…",
  "runtime_arch": "aarch64",
  "mode": "mobile",
  "windows": [
    {
      "app_id": "os.ocos.notes",
      "title": "Notes — welcome",
      "document_id": "welcome",
      "cursor": 42,
      "draft": "Written on the phone. Still here on the desktop.",
      "bounds": null
    }
  ]
}
```

| Field | Meaning |
| --- | --- |
| `session_id` | Stable id for this piece of open work |
| `owner_node_id` | Node that owns the user data. The phone, on the first pair |
| `runtime_node_id` | Node whose app build currently has the windows open |
| `runtime_arch` | `aarch64` on the Pixel image, `x86_64` on the Ubuntu image |
| `mode` | `mobile` or `desktop` |
| `windows` | Open windows. Empty is a valid session |
| `app_id` | Stable application id. Notes is `os.ocos.notes` |
| `cursor` | Document offset the user had reached |
| `draft` | Uncommitted text or other small state |
| `bounds` | Optional `x`, `y`, `width`, `height` for a desktop shell |

`mode` may change when a cable is plugged or removed. `windows` may not be cleared as a side effect of that change. `ocos_session::SessionSnapshot::present_on_desktop` and `present_on_mobile` implement that rule.

## What an implementation must do

When a desktop node receives `session_offer`:

1. Open the x86 build of each app in the snapshot.
2. Restore every document, draft, and cursor.
3. Set `runtime_node_id` to the desktop and `runtime_arch` to `x86_64`.
4. Reply `session_accept` with `preserved: true` and the same `session_id`.
5. Send `desktop_mode` with `active: true`.

When the link drops:

1. The desktop writes the latest snapshot back.
2. The phone sets `runtime_node_id` to itself and `runtime_arch` to `aarch64`.
3. The phone build opens that latest snapshot.

## What an implementation must not do

- Treat a video stream of the phone screen as the handoff.
- Require the user to sign in again because the runtime changed.
- Move an ARM process into the desktop kernel. The desktop starts its own build.
- Open an Android-only package that has no x86 build.
- Put passwords or session cookies into `draft` while the channel is still plain TCP.

## Notes

Notes is the reference application because its state is obvious: a document id, a draft, and a cursor. A shell later can map `os.ocos.notes` to a real editor. Other apps add their own `app_id` and put their own small state in the same window object. Opaque blobs can be added when a real app needs them; do not break the fields above.
