# Details

Date : 2025-05-29 19:45:05

Directory /home/endre/rust_crates/micro_sp

Total : 51 files,  7773 codes, 1677 comments, 1134 blanks, all 10584 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [README.md](/README.md) | Markdown | 12 | 0 | 3 | 15 |
| [src/core/mod.rs](/src/core/mod.rs) | Rust | 7 | 0 | 1 | 8 |
| [src/core/sp\_assignment.rs](/src/core/sp_assignment.rs) | Rust | 98 | 6 | 18 | 122 |
| [src/core/sp\_goal.rs](/src/core/sp_goal.rs) | Rust | 100 | 2 | 11 | 113 |
| [src/core/sp\_state.rs](/src/core/sp_state.rs) | Rust | 476 | 20 | 54 | 550 |
| [src/core/sp\_value.rs](/src/core/sp_value.rs) | Rust | 803 | 10 | 105 | 918 |
| [src/core/sp\_variable.rs](/src/core/sp_variable.rs) | Rust | 99 | 46 | 20 | 165 |
| [src/core/sp\_wrapped.rs](/src/core/sp_wrapped.rs) | Rust | 213 | 4 | 21 | 238 |
| [src/core/structs.rs](/src/core/structs.rs) | Rust | 128 | 4 | 18 | 150 |
| [src/lib.rs](/src/lib.rs) | Rust | 49 | 1 | 8 | 58 |
| [src/macros/action.rs](/src/macros/action.rs) | Rust | 6 | 0 | 1 | 7 |
| [src/macros/mod.rs](/src/macros/mod.rs) | Rust | 5 | 0 | 1 | 6 |
| [src/macros/predicate.rs](/src/macros/predicate.rs) | Rust | 48 | 0 | 5 | 53 |
| [src/macros/sp\_assignment.rs](/src/macros/sp_assignment.rs) | Rust | 6 | 0 | 1 | 7 |
| [src/macros/sp\_variable.rs](/src/macros/sp_variable.rs) | Rust | 72 | 9 | 7 | 88 |
| [src/macros/transition.rs](/src/macros/transition.rs) | Rust | 30 | 0 | 2 | 32 |
| [src/modelling/action.rs](/src/modelling/action.rs) | Rust | 94 | 2 | 15 | 111 |
| [src/modelling/mod.rs](/src/modelling/mod.rs) | Rust | 7 | 0 | 0 | 7 |
| [src/modelling/model.rs](/src/modelling/model.rs) | Rust | 34 | 30 | 4 | 68 |
| [src/modelling/operation.rs](/src/modelling/operation.rs) | Rust | 240 | 20 | 21 | 281 |
| [src/modelling/parser.rs](/src/modelling/parser.rs) | Rust | 297 | 6 | 34 | 337 |
| [src/modelling/predicate.rs](/src/modelling/predicate.rs) | Rust | 465 | 10 | 34 | 509 |
| [src/modelling/sops.rs](/src/modelling/sops.rs) | Rust | 9 | 70 | 7 | 86 |
| [src/modelling/transition.rs](/src/modelling/transition.rs) | Rust | 479 | 314 | 76 | 869 |
| [src/planning/mod.rs](/src/planning/mod.rs) | Rust | 245 | 1 | 14 | 260 |
| [src/planning/operation.rs](/src/planning/operation.rs) | Rust | 72 | 1 | 3 | 76 |
| [src/planning/transition.rs](/src/planning/transition.rs) | Rust | 82 | 2 | 4 | 88 |
| [src/running/auto\_runner.rs](/src/running/auto_runner.rs) | Rust | 61 | 3 | 10 | 74 |
| [src/running/goal\_runner.rs](/src/running/goal_runner.rs) | Rust | 132 | 47 | 32 | 211 |
| [src/running/goal\_scheduler.rs](/src/running/goal_scheduler.rs) | Rust | 178 | 375 | 50 | 603 |
| [src/running/main\_runner.rs](/src/running/main_runner.rs) | Rust | 87 | 44 | 27 | 158 |
| [src/running/mod.rs](/src/running/mod.rs) | Rust | 8 | 1 | 0 | 9 |
| [src/running/operation\_runner.rs](/src/running/operation_runner.rs) | Rust | 337 | 207 | 67 | 611 |
| [src/running/planner\_ticker.rs](/src/running/planner_ticker.rs) | Rust | 163 | 10 | 15 | 188 |
| [src/running/state\_manager.rs](/src/running/state_manager.rs) | Rust | 916 | 53 | 136 | 1,105 |
| [src/running/utils.rs](/src/running/utils.rs) | Rust | 124 | 34 | 18 | 176 |
| [src/transforms/cycles.rs](/src/transforms/cycles.rs) | Rust | 255 | 52 | 50 | 357 |
| [src/transforms/examples/data/chair.json](/src/transforms/examples/data/chair.json) | JSON | 22 | 0 | 0 | 22 |
| [src/transforms/examples/data/couch.json](/src/transforms/examples/data/couch.json) | JSON | 22 | 0 | 0 | 22 |
| [src/transforms/examples/data/floor.json](/src/transforms/examples/data/floor.json) | JSON | 30 | 0 | 0 | 30 |
| [src/transforms/examples/data/food.json](/src/transforms/examples/data/food.json) | JSON | 22 | 0 | 0 | 22 |
| [src/transforms/examples/data/plate.json](/src/transforms/examples/data/plate.json) | JSON | 21 | 0 | 0 | 21 |
| [src/transforms/examples/data/table.json](/src/transforms/examples/data/table.json) | JSON | 21 | 0 | 0 | 21 |
| [src/transforms/examples/space\_tree.rs](/src/transforms/examples/space_tree.rs) | Rust | 102 | 2 | 44 | 148 |
| [src/transforms/examples/space\_tree\_ros.rs](/src/transforms/examples/space_tree_ros.rs) | Rust | 99 | 27 | 54 | 180 |
| [src/transforms/loading.rs](/src/transforms/loading.rs) | Rust | 255 | 54 | 31 | 340 |
| [src/transforms/lookup.rs](/src/transforms/lookup.rs) | Rust | 631 | 32 | 75 | 738 |
| [src/transforms/mod.rs](/src/transforms/mod.rs) | Rust | 4 | 0 | 0 | 4 |
| [src/transforms/treeviz.rs](/src/transforms/treeviz.rs) | Rust | 54 | 175 | 32 | 261 |
| [src/utils/logger.rs](/src/utils/logger.rs) | Rust | 52 | 3 | 5 | 60 |
| [src/utils/mod.rs](/src/utils/mod.rs) | Rust | 1 | 0 | 0 | 1 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)