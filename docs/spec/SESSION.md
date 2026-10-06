# Sessions

A session is the user's open work: which applications, which documents, where the cursor is, and what was not saved yet.

Desktop Mode does not create a second session. It presents the same one.

## Snapshot

```json
{
  "session_id": "ses_demo",
  "owner_node_id": "node_…",
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
| `owner_node_id` | Node whose processes these windows belong to |
| `mode` | `mobile` or `desktop`. Presentation only |
| `windows` | Open windows. Empty is a valid session |
| `app_id` | Stable application id. Notes is `os.ocos.notes` |
| `cursor` | Document offset the user had reached |
| `draft` | Uncommitted text or other small state |
| `bounds` | Optional `x`, `y`, `width`, `height` for a desktop shell |

`mode` may change when a cable is plugged or removed. `windows` may not be cleared as a side effect of that change. `ocos_session::SessionSnapshot::present_on_desktop` and `present_on_mobile` implement that rule.

## What an implementation must do

When a desktop node receives `session_offer`:

1. Keep every window in the snapshot.
2. Show them in Desktop presentation.
3. Reply `session_accept` with `preserved: true` and the same `session_id`.
4. Send `desktop_mode` with `active: true`.

When the link drops:

1. The owner node still has the processes.
2. Its shell returns to Mobile presentation.
3. The desktop shell drops the windows. It does not become a second owner of the draft.

## What an implementation must not do

- Relaunch the application on x86 and call that a handoff.
- Require the user to sign in again because the monitor changed.
- Move an arbitrary ARM process into the desktop kernel. That is live migration, and it is out of scope.
- Put passwords or session cookies into `draft` while the channel is still plain TCP.

## Notes

Notes is the reference application because its state is obvious: a document id, a draft, and a cursor. A shell later can map `os.ocos.notes` to a real editor. Other apps add their own `app_id` and put their own small state in the same window object. Opaque blobs can be added when a real app needs them; do not break the fields above.
