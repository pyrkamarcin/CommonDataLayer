# Documentation structure and RFC process

## Glossary

RFC - Request for Comments

## Documentation structure

### Architecture
* path: `docs/architecture`
* purpose: information about each component, schema registry, object builder, and other applications. It contains only our internal architecture description, the current state of each element. Information what it does, why and how, what algorithms (from a broad perspective without many implementation details) were used, the protocol between the systems, etc. It should be enough information to adapt a new developer to CDL quickly. 
* used by: CDL team

### Configuration
* path: `docs/configuration`
* purpose: description of all ENV variables and configuration files
* used by: CDL user

### Deployment
* path: `docs/deployment`
* purpose: documentation of how to deploy CDL
* used by: CDL user

### Examples
* path: `docs/examples`
* purpose: a place for tutorials and guides.
* used by: CDL user

### Features
* path: `docs/features`
* purpose: a place for feature (behavior) description. Each feature should have at least one document describing it. Example: ordering.md, materialization.md, routing.md, CDLIM_versioning.md
* used by: CDL user

### Processes
* path: `docs/processes`
* purpose: a place for all our processes. For example: Commit message formalization or this process.
* used by: CDL team

### RFCs
* path: `docs/rfc`
* purpose: a place for all RFCs.
* used by: CDL team

## RFC process

RFC describes only **change** to CDL. The change might be:
* New feature
* Modification of existing feature
* New process
* Modification of existing process
* Documentation about research done

It does not contain information about a single component (its current state, as this is the purpose of `architecture`).

### RFC Category

Each RFC must contain a category informing whether it describes the feature, process or the research.
Accepted values: `Feature`/`Process`/`Research`.

### RFC Deadline

Each RFC must contain a deadline to speed up the process.
Before the deadline is met, the author should remind all interested parties about it.

If the deadline is met without conclusion, the author of the RFC should create a meeting and invite all interested in these RFC parties. Then during this meeting resolution has to be made.

The deadline must be set at least 3 days from the announcement. RFC author should schedule the meeting on the nearest possible date that is acceptable for all parties.

Until the deadline, everyone is allowed to comment on the RFC. Participants should use the scheduled meeting only to decide when there is more than one strong opinion about the subject. All issues should be defined and discussed via GitHub comments. The deadline meeting should be relatively short if everyone agrees on the subject.

There may be exceptions to this rule - for example, when during a deadline meeting, everyone agrees to wait for new information from superiors, etc.

### RFC ordering number

Each RFC has an assigned ordering number that is an always-increasing integer. In rare situations, there might be gaps between two integers.
RFC ordering number is encoded by 4 digit (fixed length) decimal integer (values from `0001` to `9999`). In the future it might be extended by prefixing additional zeroes (`9999` becomes `09999`).
RFC ordering number has to be unique.

RFC ordering number and feature ID are not the same concepts.

### The process

#### Draft
The author of the RFC should create a new document in the `rfc` directory. This directory has a flat structure, and each RFC should have a filename in human-readable format with RFC ordering prefix number and version postfix number. For example: `1234_schema_registry_less_CDL_deployment_01.md`.

Version postfix number is increased when RFC describes changes for existing RFC; for example, when there is `xxx_feature_01.md`, the author wants to describe the modification or new approach to the feature (which isn't a new feature itself), they should write `yyy_feature_02.md`. In that case, however, the ordering number is always different - `xxx` < `yyy`.

Format:
`<RFC ordering number>_<Human readable name>_<Version postfix number>.md`

Each RFC has to contain Front Matter in the format:
```` 
# Front Matter

```
Title           : Schema-Registry-less CDL deployment
Category        : Feature
Author(s)       : Wojciech Polak
Team            : CommonDataLayer
Created         : 2021-06-24
Deadline        : 2021-06-26
CDL feature ID  : CDLF-00016-00
```
```` 

CDL feature ID is only required for feature descriptions. If the RFC describes a process, this field should be omitted.

Team field is optional. If missing, its implied that CDL Team has created it.

RFC format allows other custom fields to be included if necessary, such as why RFC has been abandoned.

For example, the author should commit the draft to a separate branch, for example, `rfc/<filename>`.


#### Pull Request
When the draft is ready for review, an author should create PR and inform reviewers.

#### Resolution
Each RFC must result in:
* When accepted:
    * If the team decide to implement feature described in the RFC:
        * Implementator should create tracking issue - he should include its id in `features/index.md` table.
        * Update in `architecture` - all components that were changed should be updated, so this RFC won't be needed to understand how for example, Object Builder works. It should be done as a part of PR that has been merged.
        * Update in `features` - if RFCs category is `Feature` - Update existing or create new feature.md and write a new description of what this feature does. In some cases, the author can selectively copy-paste it from RFC; however, the author must be careful. It should be done as a part of PR that has been merged.
    * If RFC describes the process: 
        * `processes/index.md` needs to be updated
        * Update in `processes` - if RFCs category is `Process` - Update existing or create a new process.md and write a further description of how the process looks. In some cases, the author can selectively copy-paste it from RFC; however, the author must be careful. Implementator should do it in PR that has been merged.
* If the deadline has been met, but no conclusion has been reached - RFC should be updated with the new deadline
(sub-issues).
* When abandoned - RFC should be updated with `Abandon reason` in Front Matter (top of the document).

It also means PRs with implementations should not be accepted and merged before merging documentation.
Exception: It is allowed to split PR into smaller, by merging the first and then separate PR documentation. However, until the last PR implementation is merged, **tracking issue has to be kept open**.

#### Further changes
Apart from minor typo fixes, and formatting RFC iss immutable.
Whenever there is a need to change the feature or the process, one should create another RFC describing new goals and changes.

### Summary
RFC describes **changes** while `features`/`processes`/`architecture` keep up-to-date information about the current state of CDL.
