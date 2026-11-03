# Tasks

## 1. Preparation

- [x] 1.1 Load the code skill: read its SKILL.md and the security, data, llm and ui references named by design.md, then follow those rules in every later task. Verify by recording the reference paths read.
- [x] 1.2 Inventory the files to edit with the code skill findings script before the first code edit. Verify the inventory names the existing thought, channel, store and page owners and the search terms used.

## 2. Core suggestion and confirm

- [x] 2.1 Add a failing core test for the migrated suggestion table, stable preview revision, stale revision and duplicate confirm, using the fake provider. Verify the selected test fails because confirm/suggest is missing, and that the test command selects it.
- [x] 2.2 Implement core suggestion, revision and confirm-to-draft until that test passes. Verify the same test passes and a provider failure still stores no piece.
- [x] 2.3 Remove any copied channel limit or second draft writer found while making 2.2 pass, without changing the passing assertions. Verify the same test still passes.

## 3. Loopback API and pages

- [x] 3.1 Add a failing server test for suggest, preview, confirm, malformed bodies, stale and duplicate confirmation, and Host, Origin and session rejection, against a running server and fake provider. Verify the new test fails because the routes are absent.
- [x] 3.2 Add thin routes and the separate capture pages until that test passes. Verify confirm creates pending drafts only, preview is labeled not an approval, and no approve or publish control is present.
- [x] 3.3 Update the migration row and studio guide to the rebuilt text path only. Verify attachment intake, URL fetch and confirm-and-publish remain unmigrated in the row.

## 4. Integrated proof

- [x] 4.1 Run strict OpenSpec validation and the change-shape checker. Verify both exit 0.
- [x] 4.2 Critique and audit the capture pages with the pinned Impeccable guidance, fix material findings, and save light/dark captures at 1440 and 390. Verify the four images have recorded hashes, dimensions and no horizontal overflow.
- [x] 4.3 Run the code skill verify script against the pre-change base with the project test command. Verify the receipt and each check result are recorded, including skips.

## 5. Prelanding narrow-header correction

- [x] 5.1 Reproduce Main1436's missing Workspace link on the owned server at 390px and observe the rendered assertion and stylesheet regression fail before changing CSS. Restore the existing shared-header link without duplicating navigation. Verify wide/narrow, both themes, keyboard return and no clipping, including long source.

Final handoff gate after committing the correction: refresh all four exact-head captures, image manifest, original-base Code Verify and clean FF receipt. Record individual check results and omissions, and provide the two corrected narrow images for firstmate to forward to Main before landing. This delivery gate does not authorize the worker to merge.
