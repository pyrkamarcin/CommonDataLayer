
# Front Matter

```
Title           : Development process
Category        : Process
Author(s)       : @Kononnable
Created         : 2021-07-26
Deadline        : 2021-08-03
```

# Goal
This RFC proposes changes to a development process to gain more insight into the state of each issue. Additionally, it splits the testing phase into smaller ones to properly map people responsible for each phase. It focuses only on tasks on the sprint board.

# Glossary
Issue - a specified problem we need to address, with description, acceptance criteria, etc. - sometimes called a task

Issue state - categorization over issues to more easily keep track of how much(and by whom) work is required to close a task. Each state of planned tasks should be mapped 1:1 to a column on a sprint board.


# Current State

Current sprint board columns on sprint board(`2021 - 28/30`):

To do(*) -> In progress -> Blocked -> Review in Progress -> Testing -> Done

> `*` - default state for new issues inserted into a sprint
# New Approach

Proposed issue states:

Blocked -> To do(*) -> In progress -> Code review -> Acceptance tests -> General testing -> Done 
> `*` - default state for new issues inserted into a sprint


## Definitions of each state:
- BLOCKED - special state, issues from any state can go directly to this column is blocked by something, when leaving they should go back to the previous state unless a resolution of blockade changed some important details about the task
- TODO: default state for new issues, work on the issue resolution has not yet been started 
- IN PROGRESS: task on which there is an active development
- CODE REVIEW: implementation is completed, the review should focus mostly on the code, checks its quality; for chores and other internal enhancements it should include testing if the change actually works(if there is something to test manually)
- ACCEPTANCE TESTS: waiting for automated tests to be written on new/changed functionality. Those tests should focus mostly on covering all of the use cases - check if business logic is implemented properly and if the feature is useful for the user. Corner cases and error handling doesn't have to be tested in this state. Additionally, architecture and general documentation should be written in the process(if not extracted to another task - needs discussion).
- GENERAL TESTS: waiting for automated and/or manual tests to be done on the functionality - checking for corner cases, proper error handling/propagation, testing in multiple different configurations(e.g. making sure feature works on Kafka, RabbitMQ, and Grpc environments)
- DONE: Work on the task was finished(task implemented or abandoned)

## Task types(related to different workflows):
- Feature - adding/changing behavior of a user-facing feature
- Bug - wrong behavior related to user-facing functionality
- Documentation - documentation related changes
- RFC - a request for comments on proposed change(feature or process-related)
- Internal enhancements, Chores - internal changes(refactors, dependency upgrades, code formatting, CI-related changes, etc.)


## Task state traversal
### General
| BEFORE      | AFTER       | CHANGE CONDITION   |
|-------------|-------------|--------------------|
| ANY         | DONE        | Task was discarded |
| ANY         | BLOCKED     | Moving task to next state is blocked by something, not dependant on the team(e.g. other team decision/reply) or on another issue |
|BLOCKED      | _PREVIOUS_  | Blocking reason was resolved. If resolve reason changes the task drastically new state should be TO DO or IN PROGRESS |
| ANY         | TODO        | During the work on the task we decided that it should be redefined/we need to take care of the problem in a different way, should not happen often(if at all), if we don't know the solution(needs RFC) task should go to BLOCKED instead |
| TODO        | IN PROGRESS |  Someone started working on the issue |
| IN PROGRESS | CODE REVIEW | Implementation work/Design is done, acceptance criteria are achieved |
### Feature
| BEFORE           | AFTER           | CHANGE CONDITION     |
|------------------|-----------------|----------------------|
| CODE REVIEW      | ACCEPTANCE TESTS| Code quality is fine |
| ACCEPTANCE TESTS | GENERAL TESTS   | Functionality works as defined in documentation | 
| GENERAL TESTS    | DONE            | Functionality works on all supported configurations, corner cases, error handling are covered by tests |
| ACCEPTANCE TESTS | IN PROGRESS     | Adding acceptance tests reviled that some parts of the functionality are missing/major use case is not working correctly. If that use case wasn't covered by documentation new feature related task should be added instead. |

### Bug
| BEFORE      | AFTER        | CHANGE CONDITION |
|-------------|--------------|------------------|
| CODE REVIEW | GENERAL TESTS| Code quality is fine, task should go to the top of the column(prioritizing) |
| GENERAL TESTS | DONE | Functionality works on all supported configurations, corner cases, error handling are covered by tests  |
|GENERAL TESTS | TO DO | Bug was not resolved. It's up to the tester to decide if the issue closed(during testing another wrong behavior was observed) or sent back to TO DO(the same wrong behavior still happens). If an issue is sent back person previously implementing the task should be assigned and the issue should go to the top of the TODO list. | 
### Documentation
| BEFORE      | AFTER | CHANGE CONDITION              |
|-------------|-------|-------------------------------|
| CODE REVIEW | DONE  | Documentation quality is fine |

### RFC
| BEFORE      | AFTER | CHANGE CONDITION |
|-------------|-------|------------------|
| CODE REVIEW | DONE  | RFC meeting ended with a positive conclusion. Problems mentioned at the meeting were fixed. PR with RFC are merged. |

### Internal enhancements, chores
| BEFORE      | AFTER | CHANGE CONDITION |
|-------------|-------|------------------|
| CODE REVIEW | DONE  | Code quality is fine, change works correctly(manual testing may be required) |
## What is considered an undelivered task in sprint scope

 In general, tasks taken for the sprint should be finished by end of the sprint. However, it's common to define some states which will be treated as valid for sprint end. 
Due to the difference in development vs testing manpower I recommend we don't treat ** feature-related** tasks in the GENERAL TESTING state as not-delivered. Such tasks should be added to the next sprint in the same column. This should ease our planning sessions and make workflow more fluid(as usually there are no new tasks to test at the beginning of the sprint).

> Note: Bugs in the GENERAL TESTING state should mark sprint as undelivered. They can move tasks back on the pipeline - they check if the issue was solved(with feature-related tasks this requirement is met on ACCEPTANCE TESTS state).

## Assignees
We can decide that tasks in specific states should have the correct people assigned to them. Assigning a proper person is a responsibility by of person changing the task state.
Those are general rules - we may make exceptions on per-issue cases(e.g. someone is out of office for the rest of the sprint, etc.).

BLOCKED - a person responsible for checking if blockade can be removed, most often a person to which issue was assigned before entering BLOCKED 

TODO - no assign, exceptions: bug issues getting back from GENERAL TESTS state(person implementing the issue), on planning we may decide that specific task should be done by a specific person(on per task basis)

IN PROGRESS - person actively working on the issue

CODE REVIEW - a person who implemented the issue and the main reviewer(picked automatically) - it's often for reviewers to have some comments that have to be addressed by the author

ACCEPTANCE TESTS - author of RFC describing the feature, can be changed to another person

GENERAL TESTING - no one - it's unknown if a task will be taken by a tester or by a developer, we're not planning tester sprint capability for the moment

DONE - not specified

> Note: With assigning reviewers to the issues we may decide not to assign reviewers to PRs - the purpose of this assignment(notification) is already served.
